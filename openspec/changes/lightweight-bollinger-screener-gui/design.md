## Context

目前專案具有健全的台股收盤行情爬蟲核心與 SQLite 資料庫。使用者希望在此基礎上擴充「技術指標數據視覺化」與「布林通道擠壓（Bollinger Band Squeeze）選股篩選器」。由於體積小、依賴少是核心設計訴求，因此我們必須避免引入像 PySide6/PyQt 等大型 GUI 框架，並將 yfinance 定位為快速補齊本地歷史資料的補充來源，核心篩選則完全在本地 SQLite 上進行。

## Goals / Non-Goals

**Goals:**
- **極簡依賴**：使用 Python 內建的 `tkinter` 結合 `matplotlib`，在滿足數據視覺化與 GUI 控制的同時，將新增的套件體積縮減 90% 以上。
- **本地高效能選股**：設計高效率 SQL 語句配合 Pandas 的分組 (groupby) 運算，在 1 秒內完成所有股票的 20 日布林通道帶寬計算。
- **yfinance 本地補檔器**：實作一鍵透過 yfinance 下載指定股票清單（如台灣50）過去 30~60 天的歷史報價，並 Upsert 寫入本地 SQLite，複用既有的 `SQLiteMarketRepository` 批次寫入邏輯。
- **異步處理**：使用背景執行緒處理爬蟲同步與選股，防止 Tkinter 介面因 I/O 阻塞而凍結卡死。

**Non-Goals:**
- 不修改現有 SQLite Schema。
- 不變更原先 `twse-crawler` CLI 的執行邏輯與分類路由。

## Decisions

### 1. 技術棧選擇：Tkinter + Matplotlib (FigureCanvasTkAgg)
- **決策**：我們決定不使用 `PySide6`，改用 `tkinter` 搭配 `matplotlib`。
- **理由**：Tkinter 為標準庫無須安裝，Matplotlib 體積約 10~15MB，相較於 Qt 的 100MB+ 能極大程度滿足「依賴少、體積小」的要求。且 Matplotlib 的 `FigureCanvasTkAgg` 支援無縫嵌入 Tkinter 視窗。

### 2. 異步 UI 渲染與背景工作執行緒
- **決策**：使用 Python 的 `threading.Thread` 與 `queue.Queue` 來實作 UI 與後台任務的分離。
- **理由**：Tkinter 的 `root.mainloop()` 執行在主執行緒，任何 I/O（如 yfinance 資料下載或大量資料庫運算）都會導致主執行緒阻塞。我們將下載與選股放到背景執行緒，並透過線程安全的 `Queue` 傳送狀態，主執行緒使用 `root.after()` 定期檢查 Queue 來更新進度條與日誌。

### 3. 選股運算流程設計
- **決策**：
  1. 一次性自資料庫讀取符合時間區間（例如最近 $M+30$ 天）的報價資料：
     `SELECT symbol_clean, trade_date, close_price FROM market_quote_core WHERE trade_date >= :start_date ORDER BY symbol_clean, trade_date ASC`
  2. 使用 Pandas 將資料按 `symbol_clean` 分組 (groupby)，呼叫 `pandas_ta.bbands` 計算 20 日布林通道。
  3. 計算帶寬：$\text{Bandwidth} = (\text{BBU} - \text{BBL}) / \text{BBM}$。
  4. 篩選最近 $N$ 日之帶寬最大值皆低於門檻值（例如 0.08）的股票，並將其結果回傳至 Tkinter 表格（Treeview）。

### 4. yfinance 本地同步設計
- **決策**：在 `market_crawler/yfinance_sync.py` 中，將台股代號轉為 Yahoo 格式（如 `2330` ➔ `2330.TW`），批次呼叫 `yf.download` 抓取歷史日 K，將欄位轉為 `CanonicalQuote` 格式後，調用 `SQLiteMarketRepository.upsert_core_batch` 寫入本地資料庫。

## Risks / Trade-offs

- **[Risk] yfinance 的 Rate Limit 與相容性問題**
  - *Mitigation*：`yfinance` 僅做為「快速初始化本地歷史資料」的補充管道，每日精準報價仍優先推薦使用原有的官方爬蟲核心。且下載時採分批（batch）限速處理，以防 IP 被 Yahoo 封鎖。
- **[Risk] Tkinter 介面樣式較為傳統**
  - *Mitigation*：使用簡潔明瞭的版面配置，並利用 Matplotlib 畫出高質感的布林通道圖（上軌、下軌使用 fill_between 填充淡藍色半透明區間），以優質的數據視覺化來提升使用者體驗。
