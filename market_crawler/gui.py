from __future__ import annotations

import logging
import os
import platform
import queue
import sqlite3
import threading
import tkinter as tk
from tkinter import messagebox, ttk
from typing import Any

import matplotlib
import matplotlib.pyplot as plt
import mplfinance as mpf
import pandas as pd
import pandas_ta as ta
from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg

from market_crawler.classification import reclassify_database
from market_crawler.http_client import HttpPolicy
from market_crawler.orchestrator import classify_day, iter_date_strings
from market_crawler.pipeline import fetch_and_ingest_day, make_fetcher
from market_crawler.repository import SQLiteMarketRepository
from market_crawler.screener import screen_bollinger_squeeze
from market_crawler.yfinance_sync import sync_yfinance_to_db

# 設定 Matplotlib 支援 Tkinter 渲染並套用美觀樣式
matplotlib.use("TkAgg")
plt.style.use("ggplot")  # 使用 ggplot 樣式讓圖表更美觀

# 關閉 matplotlib 的 debug/warning 日誌，防止控制台被 findfont missing 警告訊息洗板
logging.getLogger("matplotlib").setLevel(logging.ERROR)

# 解決跨平台中文字型顯示問題，防止 Linux (DejaVu Sans) 出現 Glyph missing 警告與亂碼
_system = platform.system()
if _system == "Darwin":  # macOS
    matplotlib.rc("font", family="Arial Unicode MS")
elif _system == "Windows":
    matplotlib.rc("font", family="Microsoft JhengHei")
else:  # Linux / Unix
    # 將 Noto CJK 字型加入 matplotlib 字型管理
    for _dir in ("/usr/share/fonts/opentype/noto", "/tmp/noto_cjk"):
        if os.path.isdir(_dir):
            for _fname in os.listdir(_dir):
                if _fname.endswith((".ttf", ".otf", ".ttc")):
                    matplotlib.font_manager.fontManager.addfont(os.path.join(_dir, _fname))
    # 清除字型快取令新字型生效
    matplotlib.font_manager._load_fontmanager(try_read_cache=False)
    # 使用常見的 Linux 繁體中文/中文字型
    matplotlib.rc("font", family=["Noto Sans CJK TC", "Noto Sans CJK SC", "Source Han Sans TC", "Droid Sans Fallback", "sans-serif"])
matplotlib.rcParams["axes.unicode_minus"] = False  # 正常顯示負號

# 台灣 50 常見成份股與權值股，供使用者快速批次下載測試
TAIWAN_50_SYMBOLS = [
    "0050", "2330", "2317", "2454", "2308", "2881", "2882", "2303", "2891", "2886",
    "2884", "1216", "2892", "2885", "3711", "2382", "2880", "2883", "2890", "2002",
    "5880", "2357", "2301", "2603", "2327", "2409", "3045", "2609", "2408", "3008",
    "2615", "5876", "4938", "2395", "2379", "3231", "2474", "1301", "1303", "1326",
    "6505", "9904", "1101", "1402", "2105", "2912", "3034", "3037", "4904", "9910"
]


class StockScreenerApp:
    def __init__(self, root: tk.Tk) -> None:
        self.root = root
        self.root.title("台股布林通道擠壓選股與分析工具")
        self.root.geometry("1300x850")
        self.root.minsize(1024, 768)

        # 初始化資料庫連線與 Repository
        self.db_path = "market_data.db"
        self.conn = sqlite3.connect(self.db_path, check_same_thread=False, timeout=10)
        self.repo = SQLiteMarketRepository(self.conn)
        self.repo.init_schema()

        # 線程安全佇列與鎖
        self.task_queue: queue.Queue[tuple[str, Any]] = queue.Queue()
        self.db_lock = threading.Lock()

        # 篩選結果快取（供排序用）
        self._t2_df_res: pd.DataFrame | None = None

        # 補齊歷史股票名稱，需在 db_lock 初始化後執行
        self._patch_missing_security_names()

        # 背景重新分類既有資料（使用獨立連線避免鎖住主執行緒）
        def _reclassify_task() -> None:
            bg_conn = sqlite3.connect(self.db_path, timeout=10)
            try:
                reclassify_database(bg_conn)
            finally:
                bg_conn.close()

        threading.Thread(target=_reclassify_task, daemon=True).start()

        # 標記程式是否正在關閉中，用於安全取消定時器防止 race condition
        self.is_closing = False

        # 建立 UI 介面
        self._setup_styles()
        self._create_widgets()

        # 載入下拉選單初始資料
        self._t1_refresh_combo()
        self._t1_set_default_symbol("2330")

        # 綁定視窗關閉事件以優雅釋放資源
        self.root.protocol("WM_DELETE_WINDOW", self._on_closing)

        # 啟動佇列輪詢監聽
        self.after_id = self.root.after(100, self._process_queue)

    def _patch_missing_security_names(self) -> None:
        """修補資料庫中 security_name 為空或 NULL 的歷史記錄。"""
        import os
        try:
            # 1. 取得所有名稱為空或 NULL 的 symbols
            with self.db_lock:
                cur = self.conn.cursor()
                cur.execute("SELECT DISTINCT symbol_clean FROM market_quote_core WHERE security_name IS NULL OR security_name = ''")
                symbols = [r[0] for r in cur.fetchall()]

            if not symbols:
                return

            # 2. 載入對照表
            name_map = {}
            csv_path = "StockID.csv"
            if os.path.exists(csv_path):
                try:
                    from market_crawler.classification import load_security_master
                    master = load_security_master(csv_path)
                    name_map = {sym: entry.name for sym, entry in master.items()}
                except Exception:
                    pass

            # 再融合靜態表作為 fallback
            from market_crawler.yfinance_sync import _STATIC_SYMBOL_NAMES
            for sym, name in _STATIC_SYMBOL_NAMES.items():
                if sym not in name_map:
                    name_map[sym] = name

            # 3. 批次更新
            with self.db_lock:
                cur = self.conn.cursor()
                for sym in symbols:
                    name = name_map.get(sym)
                    if name:
                        cur.execute(
                            "UPDATE market_quote_core SET security_name = ? WHERE symbol_clean = ? AND (security_name IS NULL OR security_name = '')",
                            (name, sym)
                        )
                self.conn.commit()
        except Exception as e:
            # 僅靜默處理，不影響主程式啟動
            logging.error(f"自動補齊股票名稱失敗：{e}")

    def _setup_styles(self) -> None:
        """設定 ttk 風格樣式，提升介面美觀度。"""
        style = ttk.Style()
        style.theme_use("clam")

        # 定義高雅色彩
        primary_color = "#1a365d"  # 深藍
        accent_color = "#3182ce"   # 亮藍
        bg_light = "#f7fafc"

        style.configure(".", background=bg_light, font=("Microsoft JhengHei", 10))
        style.configure("TFrame", background=bg_light)
        style.configure("TLabel", background=bg_light, foreground="#2d3748")

        # 標題樣式
        style.configure("Header.TLabel", font=("Microsoft JhengHei", 16, "bold"), foreground=primary_color)
        style.configure("Section.TLabel", font=("Microsoft JhengHei", 12, "bold"), foreground=primary_color)

        # 按鈕樣式
        style.configure("Primary.TButton", background=accent_color, foreground="white", borderwidth=0, font=("Microsoft JhengHei", 10, "bold"))
        style.map("Primary.TButton", background=[("active", "#2b6cb0"), ("disabled", "#cbd5e0")])

        style.configure("Action.TButton", background=primary_color, foreground="white", borderwidth=0, font=("Microsoft JhengHei", 10, "bold"))
        style.map("Action.TButton", background=[("active", "#2c5282"), ("disabled", "#cbd5e0")])

        # Notebook 標籤樣式
        style.configure("TNotebook", background="#edf2f7")
        style.configure("TNotebook.Tab", font=("Microsoft JhengHei", 11, "bold"), padding=[12, 6])
        style.map("TNotebook.Tab", background=[("selected", bg_light)], foreground=[("selected", primary_color)])

    def _create_widgets(self) -> None:
        """建立主要 UI 佈局。"""
        # 主容器
        main_container = ttk.Frame(self.root, padding=10)
        main_container.pack(fill=tk.BOTH, expand=True)

        # 頂部標題區
        title_frame = ttk.Frame(main_container)
        title_frame.pack(fill=tk.X, pady=(0, 10))
        title_label = ttk.Label(title_frame, text="📈 輕量化布林通道擠壓選股分析系統", style="Header.TLabel")
        title_label.pack(side=tk.LEFT)

        # 狀態與進度條區
        self.status_var = tk.StringVar(value="系統就緒")

        # 結束程式按鈕（擺在最右側）
        self.exit_btn = ttk.Button(title_frame, text="🚪 結束程式", style="Action.TButton", command=self._on_closing)
        self.exit_btn.pack(side=tk.RIGHT, padx=(10, 0))

        self.status_label = ttk.Label(title_frame, textvariable=self.status_var, font=("Microsoft JhengHei", 10, "italic"), foreground="#4a5568")
        self.status_label.pack(side=tk.RIGHT, padx=10)

        self.progress_bar = ttk.Progressbar(main_container, orient=tk.HORIZONTAL, mode="indeterminate")
        self.progress_bar.pack(fill=tk.X, pady=(0, 10))

        # 分頁控制項
        self.notebook = ttk.Notebook(main_container)
        self.notebook.pack(fill=tk.BOTH, expand=True)

        # 建立三個分頁
        self.tab1 = ttk.Frame(self.notebook, padding=10)
        self.tab2 = ttk.Frame(self.notebook, padding=10)
        self.tab3 = ttk.Frame(self.notebook, padding=10)

        self.notebook.add(self.tab1, text="📊 單股分析與視覺化")
        self.notebook.add(self.tab2, text="🔍 布林擠壓選股器")
        self.notebook.add(self.tab3, text="⬇️ 下載管理")

        self._build_tab1()
        self._build_tab2()
        self._build_tab3()

    # ==================== TAB 1: 單股分析 ====================
    def _build_tab1(self) -> None:
        """建立單股分析分頁。"""
        # 左右分欄
        left_frame = ttk.Frame(self.tab1, width=320)
        left_frame.pack(side=tk.LEFT, fill=tk.Y, padx=(0, 10))
        left_frame.pack_propagate(False)

        right_frame = ttk.Frame(self.tab1)
        right_frame.pack(side=tk.RIGHT, fill=tk.BOTH, expand=True)

        # 左側控制面板
        ctrl_frame = ttk.LabelFrame(left_frame, text=" 參數設定與同步 ", padding=10)
        ctrl_frame.pack(fill=tk.X, pady=(0, 10))

        ttk.Label(ctrl_frame, text="股票代號:").grid(row=0, column=0, sticky=tk.W, pady=5)
        self.t1_symbol_combo = ttk.Combobox(ctrl_frame, width=13, state="normal")
        self.t1_symbol_combo.grid(row=0, column=1, sticky=tk.W, pady=5, padx=5)

        ttk.Label(ctrl_frame, text="市場類型:").grid(row=1, column=0, sticky=tk.W, pady=5)
        self.t1_market_combo = ttk.Combobox(ctrl_frame, values=["TWSE", "TPEX"], width=13, state="readonly")
        self.t1_market_combo.set("TWSE")
        self.t1_market_combo.grid(row=1, column=1, sticky=tk.W, pady=5, padx=5)

        ttk.Label(ctrl_frame, text="歷史期間:").grid(row=2, column=0, sticky=tk.W, pady=5)
        self.t1_period_combo = ttk.Combobox(ctrl_frame, values=["3mo", "6mo", "1y", "2y"], width=13, state="readonly")
        self.t1_period_combo.set("6mo")
        self.t1_period_combo.grid(row=2, column=1, sticky=tk.W, pady=5, padx=5)

        # 按鈕區
        btn_frame = ttk.Frame(ctrl_frame)
        btn_frame.grid(row=3, column=0, columnspan=2, pady=10, sticky=tk.EW)

        self.t1_sync_btn = ttk.Button(btn_frame, text="☁️ 下載/同步歷史數據", style="Action.TButton", command=self._t1_sync_data)
        self.t1_sync_btn.pack(fill=tk.X, pady=2)

        self.t1_plot_btn = ttk.Button(btn_frame, text="📈 繪製布林通道圖", style="Primary.TButton", command=self._t1_plot_data)
        self.t1_plot_btn.pack(fill=tk.X, pady=2)

        # 指標顯示區
        self.t1_info_frame = ttk.LabelFrame(left_frame, text=" 當前指標狀態 ", padding=10)
        self.t1_info_frame.pack(fill=tk.BOTH, expand=True)

        self.t1_info_text = tk.Text(self.t1_info_frame, wrap=tk.WORD, font=("Courier New", 10), bg="#ffffff", fg="#2d3748", bd=1, relief=tk.SOLID)
        self.t1_info_text.pack(fill=tk.BOTH, expand=True)
        self.t1_info_text.config(state=tk.DISABLED)

        # 右側 Matplotlib 畫布
        self.t1_fig, (self.t1_ax_price, self.t1_ax_vol) = plt.subplots(
            2, 1, sharex=True, gridspec_kw={"height_ratios": [3, 1]}, figsize=(8, 6)
        )
        self.t1_fig.tight_layout(pad=3.0)

        self.t1_canvas = FigureCanvasTkAgg(self.t1_fig, master=right_frame)
        self.t1_canvas.get_tk_widget().pack(fill=tk.BOTH, expand=True)
        self._t1_draw_placeholder("請輸入股票代號並點擊「繪製布林通道圖」")

    def _t1_draw_placeholder(self, message: str) -> None:
        """在單股分析圖表區繪製佔位文字。"""
        self.t1_ax_price.clear()
        self.t1_ax_vol.clear()
        self.t1_ax_price.text(0.5, 0.5, message, ha="center", va="center", fontsize=12, color="gray")
        self.t1_canvas.draw()

    # ==================== TAB 2: 布林擠壓篩選器 ====================
    def _build_tab2(self) -> None:
        """建立布林通道篩選器分頁。"""
        # 左右分欄
        left_frame = ttk.Frame(self.tab2, width=550)
        left_frame.pack(side=tk.LEFT, fill=tk.BOTH, expand=True, padx=(0, 10))

        right_frame = ttk.Frame(self.tab2, width=650)
        right_frame.pack(side=tk.RIGHT, fill=tk.BOTH, expand=True)
        right_frame.pack_propagate(False)

        # 左側控制與表格區
        ctrl_frame = ttk.LabelFrame(left_frame, text=" 篩選與批次下載設定 ", padding=10)
        ctrl_frame.pack(fill=tk.X, pady=(0, 10))

        # 篩選條件
        ttk.Label(ctrl_frame, text="帶寬上限:").grid(row=0, column=0, sticky=tk.W, pady=5)
        self.t2_limit_entry = ttk.Entry(ctrl_frame, width=10)
        self.t2_limit_entry.insert(0, "0.08")
        self.t2_limit_entry.grid(row=0, column=1, sticky=tk.W, pady=5, padx=5)

        ttk.Label(ctrl_frame, text="持續天數 (N):").grid(row=0, column=2, sticky=tk.W, pady=5, padx=(10, 0))
        self.t2_days_entry = ttk.Entry(ctrl_frame, width=10)
        self.t2_days_entry.insert(0, "5")
        self.t2_days_entry.grid(row=0, column=3, sticky=tk.W, pady=5, padx=5)

        # 最低成交量
        ttk.Label(ctrl_frame, text="最低成交量:").grid(row=1, column=0, sticky=tk.W, pady=5)
        self.t2_vol_entry = ttk.Entry(ctrl_frame, width=10)
        self.t2_vol_entry.insert(0, "0")
        self.t2_vol_entry.grid(row=1, column=1, sticky=tk.W, pady=5, padx=5)

        # 證券類型篩選 checkboxes
        type_frame = ttk.Frame(ctrl_frame)
        type_frame.grid(row=2, column=0, columnspan=4, sticky=tk.W, pady=5)
        ttk.Label(type_frame, text="證券類型:").pack(side=tk.LEFT)
        self.t2_type_vars: dict[str, tk.BooleanVar] = {}
        type_labels = [
            ("stock", "股票"),
            ("etf", "ETF"),
            ("warrant", "權證"),
            ("convertible_bond", "可轉債"),
            ("preferred_stock", "特別股"),
            ("other", "其他"),
        ]
        for i, (key, label) in enumerate(type_labels):
            var = tk.BooleanVar(value=(key == "stock"))
            cb = ttk.Checkbutton(type_frame, text=label, variable=var)
            cb.pack(side=tk.LEFT, padx=(10 if i > 0 else 5, 0))
            if key == "stock":
                cb.config(state=tk.DISABLED)
            self.t2_type_vars[key] = var

        # 控制按鈕
        self.t2_screen_btn = ttk.Button(ctrl_frame, text="🔍 開始本地篩選", style="Primary.TButton", command=self._t2_run_screener)
        self.t2_screen_btn.grid(row=3, column=0, columnspan=2, pady=10, sticky=tk.EW)

        self.t2_batch_btn = ttk.Button(ctrl_frame, text="☁️ 批次同步台灣50歷史日K", style="Action.TButton", command=self._t2_batch_sync)
        self.t2_batch_btn.grid(row=3, column=2, columnspan=2, pady=10, sticky=tk.EW, padx=(10, 0))

        # 排序狀態
        self.t2_sort_col: str | None = None
        self.t2_sort_asc = True

        # 表格顯示區
        table_frame = ttk.LabelFrame(left_frame, text=" 篩選符合條件股票清單 (點選可即時更新右側圖表) ", padding=5)
        table_frame.pack(fill=tk.BOTH, expand=True)

        columns = ("symbol", "name", "latest_close", "latest_volume", "max_bandwidth", "latest_bandwidth")
        self.t2_tree = ttk.Treeview(table_frame, columns=columns, show="headings", selectmode="browse")

        col_texts = [
            ("symbol", "股票代號"),
            ("name", "股票名稱"),
            ("latest_close", "最新收盤價"),
            ("latest_volume", "最新成交量"),
            ("max_bandwidth", "N日內最大頻寬"),
            ("latest_bandwidth", "最新頻寬"),
        ]
        for col_id, text in col_texts:
            self.t2_tree.heading(
                col_id,
                text=text,
                command=lambda c=col_id: self._t2_sort_by_column(c),
            )

        # 設定欄位寬度與置中
        self.t2_tree.column("symbol", width=70, anchor=tk.CENTER)
        self.t2_tree.column("name", width=90, anchor=tk.CENTER)
        self.t2_tree.column("latest_close", width=90, anchor=tk.E)
        self.t2_tree.column("latest_volume", width=110, anchor=tk.E)
        self.t2_tree.column("max_bandwidth", width=110, anchor=tk.E)
        self.t2_tree.column("latest_bandwidth", width=90, anchor=tk.E)

        # 捲軸
        vsb = ttk.Scrollbar(table_frame, orient="vertical", command=self.t2_tree.yview)
        self.t2_tree.configure(yscrollcommand=vsb.set)

        self.t2_tree.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        vsb.pack(side=tk.RIGHT, fill=tk.Y)

        # 綁定點選事件
        self.t2_tree.bind("<<TreeviewSelect>>", self._t2_on_row_select)

        # 批次下載的日誌框
        log_frame = ttk.LabelFrame(left_frame, text=" 同步進度與日誌 ", padding=5)
        log_frame.pack(fill=tk.X, pady=(10, 0))

        self.t2_log_text = tk.Text(log_frame, height=6, wrap=tk.WORD, font=("Courier New", 9), bg="#f7fafc", fg="#4a5568")
        self.t2_log_text.pack(fill=tk.BOTH, expand=True)
        self.t2_log_text.config(state=tk.DISABLED)

        # 右側 Matplotlib 畫布 (篩選聯動繪圖)
        self.t2_fig, (self.t2_ax_price, self.t2_ax_vol) = plt.subplots(
            2, 1, sharex=True, gridspec_kw={"height_ratios": [3, 1]}, figsize=(6, 6)
        )
        self.t2_fig.tight_layout(pad=3.0)

        self.t2_canvas = FigureCanvasTkAgg(self.t2_fig, master=right_frame)
        self.t2_canvas.get_tk_widget().pack(fill=tk.BOTH, expand=True)
        self._t2_draw_placeholder("請在左側表格選取股票以繪製圖表")

    def _t2_draw_placeholder(self, message: str) -> None:
        """在選股器圖表區繪製佔位文字。"""
        self.t2_ax_price.clear()
        self.t2_ax_vol.clear()
        self.t2_ax_price.text(0.5, 0.5, message, ha="center", va="center", fontsize=11, color="gray")
        self.t2_canvas.draw()

    # ==================== TAB 3: 下載管理 ====================
    def _build_tab3(self) -> None:
        """建立下載管理分頁。"""
        # 上方面板：模式選擇與日期輸入
        ctrl_frame = ttk.LabelFrame(self.tab3, text=" 下載設定 ", padding=10)
        ctrl_frame.pack(fill=tk.X, pady=(0, 10))

        ttk.Label(ctrl_frame, text="下載模式:").grid(row=0, column=0, sticky=tk.W, pady=5)
        self.t3_mode_combo = ttk.Combobox(ctrl_frame, values=["下載今日", "指定日期", "日期區間"], width=15, state="readonly")
        self.t3_mode_combo.set("下載今日")
        self.t3_mode_combo.grid(row=0, column=1, sticky=tk.W, pady=5, padx=5)
        self.t3_mode_combo.bind("<<ComboboxSelected>>", self._t3_on_mode_change)

        self.t3_date_label = ttk.Label(ctrl_frame, text="日期 (YYYY-MM-DD):")
        self.t3_date_label.grid(row=0, column=2, sticky=tk.W, pady=5, padx=(15, 0))
        self.t3_date_entry = ttk.Entry(ctrl_frame, width=14)
        self.t3_date_entry.grid(row=0, column=3, sticky=tk.W, pady=5, padx=5)
        self.t3_date_cal_btn = ttk.Button(ctrl_frame, text="📅", width=3, command=lambda: self._show_calendar(self.t3_date_entry))
        self.t3_date_cal_btn.grid(row=0, column=4, sticky=tk.W, pady=5)

        self.t3_start_label = ttk.Label(ctrl_frame, text="起始日期:")
        self.t3_start_entry = ttk.Entry(ctrl_frame, width=14)
        self.t3_start_cal_btn = ttk.Button(ctrl_frame, text="📅", width=3, command=lambda: self._show_calendar(self.t3_start_entry))
        self.t3_end_label = ttk.Label(ctrl_frame, text="結束日期:")
        self.t3_end_entry = ttk.Entry(ctrl_frame, width=14)
        self.t3_end_cal_btn = ttk.Button(ctrl_frame, text="📅", width=3, command=lambda: self._show_calendar(self.t3_end_entry))

        self.t3_holiday_var = tk.BooleanVar(value=True)
        self.t3_holiday_cb = ttk.Checkbutton(ctrl_frame, text="自動跳過假日", variable=self.t3_holiday_var)

        self.t3_dl_btn = ttk.Button(ctrl_frame, text="⬇️ 開始下載所有股票", style="Primary.TButton", command=self._t3_start_download)
        self.t3_dl_btn.grid(row=1, column=0, columnspan=6, pady=10, sticky=tk.EW)

        self._t3_update_mode_visibility()

        # 下載日誌區
        log_frame = ttk.LabelFrame(self.tab3, text=" 下載進度與日誌 ", padding=5)
        log_frame.pack(fill=tk.BOTH, expand=True)

        self.t3_log_text = tk.Text(log_frame, wrap=tk.WORD, font=("Courier New", 10), bg="#f7fafc", fg="#4a5568")
        self.t3_log_text.pack(fill=tk.BOTH, expand=True)
        self.t3_log_text.config(state=tk.DISABLED)

    def _t3_update_mode_visibility(self) -> None:
        """根據選取的模式顯示/隱藏對應的日期輸入元件。"""
        mode = self.t3_mode_combo.get()

        for w in (self.t3_date_label, self.t3_date_entry, self.t3_date_cal_btn,
                  self.t3_start_label, self.t3_start_entry, self.t3_start_cal_btn,
                  self.t3_end_label, self.t3_end_entry, self.t3_end_cal_btn,
                  self.t3_holiday_cb):
            w.grid_remove()

        if mode == "指定日期":
            self.t3_date_label.grid(row=0, column=2, sticky=tk.W, pady=5, padx=(15, 0))
            self.t3_date_entry.grid(row=0, column=3, sticky=tk.W, pady=5, padx=5)
            self.t3_date_cal_btn.grid(row=0, column=4, sticky=tk.W, pady=5)
        elif mode == "日期區間":
            self.t3_start_label.grid(row=0, column=2, sticky=tk.W, pady=5, padx=(15, 0))
            self.t3_start_entry.grid(row=0, column=3, sticky=tk.W, pady=5, padx=5)
            self.t3_start_cal_btn.grid(row=0, column=4, sticky=tk.W, pady=5)
            self.t3_end_label.grid(row=0, column=5, sticky=tk.W, pady=5, padx=(10, 0))
            self.t3_end_entry.grid(row=0, column=6, sticky=tk.W, pady=5, padx=5)
            self.t3_end_cal_btn.grid(row=0, column=7, sticky=tk.W, pady=5)
            self.t3_holiday_cb.grid(row=0, column=8, sticky=tk.W, pady=5, padx=(15, 0))

    def _t3_on_mode_change(self, event: Any = None) -> None:
        """模式切換時更新介面。"""
        self._t3_update_mode_visibility()

    def _show_calendar(self, target_entry: ttk.Entry) -> None:
        """彈出月曆視窗讓使用者選取日期，選取後填入 target_entry。"""
        import calendar
        from datetime import date

        today = date.today()
        cal_win = tk.Toplevel(self.root)
        cal_win.title("選取日期")
        cal_win.resizable(False, False)
        cal_win.transient(self.root)
        cal_win.grab_set()

        cur_year = today.year
        cur_month = today.month

        header_frame = ttk.Frame(cal_win, padding=5)
        header_frame.pack(fill=tk.X)

        year_var = tk.IntVar(value=cur_year)
        month_var = tk.IntVar(value=cur_month)

        def _refresh():
            for w in day_frame.winfo_children():
                w.destroy()
            cal = calendar.monthcalendar(year_var.get(), month_var.get())
            month_name = f"{year_var.get()} 年 {month_var.get()} 月"
            month_label.config(text=month_name)
            for r, week in enumerate(cal):
                for c, day in enumerate(week):
                    if day == 0:
                        continue
                    btn = tk.Button(
                        day_frame, text=str(day), width=3, padx=2, pady=1,
                        command=lambda d=date(year_var.get(), month_var.get(), day): _pick(d),
                    )
                    btn.grid(row=r, column=c, padx=1, pady=1)

        def _prev_month():
            m = month_var.get()
            y = year_var.get()
            if m == 1:
                month_var.set(12)
                year_var.set(y - 1)
            else:
                month_var.set(m - 1)
            _refresh()

        def _next_month():
            m = month_var.get()
            y = year_var.get()
            if m == 12:
                month_var.set(1)
                year_var.set(y + 1)
            else:
                month_var.set(m + 1)
            _refresh()

        def _prev_year():
            year_var.set(year_var.get() - 1)
            _refresh()

        def _next_year():
            year_var.set(year_var.get() + 1)
            _refresh()

        def _quick_today():
            _pick(today)

        def _pick(d: date) -> None:
            target_entry.delete(0, tk.END)
            target_entry.insert(0, d.isoformat())
            cal_win.destroy()

        ttk.Button(header_frame, text="◀◀", width=3, command=_prev_year).pack(side=tk.LEFT)
        ttk.Button(header_frame, text="◀", width=3, command=_prev_month).pack(side=tk.LEFT, padx=2)
        month_label = ttk.Label(header_frame, text="", width=18, anchor=tk.CENTER, font=("", 11, "bold"))
        month_label.pack(side=tk.LEFT, padx=5, fill=tk.X, expand=True)
        ttk.Button(header_frame, text="▶", width=3, command=_next_month).pack(side=tk.LEFT, padx=2)
        ttk.Button(header_frame, text="▶▶", width=3, command=_next_year).pack(side=tk.LEFT)

        # 星期標題
        dow_frame = ttk.Frame(cal_win, padding=(5, 0, 5, 0))
        dow_frame.pack(fill=tk.X)
        weekdays = ["一", "二", "三", "四", "五", "六", "日"]
        for i, d in enumerate(weekdays):
            fg = "red" if i >= 5 else "black"
            tk.Label(dow_frame, text=d, width=3, anchor=tk.CENTER, fg=fg).pack(side=tk.LEFT, padx=1)

        day_frame = ttk.Frame(cal_win, padding=5)
        day_frame.pack(fill=tk.BOTH)

        # 底部按鈕
        btn_frame = ttk.Frame(cal_win, padding=5)
        btn_frame.pack(fill=tk.X)
        ttk.Button(btn_frame, text="今天", command=_quick_today).pack(side=tk.LEFT, padx=5)
        ttk.Button(btn_frame, text="取消", command=cal_win.destroy).pack(side=tk.RIGHT, padx=5)

        _refresh()

        x = target_entry.winfo_rootx()
        y = target_entry.winfo_rooty() + target_entry.winfo_height()
        cal_win.geometry(f"+{x}+{y}")

    def _write_log(self, message: str) -> None:
        """向日誌視窗寫入文字。"""
        self.t2_log_text.config(state=tk.NORMAL)
        self.t2_log_text.insert(tk.END, message + "\n")
        self.t2_log_text.see(tk.END)
        self.t2_log_text.config(state=tk.DISABLED)

    # ==================== 異步與多線程處理邏輯 ====================
    def _process_queue(self) -> None:  # noqa: C901
        """主執行緒定期輪詢任務佇列，更新 UI 狀態。"""
        try:
            while True:
                msg_type, status, data = self.task_queue.get_nowait()

                if msg_type == "SYNC":
                    self.progress_bar.stop()
                    self._enable_all_buttons()
                    if status == "SUCCESS":
                        count = data
                        self.status_var.set("下載成功")
                        self._t1_refresh_combo()  # 重新整理選單
                        messagebox.showinfo("成功", f"歷史數據同步完成！共匯入 {count} 筆記錄。")
                        self._t1_plot_data()  # 同步完自動畫圖
                    else:
                        self.status_var.set("下載失敗")
                        messagebox.showerror("錯誤", f"同步失敗：\n{data}")

                elif msg_type == "AUTO_SYNC_SUCCESS":
                    self.progress_bar.stop()
                    self._enable_all_buttons()
                    symbol, count = status, data
                    self.status_var.set(f"自動下載成功 ({symbol})")
                    self._t1_refresh_combo()  # 重新整理選單
                    self._t1_set_default_symbol(symbol)  # 選取該股票
                    self._t1_plot_data()  # 自動畫圖

                elif msg_type == "AUTO_SYNC_EMPTY":
                    self.progress_bar.stop()
                    self._enable_all_buttons()
                    symbol, _ = status, data
                    self.status_var.set("自動下載無數據")
                    self._t1_draw_placeholder(f"[{symbol}] yfinance 同步無數據，請確認代號與市場。")

                elif msg_type == "AUTO_SYNC_ERROR":
                    self.progress_bar.stop()
                    self._enable_all_buttons()
                    symbol, error_msg = status, data
                    self.status_var.set("自動下載錯誤")
                    self._t1_draw_placeholder(f"[{symbol}] 自動同步失敗：\n{error_msg}")

                elif msg_type == "SCREEN":
                    self.progress_bar.stop()
                    self._enable_all_buttons()
                    if status == "SUCCESS":
                        df_res: pd.DataFrame = data
                        self.status_var.set(f"篩選完成 (符合: {len(df_res)} 檔)")
                        self._t2_update_treeview(df_res)
                    else:
                        self.status_var.set("篩選錯誤")
                        messagebox.showerror("錯誤", f"選股計算失敗：\n{data}")

                elif msg_type == "BATCH_PROGRESS":
                    # 單純輸出日誌，不停止進度條
                    self._write_log(data)

                elif msg_type == "BATCH_DONE":
                    self.progress_bar.stop()
                    self._enable_all_buttons()
                    success_count, failed_count = data
                    self.status_var.set(f"批次同步完成 (成功: {success_count}, 失敗: {failed_count})")
                    self._t1_refresh_combo()  # 重新整理選單
                    messagebox.showinfo("完成", f"台灣 50 批次同步結束！\n成功：{success_count} 檔\n失敗：{failed_count} 檔")

                elif msg_type == "BULK_LOG":
                    self._t3_write_log(data)

                elif msg_type == "BULK_PROGRESS":
                    day, current, total = status, data[0], data[1]
                    self.status_var.set(f"正在處理 {day} ({current}/{total})")
                    self.progress_bar["mode"] = "determinate"
                    # 進度條（含跳過的週末/假日）
                    self.progress_bar["maximum"] = total
                    self.progress_bar["value"] = current

                elif msg_type == "BULK_DONE":
                    self.progress_bar.stop()
                    self.progress_bar["value"] = 0
                    self._enable_all_buttons()
                    summary = data
                    if status == "success":
                        self.status_var.set("✓ 下載完成")
                        self._t3_write_log("\n✓ 完成：當日資料下載成功")
                        messagebox.showinfo("完成", "當日收盤行情下載成功！")
                    elif status == "warning":
                        self.status_var.set("- 當日休市")
                        self._t3_write_log("\n- 完成：當日休市（無資料）")
                        messagebox.showinfo("提示", "當日為休市日，無資料可下載。")
                    elif status == "failed":
                        self.status_var.set("✗ 下載失敗")
                        self._t3_write_log("\n✗ 完成：下載失敗")
                        messagebox.showerror("錯誤", "當日資料下載失敗，請檢查網路或稍後重試。")
                    elif status == "error":
                        self.status_var.set("✗ 下載異常中斷")
                        self._t3_write_log("\n✗ 下載程序異常中斷")
                        messagebox.showerror("錯誤", "下載程序發生未預期錯誤。")
                    elif status == "range_done":
                        s = summary["success"]
                        m = summary["market_closed"]
                        f = summary["failed"]
                        sk = summary.get("skipped", 0)
                        self.status_var.set(f"✓ 區間下載完成 (成功 {s} / 休市 {m} / 失敗 {f})")
                        self._t3_write_log(
                            f"\n{'='*40}"
                            f"\n📊 下載摘要"
                            f"\n  成功天數：{s}"
                            f"\n  休市天數：{m}"
                            f"\n  失敗天數：{f}"
                            f"\n  跳過假日：{sk}"
                            f"\n{'='*40}"
                        )
                        msg = (
                            f"區間下載完成！\n\n"
                            f"✅ 成功：{s} 天\n"
                            f"➖ 休市：{m} 天\n"
                            f"❌ 失敗：{f} 天"
                        )
                        if f > 0:
                            messagebox.showwarning("完成", msg)
                        else:
                            messagebox.showinfo("完成", msg)
                    self._t1_refresh_combo()

                self.task_queue.task_done()
        except queue.Empty:
            pass
        finally:
            if not getattr(self, "is_closing", False):
                try:
                    self.after_id = self.root.after(100, self._process_queue)
                except Exception:
                    self.after_id = None

    def _on_closing(self) -> None:
        """在關閉視窗時，優雅釋放資料庫連線等資源並退出。"""
        self.is_closing = True

        # 取消未執行的 after() 定時器，防止退出後出現 invalid command name 警告
        if hasattr(self, "after_id") and self.after_id:
            try:
                self.root.after_cancel(self.after_id)
            except Exception:
                pass

        try:
            with self.db_lock:
                self.conn.close()
        except Exception:
            pass
        self.root.quit()

    def _disable_all_buttons(self) -> None:
        """執行耗時任務時禁用所有按鈕，防重複點擊。"""
        self.t1_sync_btn.config(state=tk.DISABLED)
        self.t1_plot_btn.config(state=tk.DISABLED)
        self.t2_screen_btn.config(state=tk.DISABLED)
        self.t2_batch_btn.config(state=tk.DISABLED)
        self.t3_dl_btn.config(state=tk.DISABLED)
        self.exit_btn.config(state=tk.DISABLED)

    def _enable_all_buttons(self) -> None:
        """任務結束後啟用所有按鈕。"""
        self.t1_sync_btn.config(state=tk.NORMAL)
        self.t1_plot_btn.config(state=tk.NORMAL)
        self.t2_screen_btn.config(state=tk.NORMAL)
        self.t2_batch_btn.config(state=tk.NORMAL)
        self.t3_dl_btn.config(state=tk.NORMAL)
        self.exit_btn.config(state=tk.NORMAL)

    # ==================== 單股分析事件 ====================
    def _t1_sync_data(self) -> None:
        """按下單股同步按鈕。"""
        raw_val = self.t1_symbol_combo.get().strip()
        if not raw_val:
            messagebox.showwarning("警告", "請先輸入股票代號！")
            return
        symbol = raw_val.split(" - ")[0].strip()

        market = self.t1_market_combo.get()
        period = self.t1_period_combo.get()

        self._disable_all_buttons()
        self.progress_bar.start(10)
        self.status_var.set(f"正在自 yfinance 下載 {symbol} ({period}) ...")

        def worker() -> None:
            try:
                with self.db_lock:
                    count = sync_yfinance_to_db(self.repo, symbol, market, period=period)
                self.task_queue.put(("SYNC", "SUCCESS", count))
            except Exception as e:
                self.task_queue.put(("SYNC", "ERROR", str(e)))

        threading.Thread(target=worker, daemon=True).start()

    def _t1_plot_data(self) -> None:
        """讀取本地資料並繪製布林通道圖。"""
        raw_val = self.t1_symbol_combo.get().strip()
        if not raw_val:
            messagebox.showwarning("警告", "請先輸入股票代號！")
            return
        symbol = raw_val.split(" - ")[0].strip()

        with self.db_lock:
            df = pd.read_sql_query(
                """
                SELECT trade_date, close_price, volume_shares, security_name
                FROM market_quote_core
                WHERE symbol_clean = ?
                ORDER BY trade_date ASC
                """,
                self.conn,
                params=(symbol,)
            )

        if df.empty:
            self._t1_auto_sync_and_plot(symbol)
            return

        if len(df) < 20:
            self._t1_draw_placeholder(f"[{symbol}] 資料筆數 ({len(df)} 筆) 不足 20 筆，無法計算布林通道。")
            self._update_t1_info_text(df)
            return

        # 計算布林通道
        try:
            bb = ta.bbands(df["close_price"], length=20, std=2)
            if bb is None or bb.empty:
                raise ValueError("布林通道計算傳回空值")
        except Exception as e:
            self._t1_draw_placeholder(f"指標計算錯誤：{str(e)}")
            return

        # 計算帶寬
        bbl = bb.iloc[:, 0]
        bbm = bb.iloc[:, 1]
        bbu = bb.iloc[:, 2]
        df["bandwidth"] = (bbu - bbl) / bbm.replace(0, float("nan"))
        df["bbl"] = bbl
        df["bbm"] = bbm
        df["bbu"] = bbu

        # 轉換日期以獲得漂亮的 Matplotlib 日期格式
        dates = pd.to_datetime(df["trade_date"])

        # 繪製圖表
        self.t1_ax_price.clear()
        self.t1_ax_vol.clear()

        # 1. 價格主圖
        self.t1_ax_price.plot(dates, df["close_price"], label="收盤價", color="#1a365d", linewidth=1.5)
        self.t1_ax_price.plot(dates, df["bbm"], label="20 MA (中軌)", color="#dd6b20", linestyle="--", linewidth=1.0)
        self.t1_ax_price.plot(dates, df["bbu"], label="布林上軌", color="#319795", linewidth=1.0, alpha=0.8)
        self.t1_ax_price.plot(dates, df["bbl"], label="布林下軌", color="#319795", linewidth=1.0, alpha=0.8)

        # 填充布林通道半透明區間
        self.t1_ax_price.fill_between(dates, df["bbl"], df["bbu"], facecolor="#ebf8ff", alpha=0.6, label="布林通道區間")

        # 取得最新且非空的股票名稱
        names_non_null = df["security_name"].dropna()
        name = names_non_null.iloc[-1] if not names_non_null.empty else None
        title_name = f" {name}" if name else ""

        self.t1_ax_price.set_title(f"[{symbol}]{title_name} 布林通道走勢圖 (20, 2)", fontsize=12, fontweight="bold", color="#1a365d")
        self.t1_ax_price.set_ylabel("股價")
        self.t1_ax_price.legend(loc="upper left", framealpha=0.9, facecolor="white", fontsize=8)

        # 2. 成交量副圖
        self.t1_ax_vol.bar(dates, df["volume_shares"], color="#a0aec0", alpha=0.7, width=0.6, label="成交量")
        self.t1_ax_vol.set_ylabel("成交股數")
        self.t1_ax_vol.set_xlabel("交易日期")

        # 自動調整 X 軸日期排列
        self.t1_fig.autofmt_xdate()
        self.t1_canvas.draw()

        self._update_t1_info_text(df)

    def _t1_load_db_choices(self) -> list[str]:
        """從資料庫讀取已存在的所有股票代號與名稱，格式化為 '代號 - 名稱'。"""
        try:
            with self.db_lock:
                cur = self.conn.cursor()
                cur.execute("""
                    SELECT symbol_clean, MIN(security_name)
                    FROM market_quote_core
                    GROUP BY symbol_clean
                    ORDER BY symbol_clean ASC
                """)
                rows = cur.fetchall()

            choices = []
            for sym, name in rows:
                if name:
                    choices.append(f"{sym} - {name}")
                else:
                    choices.append(f"{sym}")
            return choices
        except Exception:
            return []

    def _t1_refresh_combo(self) -> None:
        """更新單股分析中股票代號下拉選單的選項。"""
        choices = self._t1_load_db_choices()
        self.t1_symbol_combo["values"] = choices

    def _t1_set_default_symbol(self, default_sym: str) -> None:
        """嘗試設定選單的預設選取股票。"""
        choices = self.t1_symbol_combo["values"]
        matched = None
        for choice in choices:
            if choice.startswith(default_sym):
                matched = choice
                break

        if matched:
            self.t1_symbol_combo.set(matched)
        else:
            self.t1_symbol_combo.set(default_sym)

    def _t1_auto_sync_and_plot(self, symbol: str) -> None:
        """當本地無資料時，自動發起背景下載，下載完成後自動畫圖。"""
        market = self.t1_market_combo.get()
        period = self.t1_period_combo.get()

        self._disable_all_buttons()
        self.progress_bar.start(10)
        self.status_var.set(f"本地無資料，自動自 yfinance 同步 {symbol}... (期間: {period})")

        def worker() -> None:
            try:
                with self.db_lock:
                    count = sync_yfinance_to_db(self.repo, symbol, market, period=period)
                if count > 0:
                    self.task_queue.put(("AUTO_SYNC_SUCCESS", symbol, count))
                else:
                    self.task_queue.put(("AUTO_SYNC_EMPTY", symbol, "yfinance 沒有回傳任何數據"))
            except Exception as e:
                self.task_queue.put(("AUTO_SYNC_ERROR", symbol, str(e)))

        threading.Thread(target=worker, daemon=True).start()

    def _update_t1_info_text(self, df: pd.DataFrame | None) -> None:
        """更新單股指標的文字資訊區。"""
        self.t1_info_text.config(state=tk.NORMAL)
        self.t1_info_text.delete("1.0", tk.END)

        if df is None:
            self.t1_info_text.insert(tk.END, "尚無數據。\n請先執行數據同步與繪圖。")
            self.t1_info_text.config(state=tk.DISABLED)
            return

        latest_idx = -1
        latest_row = df.iloc[latest_idx]

        # 取得最新且非空的股票名稱
        names_non_null = df["security_name"].dropna()
        name = names_non_null.iloc[-1] if not names_non_null.empty else None

        info = []
        info.append("====== 基本資訊 ======")
        # 從下拉選單取得解析後代號
        combo_val = self.t1_symbol_combo.get().strip()
        info_symbol = combo_val.split(" - ")[0].strip() if combo_val else ""
        info.append(f"代號: {info_symbol}")
        if name:
            info.append(f"名稱: {name}")
        info.append(f"總資料天數: {len(df)} 天")
        info.append(f"最新交易日: {latest_row['trade_date']}")
        info.append(f"最新收盤價: {latest_row['close_price']:.2f}")
        info.append(f"最新成交量: {int(latest_row['volume_shares']):,}")

        if "bandwidth" in latest_row and pd.notna(latest_row["bandwidth"]):
            info.append("\n====== 布林通道指標 ======")
            info.append(f"布林上軌: {latest_row['bbu']:.2f}")
            info.append(f"布林中軌: {latest_row['bbm']:.2f}")
            info.append(f"布林下軌: {latest_row['bbl']:.2f}")
            info.append(f"布林頻寬: {latest_row['bandwidth'] * 100:.2f}%")

            # 計算最近 5 天與 10 天的頻寬波動
            recent_5 = df["bandwidth"].tail(5)
            info.append(f"5日最大頻寬: {recent_5.max() * 100:.2f}%")
            info.append(f"5日最小頻寬: {recent_5.min() * 100:.2f}%")

            # 判斷是否處於擠壓狀態
            is_squeeze = "是 (<= 8%)" if latest_row["bandwidth"] <= 0.08 else "否 (> 8%)"
            info.append(f"布林擠壓狀態: {is_squeeze}")

        self.t1_info_text.insert(tk.END, "\n".join(info))
        self.t1_info_text.config(state=tk.DISABLED)

    # ==================== 選股篩選事件 ====================
    def _t2_run_screener(self) -> None:
        """執行本地資料庫的布林擠壓篩選。"""
        try:
            limit = float(self.t2_limit_entry.get().strip())
            days = int(self.t2_days_entry.get().strip())
            min_vol = int(self.t2_vol_entry.get().strip())
        except ValueError:
            messagebox.showwarning("警告", "請輸入有效的帶寬上限、持續天數與最低成交量！")
            return

        selected_types = [k for k, v in self.t2_type_vars.items() if v.get()]
        if not selected_types:
            messagebox.showwarning("警告", "請至少勾選一種證券類型！")
            return

        self._disable_all_buttons()
        self.progress_bar.start(10)
        self.status_var.set("正在執行本地選股計算...")

        def worker() -> None:
            try:
                with self.db_lock:
                    df_res = screen_bollinger_squeeze(
                        self.conn,
                        bandwidth_limit=limit,
                        consecutive_days=days,
                        instrument_types=selected_types,
                        min_volume=min_vol,
                    )
                self.task_queue.put(("SCREEN", "SUCCESS", df_res))
            except Exception as e:
                self.task_queue.put(("SCREEN", "ERROR", str(e)))

        threading.Thread(target=worker, daemon=True).start()

    def _t2_update_treeview(self, df_res: pd.DataFrame) -> None:
        """將篩選結果更新至 Treeview。"""
        for item in self.t2_tree.get_children():
            self.t2_tree.delete(item)

        if df_res.empty:
            messagebox.showinfo("提示", "本地資料庫中沒有符合篩選條件的股票。")
            self._t2_df_res = None
            return

        self._t2_df_res = df_res.copy()
        self.t2_sort_col = "latest_bandwidth"
        self.t2_sort_asc = True
        self._t2_populate_treeview()

    def _t2_populate_treeview(self) -> None:
        """依目前排序狀態填入 Treeview（不重新查詢）。"""
        if self._t2_df_res is None or self._t2_df_res.empty:
            return

        for item in self.t2_tree.get_children():
            self.t2_tree.delete(item)

        df = self._t2_df_res.copy()
        col = self.t2_sort_col or "latest_bandwidth"
        ascending = self.t2_sort_asc
        df = df.sort_values(col, ascending=ascending)

        for _, row in df.iterrows():
            self.t2_tree.insert(
                "",
                tk.END,
                values=(
                    row["symbol"],
                    row["name"] or "無數據",
                    f"{row['latest_close']:.2f}",
                    f"{int(row['latest_volume']):,}",
                    f"{row['max_bandwidth'] * 100:.2f}%",
                    f"{row['latest_bandwidth'] * 100:.2f}%",
                )
            )

        arrows = {col: " ▲" if ascending else " ▼"}
        display_names = {
            "symbol": "股票代號", "name": "股票名稱",
            "latest_close": "最新收盤價", "latest_volume": "最新成交量",
            "max_bandwidth": "N日內最大頻寬", "latest_bandwidth": "最新頻寬",
        }
        for cid, txt in display_names.items():
            self.t2_tree.heading(cid, text=txt + arrows.get(cid, ""))

    def _t2_sort_by_column(self, col_id: str) -> None:
        """點擊欄位標題：切換排序方向或切換欄位。"""
        if self._t2_df_res is None or self._t2_df_res.empty:
            return
        if self.t2_sort_col == col_id:
            self.t2_sort_asc = not self.t2_sort_asc
        else:
            self.t2_sort_col = col_id
            self.t2_sort_asc = True
        self._t2_populate_treeview()

    def _t2_on_row_select(self, event: Any) -> None:
        """點擊 Treeview 表格列時，連動繪製該股布林通道圖。"""
        selected_items = self.t2_tree.selection()
        if not selected_items:
            return

        item = selected_items[0]
        values = self.t2_tree.item(item, "values")
        symbol = values[0]
        name = values[1]

        with self.db_lock:
            df = pd.read_sql_query(
                """
                SELECT trade_date, open_price, high_price, low_price,
                       close_price, volume_shares, security_name
                FROM market_quote_core
                WHERE symbol_clean = ?
                ORDER BY trade_date ASC
                """,
                self.conn,
                params=(symbol,)
            )

        if df.empty or len(df) < 20:
            self._t2_draw_placeholder(f"[{symbol}] 資料不足 20 筆，無法畫圖。")
            return

        # 準備 mplfinance OHLC 格式
        mpf_df = df.rename(columns={
            "open_price": "Open",
            "high_price": "High",
            "low_price": "Low",
            "close_price": "Close",
            "volume_shares": "Volume",
        })
        mpf_df["Date"] = pd.to_datetime(df["trade_date"])
        mpf_df = mpf_df.set_index("Date")
        mpf_df.index.name = "Date"

        # 計算布林通道
        bb = ta.bbands(df["close_price"], length=20, std=2)
        if bb is None or bb.empty:
            self._t2_draw_placeholder("指標計算錯誤")
            return

        bbl = bb.iloc[:, 0].values
        bbm = bb.iloc[:, 1].values
        bbu = bb.iloc[:, 2].values

        # 用 addplot 疊加布林通道
        add_plots = [
            mpf.make_addplot(bbu, color="#319795", width=0.8),
            mpf.make_addplot(bbm, color="#dd6b20", width=0.8, linestyle="--"),
            mpf.make_addplot(bbl, color="#319795", width=0.8),
        ]

        title_name = f" {name}" if name and name != "無數據" else ""
        fig, axes = mpf.plot(
            mpf_df,
            type="candle",
            addplot=add_plots,
            volume=True,
            volume_alpha=0.5,
            ylabel="股價",
            ylabel_lower="成交量",
            xrotation=0,
            returnfig=True,
            figsize=(6.5, 6),
            style="charles",
            tight_layout=True,
            title=f"[{symbol}]{title_name} 布林通道走勢圖",
        )
        fig.subplots_adjust(right=0.88)

        # 取代原本的 figure/axes
        self.t2_fig.clf()
        self.t2_fig = fig
        self.t2_ax_price = axes[0]
        self.t2_ax_vol = axes[2]
        self.t2_canvas.figure = fig
        self.t2_canvas.draw()

    # ==================== 批次下載事件 ====================
    def _t2_batch_sync(self) -> None:
        """批次下載台灣 50 歷史數據。"""
        confirm = messagebox.askyesno(
            "確認",
            f"即將批次同步台灣 50 成份股 ({len(TAIWAN_50_SYMBOLS)} 檔) 近 3 個月歷史日K。\n預估耗時 30-60 秒，是否繼續？"
        )
        if not confirm:
            return

        self._disable_all_buttons()
        self.progress_bar.start(10)
        self.status_var.set("批次同步台灣50歷史數據中...")

        # 清空舊日誌
        self.t2_log_text.config(state=tk.NORMAL)
        self.t2_log_text.delete("1.0", tk.END)
        self.t2_log_text.config(state=tk.DISABLED)

        self._write_log("▶ 啟動台灣 50 批次歷史同步程序...\n")

        def worker() -> None:
            success = 0
            failed = 0
            total = len(TAIWAN_50_SYMBOLS)

            for idx, symbol in enumerate(TAIWAN_50_SYMBOLS):
                self.task_queue.put(("BATCH_PROGRESS", "INFO", f"[{idx+1}/{total}] 正在同步 {symbol}..."))

                # yfinance 大部分台灣股票代號是 .TW，少部分是 .TWO
                # 台灣50全部都是 TWSE（.TW）
                try:
                    with self.db_lock:
                        count = sync_yfinance_to_db(self.repo, symbol, "TWSE", period="3mo")

                    if count > 0:
                        success += 1
                        self.task_queue.put(("BATCH_PROGRESS", "INFO", f" ✓ {symbol} 同步完成：共 {count} 筆數據入庫"))
                    else:
                        # 嘗試 TPEX
                        with self.db_lock:
                            count = sync_yfinance_to_db(self.repo, symbol, "TPEX", period="3mo")
                        if count > 0:
                            success += 1
                            self.task_queue.put(("BATCH_PROGRESS", "INFO", f" ✓ {symbol} (TPEX) 同步完成：共 {count} 筆數據入庫"))
                        else:
                            failed += 1
                            self.task_queue.put(("BATCH_PROGRESS", "INFO", f" ✗ {symbol} 同步失敗：yfinance 未回傳數據"))
                except Exception as e:
                    failed += 1
                    self.task_queue.put(("BATCH_PROGRESS", "INFO", f" ✗ {symbol} 錯誤：{str(e)}"))

            self.task_queue.put(("BATCH_DONE", "SUCCESS", (success, failed)))

        threading.Thread(target=worker, daemon=True).start()

    # ==================== 下載管理事件 (TAB 3) ====================
    def _t3_start_download(self) -> None:
        """按下「開始下載」按鈕，啟動背景執行緒下載所有上市/上櫃股票。"""
        mode = self.t3_mode_combo.get()

        date_str: str | None = None
        start: str | None = None
        end: str | None = None
        use_holidays = self.t3_holiday_var.get()

        if mode == "指定日期":
            date_str = self.t3_date_entry.get().strip()
            if not date_str:
                messagebox.showwarning("警告", "請輸入日期 (YYYY-MM-DD)")
                return
        elif mode == "日期區間":
            start = self.t3_start_entry.get().strip()
            end = self.t3_end_entry.get().strip()
            if not start or not end:
                messagebox.showwarning("警告", "請輸入起始與結束日期")
                return

        self._disable_all_buttons()
        self.progress_bar.start(10)
        self.status_var.set("準備下載...")

        self.t3_log_text.config(state=tk.NORMAL)
        self.t3_log_text.delete("1.0", tk.END)
        self.t3_log_text.config(state=tk.DISABLED)

        self._t3_write_log(f"▶ 模式：{mode}")
        self._t3_write_log(f"  資料庫：{self.db_path}\n")

        threading.Thread(
            target=self._t3_download_worker,
            args=(mode, date_str, start, end, use_holidays),
            daemon=True,
        ).start()

    def _t3_write_log(self, message: str) -> None:
        """寫入 tab3 日誌框。"""
        self.t3_log_text.config(state=tk.NORMAL)
        self.t3_log_text.insert(tk.END, message + "\n")
        self.t3_log_text.see(tk.END)
        self.t3_log_text.config(state=tk.DISABLED)

    def _t3_download_worker(self, mode: str, date_str: str | None,
                            start: str | None, end: str | None, use_holidays: bool) -> None:
        """背景下載執行緒：複用 Pipeline 下載所有上市/上櫃股票。"""
        import datetime as _dt

        from market_crawler.pipeline import _fetch_twse_holidays, fetch_and_ingest_day

        policy = HttpPolicy(
            headers={"user-agent":
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                "AppleWebKit/537.36 (KHTML, like Gecko) "
                "Chrome/123.0.0.0 Safari/537.36"
            }
        )

        try:
            if mode == "下載今日":
                today = _dt.date.today().isoformat()
                self.task_queue.put(("BULK_PROGRESS", today, (1, 1)))
                self.task_queue.put(("BULK_LOG", f"  正在下載 {today}（TWSE + TPEX）..."))

                result = fetch_and_ingest_day(today, db_path=self.db_path, policy=policy)
                if result is True:
                    self.task_queue.put(("BULK_LOG", "  ✓ 下載成功"))
                    self.task_queue.put(("BULK_DONE", "success", {"success": 1, "market_closed": 0, "failed": 0}))
                elif result is None:
                    self.task_queue.put(("BULK_LOG", "  - 當日休市"))
                    self.task_queue.put(("BULK_DONE", "warning", {"success": 0, "market_closed": 1, "failed": 0}))
                else:
                    self.task_queue.put(("BULK_LOG", "  ✗ 下載失敗"))
                    self.task_queue.put(("BULK_DONE", "failed", {"success": 0, "market_closed": 0, "failed": 1}))

            elif mode == "指定日期":
                assert date_str is not None
                self.task_queue.put(("BULK_PROGRESS", date_str, (1, 1)))
                self.task_queue.put(("BULK_LOG", f"  正在下載 {date_str}（TWSE + TPEX）..."))

                result = fetch_and_ingest_day(date_str, db_path=self.db_path, policy=policy)
                if result is True:
                    self.task_queue.put(("BULK_LOG", "  ✓ 下載成功"))
                    self.task_queue.put(("BULK_DONE", "success", {"success": 1, "market_closed": 0, "failed": 0}))
                elif result is None:
                    self.task_queue.put(("BULK_LOG", "  - 當日休市"))
                    self.task_queue.put(("BULK_DONE", "warning", {"success": 0, "market_closed": 1, "failed": 0}))
                else:
                    self.task_queue.put(("BULK_LOG", "  ✗ 下載失敗"))
                    self.task_queue.put(("BULK_DONE", "failed", {"success": 0, "market_closed": 0, "failed": 1}))

            elif mode == "日期區間":
                assert start is not None and end is not None
                holidays: set[str] = set()
                if use_holidays:
                    try:
                        twse_h = _fetch_twse_holidays(policy)
                        holidays = {d.isoformat() for d in twse_h}
                        self.task_queue.put(("BULK_LOG", f"  自動假日偵測：{len(holidays)} 天假日"))
                    except Exception as e:
                        self.task_queue.put(("BULK_LOG", f"  ⚠ 假日 API 抓取失敗：{e}"))

                days = iter_date_strings(start, end)
                total = len(days)
                self.task_queue.put(("BULK_LOG", f"  日期區間：{start} ~ {end}（共 {total} 天）"))

                success = market_closed = failed = skipped = 0

                for i, day in enumerate(days):
                    day_type = classify_day(day, holidays)
                    if day_type in ("holiday", "weekend"):
                        skipped += 1
                        continue

                    self.task_queue.put(("BULK_PROGRESS", day, (i + 1, total)))
                    self.task_queue.put(("BULK_LOG", f"  [{i + 1 - skipped}/{total}] 處理 {day}..."))

                    try:
                        result = fetch_and_ingest_day(day, db_path=self.db_path, policy=policy)
                        if result is True:
                            success += 1
                            self.task_queue.put(("BULK_LOG", f"    ✓ 寫入完成"))
                        elif result is None:
                            market_closed += 1
                            self.task_queue.put(("BULK_LOG", f"    - 休市"))
                        else:
                            failed += 1
                            self.task_queue.put(("BULK_LOG", f"    ✗ 失敗"))
                    except Exception as e:
                        failed += 1
                        self.task_queue.put(("BULK_LOG", f"    ✗ 異常：{e}"))

                self.task_queue.put(("BULK_LOG", ""))
                self.task_queue.put(("BULK_DONE", "range_done", {
                    "success": success,
                    "market_closed": market_closed,
                    "failed": failed,
                    "skipped": skipped,
                }))

        except Exception as e:
            self.task_queue.put(("BULK_LOG", f"\n✗ 下載程序異常中斷：{e}"))
            self.task_queue.put(("BULK_DONE", "error", {"success": 0, "market_closed": 0, "failed": 0}))


def main() -> None:
    root = tk.Tk()
    _ = StockScreenerApp(root)
    root.mainloop()
    try:
        root.destroy()
    except Exception:
        pass


if __name__ == "__main__":
    main()
