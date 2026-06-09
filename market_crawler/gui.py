from __future__ import annotations

import platform
import queue
import sqlite3
import threading
import tkinter as tk
from tkinter import messagebox, ttk
from typing import Any

import matplotlib
import matplotlib.pyplot as plt
import pandas as pd
import pandas_ta as ta
from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg

from market_crawler.repository import SQLiteMarketRepository
from market_crawler.screener import screen_bollinger_squeeze
from market_crawler.yfinance_sync import sync_yfinance_to_db

# 設定 Matplotlib 支援 Tkinter 渲染並套用美觀樣式
matplotlib.use("TkAgg")
plt.style.use("ggplot")  # 使用 ggplot 樣式讓圖表更美觀

# 解決跨平台中文字型顯示問題，防止 Linux (DejaVu Sans) 出現 Glyph missing 警告與亂碼
_system = platform.system()
if _system == "Darwin":  # macOS
    matplotlib.rc("font", family="Arial Unicode MS")
elif _system == "Windows":
    matplotlib.rc("font", family="Microsoft JhengHei")
else:  # Linux / Unix
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
        self.conn = sqlite3.connect(self.db_path, check_same_thread=False)
        self.repo = SQLiteMarketRepository(self.conn)
        self.repo.init_schema()

        # 線程安全佇列與鎖
        self.task_queue: queue.Queue[tuple[str, Any]] = queue.Queue()
        self.db_lock = threading.Lock()

        # 建立 UI 介面
        self._setup_styles()
        self._create_widgets()

        # 啟動佇列輪詢監聽
        self.root.after(100, self._process_queue)

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
        self.status_label = ttk.Label(title_frame, textvariable=self.status_var, font=("Microsoft JhengHei", 10, "italic"), foreground="#4a5568")
        self.status_label.pack(side=tk.RIGHT, padx=10)

        self.progress_bar = ttk.Progressbar(main_container, orient=tk.HORIZONTAL, mode="indeterminate")
        self.progress_bar.pack(fill=tk.X, pady=(0, 10))

        # 分頁控制項
        self.notebook = ttk.Notebook(main_container)
        self.notebook.pack(fill=tk.BOTH, expand=True)

        # 建立兩個分頁
        self.tab1 = ttk.Frame(self.notebook, padding=10)
        self.tab2 = ttk.Frame(self.notebook, padding=10)

        self.notebook.add(self.tab1, text="📊 單股分析與視覺化")
        self.notebook.add(self.tab2, text="🔍 布林擠壓選股器")

        self._build_tab1()
        self._build_tab2()

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
        self.t1_symbol_entry = ttk.Entry(ctrl_frame, width=15)
        self.t1_symbol_entry.insert(0, "2330")
        self.t1_symbol_entry.grid(row=0, column=1, sticky=tk.W, pady=5, padx=5)

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

        # 控制按鈕
        self.t2_screen_btn = ttk.Button(ctrl_frame, text="🔍 開始本地篩選", style="Primary.TButton", command=self._t2_run_screener)
        self.t2_screen_btn.grid(row=1, column=0, columnspan=2, pady=10, sticky=tk.EW)

        self.t2_batch_btn = ttk.Button(ctrl_frame, text="☁️ 批次同步台灣50歷史日K", style="Action.TButton", command=self._t2_batch_sync)
        self.t2_batch_btn.grid(row=1, column=2, columnspan=2, pady=10, sticky=tk.EW, padx=(10, 0))

        # 表格顯示區
        table_frame = ttk.LabelFrame(left_frame, text=" 篩選符合條件股票清單 (點選可即時更新右側圖表) ", padding=5)
        table_frame.pack(fill=tk.BOTH, expand=True)

        columns = ("symbol", "name", "latest_close", "latest_volume", "max_bandwidth", "latest_bandwidth")
        self.t2_tree = ttk.Treeview(table_frame, columns=columns, show="headings", selectmode="browse")

        self.t2_tree.heading("symbol", text="股票代號")
        self.t2_tree.heading("name", text="股票名稱")
        self.t2_tree.heading("latest_close", text="最新收盤價")
        self.t2_tree.heading("latest_volume", text="最新成交量")
        self.t2_tree.heading("max_bandwidth", text="N日內最大頻寬")
        self.t2_tree.heading("latest_bandwidth", text="最新頻寬")

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

    def _write_log(self, message: str) -> None:
        """向日誌視窗寫入文字。"""
        self.t2_log_text.config(state=tk.NORMAL)
        self.t2_log_text.insert(tk.END, message + "\n")
        self.t2_log_text.see(tk.END)
        self.t2_log_text.config(state=tk.DISABLED)

    # ==================== 異步與多線程處理邏輯 ====================
    def _process_queue(self) -> None:
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
                        messagebox.showinfo("成功", f"歷史數據同步完成！共匯入 {count} 筆記錄。")
                        self._t1_plot_data()  # 同步完自動畫圖
                    else:
                        self.status_var.set("下載失敗")
                        messagebox.showerror("錯誤", f"同步失敗：\n{data}")

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
                    messagebox.showinfo("完成", f"台灣 50 批次同步結束！\n成功：{success_count} 檔\n失敗：{failed_count} 檔")

                self.task_queue.task_done()
        except queue.Empty:
            pass
        finally:
            self.root.after(100, self._process_queue)

    def _disable_all_buttons(self) -> None:
        """執行耗時任務時禁用所有按鈕，防重複點擊。"""
        self.t1_sync_btn.config(state=tk.DISABLED)
        self.t1_plot_btn.config(state=tk.DISABLED)
        self.t2_screen_btn.config(state=tk.DISABLED)
        self.t2_batch_btn.config(state=tk.DISABLED)

    def _enable_all_buttons(self) -> None:
        """任務結束後啟用所有按鈕。"""
        self.t1_sync_btn.config(state=tk.NORMAL)
        self.t1_plot_btn.config(state=tk.NORMAL)
        self.t2_screen_btn.config(state=tk.NORMAL)
        self.t2_batch_btn.config(state=tk.NORMAL)

    # ==================== 單股分析事件 ====================
    def _t1_sync_data(self) -> None:
        """按下單股同步按鈕。"""
        symbol = self.t1_symbol_entry.get().strip()
        market = self.t1_market_combo.get()
        period = self.t1_period_combo.get()

        if not symbol:
            messagebox.showwarning("警告", "請先輸入股票代號！")
            return

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
        symbol = self.t1_symbol_entry.get().strip()
        if not symbol:
            messagebox.showwarning("警告", "請先輸入股票代號！")
            return

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
            self._t1_draw_placeholder(f"本地資料庫中無 [{symbol}] 之數據，請先點擊同步。")
            self._update_t1_info_text(None)
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
        info.append(f"代號: {self.t1_symbol_entry.get().strip()}")
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
        except ValueError:
            messagebox.showwarning("警告", "請輸入有效的帶寬上限與持續天數！")
            return

        self._disable_all_buttons()
        self.progress_bar.start(10)
        self.status_var.set("正在執行本地選股計算...")

        def worker() -> None:
            try:
                with self.db_lock:
                    # 呼叫 screener 核心函數
                    df_res = screen_bollinger_squeeze(self.conn, bandwidth_limit=limit, consecutive_days=days)
                self.task_queue.put(("SCREEN", "SUCCESS", df_res))
            except Exception as e:
                self.task_queue.put(("SCREEN", "ERROR", str(e)))

        threading.Thread(target=worker, daemon=True).start()

    def _t2_update_treeview(self, df_res: pd.DataFrame) -> None:
        """將篩選結果更新至 Treeview。"""
        # 清空舊資料
        for item in self.t2_tree.get_children():
            self.t2_tree.delete(item)

        if df_res.empty:
            messagebox.showinfo("提示", "本地資料庫中沒有符合篩選條件的股票。")
            return

        # 排序：優先展示頻寬最小（擠壓最劇烈）的股票
        df_sorted = df_res.sort_values("latest_bandwidth")

        for _, row in df_sorted.iterrows():
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
                SELECT trade_date, close_price, volume_shares, security_name
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

        # 計算指標
        bb = ta.bbands(df["close_price"], length=20, std=2)
        if bb is None or bb.empty:
            self._t2_draw_placeholder("指標計算錯誤")
            return

        bbl = bb.iloc[:, 0]
        bbm = bb.iloc[:, 1]
        bbu = bb.iloc[:, 2]

        dates = pd.to_datetime(df["trade_date"])

        # 繪圖
        self.t2_ax_price.clear()
        self.t2_ax_vol.clear()

        self.t2_ax_price.plot(dates, df["close_price"], label="收盤價", color="#1a365d", linewidth=1.5)
        self.t2_ax_price.plot(dates, bbm, label="20 MA", color="#dd6b20", linestyle="--", linewidth=1.0)
        self.t2_ax_price.plot(dates, bbu, label="上軌", color="#319795", linewidth=1.0, alpha=0.8)
        self.t2_ax_price.plot(dates, bbl, label="下軌", color="#319795", linewidth=1.0, alpha=0.8)
        self.t2_ax_price.fill_between(dates, bbl, bbu, facecolor="#ebf8ff", alpha=0.6, label="布林區間")

        title_name = f" {name}" if name and name != "無數據" else ""
        self.t2_ax_price.set_title(f"[{symbol}]{title_name} 布林通道走勢圖 (與表格同步)", fontsize=11, fontweight="bold", color="#1a365d")
        self.t2_ax_price.set_ylabel("股價")
        self.t2_ax_price.legend(loc="upper left", framealpha=0.9, facecolor="white", fontsize=8)

        self.t2_ax_vol.bar(dates, df["volume_shares"], color="#a0aec0", alpha=0.7, width=0.6)
        self.t2_ax_vol.set_ylabel("成交股數")
        self.t2_ax_vol.set_xlabel("交易日期")

        self.t2_fig.autofmt_xdate()
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


def main() -> None:
    root = tk.Tk()
    _ = StockScreenerApp(root)
    root.mainloop()


if __name__ == "__main__":
    main()
