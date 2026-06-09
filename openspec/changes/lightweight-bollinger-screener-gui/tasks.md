## 1. 環境設定與套件管理 (Setup & Dependencies)

- [x] 1.1 於 `pyproject.toml` 中的 `project.optional-dependencies` 新增 `viz = ["matplotlib", "pandas-ta", "yfinance"]`，並同步安裝。
- [x] 1.2 於 `Makefile` 中新增 `make gui` 啟動指令，並於 `make install` 流程中支援可選安裝。

## 2. yfinance 歷史數據補檔核心 (yfinance Sync Engine)

- [x] 2.1 建立 `market_crawler/yfinance_sync.py` 模組，實作將台股代號轉換為 Yahoo 格式並呼叫 `yf.download` 取得歷史日 K 資料。
- [x] 2.2 將抓取結果轉換為標準 `CanonicalQuote` 結構，呼叫 `SQLiteMarketRepository.upsert_core_batch` 寫入本地資料庫。
- [x] 2.3 於 `tests/` 新增 `test_yfinance_sync.py` 單元測試，驗證格式解析與資料入庫。

## 3. 布林通道選股引擎 (Bollinger Screener Core)

- [x] 3.1 於 `market_crawler/` 新增 `screener.py`，實作單一 SQL 查詢撈取所有股票近 $M+30$ 天歷史股價。
- [x] 3.2 實作多股分組 (groupby) 運算，利用 `pandas-ta` 計算布林頻寬（Bandwidth）。
- [x] 3.3 實作過濾演算法，篩選出最近連續 $N$ 日之頻寬最大值低於指定門檻值的股票。
- [x] 3.4 新增 `tests/test_screener.py` 驗證選股核心演算法與極端值處理。
- [x] 3.5 於 `screener.py` 選股結果中，支援撈取並回傳股票名稱。

## 4. GUI 介面與圖表視覺化 (GUI & Matplotlib Visualization)

- [x] 4.1 建立 `market_crawler/gui.py` 主程式，使用 Tkinter 實作雙分頁主視窗。
- [x] 4.2 實作後台背景執行緒 Worker (利用 `threading.Thread` 與 `queue.Queue`），在下載歷史資料與選股時避免介面卡死。
- [x] 4.3 於「單股分析」分頁，使用 `FigureCanvasTkAgg` 繪製主圖（股價、20 MA 與布林通道半透明陰影填滿區間）以及副圖（成交量）。
- [x] 4.4 於「布林擠壓篩選器」分頁，實作 Treeview 表格展示選股結果，並串接 Click 事件，實現點選表格即時更新右側圖表功能。
- [x] 4.5 於 `gui.py` 雙分頁與圖表標題中，完整展示股票名稱，並於選股表格新增名稱欄位。
- [x] 4.6 將單股分析的股票代號 Entry 改為 Combobox，支援 "代號 - 名稱" 下拉選單與手動輸入。
- [x] 4.7 實作「繪圖時若本地無資料則自動下載」以及「下載後自動重新整理選單 values」的智慧連動。
