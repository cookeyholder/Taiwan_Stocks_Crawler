## 1. 專案 Scaffolding

- [x] 1.1 在 repo 根目錄初始化 Cargo workspace，納入 stock-core、stock-cli、stock-tauri
- [x] 1.2 定義 stock-core 的 Cargo.toml 與相依套件（serde、chrono、rusqlite、reqrewst、scraper、encoding_rs、thiserror、log、tokio）
- [x] 1.3 定義 stock-cli 的 Cargo.toml 與 clap 相依
- [x] 1.4 以手動方式初始化 stock-tauri（因 create-tauri-app 需要互動終端機）：Vue 3 + TypeScript 模板 + Vite + Pinia
- [x] 1.5 設定 tauri.conf.json：視窗標題、尺寸 1200x800、最小尺寸 900x600、app identifier
- [x] 1.6 設定 stock-tauri/Cargo.toml 加入 tauri 2.x、serde、tokio 相依及 stock-core path 依賴
- [x] 1.7 在 repo 根目錄建立 Rust workspace Cargo.toml，連結三個 crate
- [ ] 1.8 驗證 `cargo build` 在 workspace 層級成功

## 2. Stock-Core：資料模型與錯誤型別

- [ ] 2.1 定義錯誤列舉 AppError，包含 Db、Http、Parse、TwseApi、NotImplemented 變體
- [ ] 2.2 為 AppError 實作 From<rusqlite::Error> 與 From<reqwest::Error>
- [ ] 2.3 定義 OHLCV struct：date、open、high、low、close、volume
- [ ] 2.4 定義 MarketQuote struct：symbol、name、trade_date、open、high、low、close、volume、instrument_type
- [ ] 2.5 定義 ScreenerResult struct：symbol、name、latest_close、latest_volume、max_bandwidth、latest_bandwidth
- [ ] 2.6 定義 ChartResponse struct：dates、ohlc、volumes、sma_data vec、bbu/bbm/bbl、rsi、macd_line/signal/histogram，全部可序列化

## 3. Stock-Core：SQLite 資料庫層

- [ ] 3.1 實作 ConnectionManager：開啟/關閉資料庫、PRAGMA 設定（journal_mode、foreign_keys）
- [ ] 3.2 實作 migration runner：讀取 PRAGMA user_version，依序執行未套用的 migration
- [ ] 3.3 撰寫 migration 001：CREATE TABLE market_quote_core，包含與既有 schema 相符的所有欄位
- [ ] 3.4 撰寫 migration 002：在 symbol_clean 與 trade_date 上建立索引
- [ ] 3.5 實作 Repository::query_ohlcv(symbol, start_date, end_date) -> Vec<OHLCV>
- [ ] 3.6 實作 Repository::upsert_quote(quote) — 單列新增或更新
- [ ] 3.7 實作 Repository::upsert_quotes_batch(quotes) — 批次 upsert
- [ ] 3.8 實作 Repository::query_screener(params) -> Vec<ScreenerResult>
- [ ] 3.9 實作 Repository::get_distinct_symbols() -> Vec<String>
- [ ] 3.10 實作 Repository::update_instrument_type(symbol, instrument_type)
- [ ] 3.11 為 Repository 撰寫單元測試：CRUD、查詢、upsert 邊界案例

## 4. Stock-Core：技術指標分析模組

- [ ] 4.1 實作 sma(data, period) -> Vec<Option<f64>>，使用 NaN 安全累加
- [ ] 4.2 實作 ema(data, period) -> Vec<Option<f64>>，以 SMA 初始化搭配遞迴公式
- [ ] 4.3 實作 bollinger(data, period, num_std) -> BollingerResult，加入數值穩定性防護
- [ ] 4.4 實作 rsi(data, period) -> Vec<Option<f64>>，使用 Wilder's smoothing
- [ ] 4.5 實作 macd(data, fast, slow, signal) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>)
- [ ] 4.6 實作 AnalysisEngine::compute_all(ohlcv, params) -> ChartResponse，整合所有指標
- [ ] 4.7 為所有指標撰寫單元測試：已知數值驗證、邊界案例（平滑資料、遺失資料）

## 5. Stock-Core：HTTP 客戶端與爬蟲

- [ ] 5.1 以 reqwest ClientBuilder 實作 HttpClient，含 User-Agent 輪換與可設定逾時
- [ ] 5.2 實作 5xx/逾時重試邏輯，使用 exponential backoff（最多 3 次）
- [ ] 5.3 實作 TWSE 爬蟲：每日行情 JSON API，透過 encoding_rs 處理 Big5 編碼
- [ ] 5.4 實作 TWSE 爬蟲：歷史 CSV API（含日期參數）
- [ ] 5.5 實作 TWSE 爬蟲：休市日 API
- [ ] 5.6 實作 TPEX 爬蟲：每日行情 API
- [ ] 5.7 實作 Yahoo Finance 爬蟲：依股票代號+日期區間下載歷史 OHLCV
- [ ] 5.8 以 scraper crate 實作 HTML 表格解析器，處理 TWSE 回應頁面
- [ ] 5.9 以 mock HTTP 回應為爬蟲撰寫單元測試
- [ ] 5.10 實作 TWSE 即時報價爬蟲：mis.twse.com.tw API，處理未成交與盤中斷線情境

## 6. Stock-Core：排程與編排

- [ ] 6.1 實作 classify_day(date) -> DayType（trading、holiday、weekend、market_closed）
- [ ] 6.2 實作 iter_date_strings(start, end) -> Vec<String>
- [ ] 6.3 實作從 TWSE API 自動抓取休市日
- [ ] 6.4 實作 fetch_and_ingest_day(fetcher, date) -> IngestResult（筆數/錯誤）
- [ ] 6.5 實作 batch_download(dates, progress_callback) -> BatchResult
- [ ] 6.6 實作 reclassify_database(conn) 進行證券類型分類
- [ ] 6.7 實作 sync_yfinance_to_db(symbols) 進行歷史資料同步

## 7. Stock-CLI

- [ ] 7.1 以 clap derive 定義子指令結構：fetch、screen、sync、classify、backfill
- [ ] 7.2 實作 `fetch --today`、`fetch --date YYYY-MM-DD`、`fetch --start --end`
- [ ] 7.3 實作 `screen --max-bandwidth --min-days --min-volume --type`，輸出表格
- [ ] 7.4 實作 `sync --symbol` 進行 yfinance 同步
- [ ] 7.5 實作 `classify` 進行資料庫重新分類
- [ ] 7.6 驗證 `cargo run -- --help` 及各子指令正確運作

## 8. Stock-Tauri：Rust 指令層

- [ ] 8.1 定義 AppState（Arc<Mutex<Connection>>）並設定 Tauri 指令
- [ ] 8.2 實作 `get_ohlcv_with_indicators` 指令：讀 DB + 計算指標 → ChartResponse
- [ ] 8.3 實作 `screen_bollinger_squeeze` 指令：代理至 stock-core Repository
- [ ] 8.4 實作 `fetch_and_ingest` 指令：下載 + 寫入，含事件進度
- [ ] 8.5 實作 `fetch_realtime_quote` 指令：呼叫 TWSE mis API，回傳 RealTimeQuote
- [ ] 8.6 實作 `batch_download` 指令：多日期迴圈，含進度事件
- [ ] 8.7 實作 `sync_yfinance` 指令：單一股票代號同步
- [ ] 8.8 實作 `reclassify_database` 指令：背景重新分類
- [ ] 8.9 在 main.rs 中以 .invoke_handler() 註冊所有指令

## 9. Stock-Tauri：Vue 3 前端 — 專案設定

- [ ] 9.1 在 stock-tauri/src 設定 Vite + Vue 3 + TypeScript + pinia
- [ ] 9.2 設定 Vue Router 或條件渲染實現三個頁簽
- [ ] 9.3 加入 npm 相依：echarts、ag-grid-vue3（社群版）、@tauri-apps/api、pinia、vue-echarts
- [ ] 9.4 建立 Pinia store 模組：analysis.ts、screener.ts、download.ts
- [ ] 9.5 設定 AG Grid 主題（alpine）與 ECharts 基礎設定
- [ ] 9.6 驗證 `npm run dev` 可正常啟動

## 10. Stock-Tauri：Vue 3 前端 — 頁簽 1：單股分析

- [ ] 10.1 建立 TabAnalysis.vue 佈局：股票代號輸入、參數面板、圖表區
- [ ] 10.2 建立股票代號自動完成/搜尋輸入元件
- [ ] 10.3 建立 StockChart.vue 元件，封裝 vue-echarts 顯示 K 線 + 布林通道 + 成交量
- [ ] 10.4 將 invoke('get_ohlcv_with_indicators') 與股票代號變更連動
- [ ] 10.5 實作參數變更的防抖重新擷取（300ms）
- [ ] 10.6 加入資訊顯示面板：最新價格、漲跌幅、頻寬值

## 11. Stock-Tauri：Vue 3 前端 — 頁簽 2：布林擠壓選股器

- [ ] 11.1 建立 TabScreener.vue 佈局：篩選面板 + AG Grid + ECharts 預覽
- [ ] 11.2 以 AG Grid 建立 ResultsTable.vue：6 欄位、可排序、數值格式化
- [ ] 11.3 實作 invoke('screen_bollinger_squeeze') 與篩選參數連動
- [ ] 11.4 將列點擊事件與 StockChart 元件連動，顯示選取股票的圖表
- [ ] 11.5 加入台灣 50 成份股批次同步按鈕

## 12. Stock-Tauri：Vue 3 前端 — 頁簽 3：下載管理

- [ ] 12.1 建立 TabDownload.vue 佈局：日期選擇器、模式選項（今日/指定日期/區間）、日誌區
- [ ] 12.2 實作 listen('download-progress') 更新進度條與狀態文字
- [ ] 12.3 加入開始/停止下載按鈕
- [ ] 12.4 建立可滾動的日誌輸出區，顯示每日結果

## 13. Stock-Tauri：前端 — 可調參數表單元件

- [ ] 13.1 建立 ParamsPanel.vue，包含 MA 週期、布林通道、RSI、MACD 與顯示切換等 fieldset 區塊
- [ ] 13.2 將參數連接至 Pinia store 並觸發防抖重新擷取
- [ ] 13.3 實作成交量、布林通道、RSI、MACD 副圖的顯示/隱藏切換
- [ ] 13.4 將上次使用的參數存入 localStorage，達到跨工作階段保存

## 14. Stock-Tauri：前端 — ECharts 進階圖表

- [ ] 14.1 設定多圖格 ECharts 佈局：價格（60%）、成交量（15%）、RSI（12%）、MACD（13%）
- [ ] 14.2 在所有圖格間實作共用 dataZoom，達到同步縮放/平移
- [ ] 14.3 設定十字線 tooltip，顯示所有 series 數值
- [ ] 14.4 在 RSI 圖格加入超買/超賣（70/30）水平參考線
- [ ] 14.5 依開收盤方向將成交量柱著色為綠/紅
- [ ] 14.6 實作圖例點擊切換個別 series 可視性

## 15. Stock-Tauri：即時股價輪詢前端

- [ ] 15.1 建立 useRealtimePolling composable：封裝 invoke + 10s interval + 倒數計數 + 狀態機（idle/polling/backoff/nontrading）
- [ ] 15.2 實作交易時段偵測（週一～週五 09:00~13:30，UTC+8）
- [ ] 15.3 建立 ClockWidget.vue：時分秒雙位數字顯示，每秒 tick
- [ ] 15.4 建立 CountdownTimer.vue：顯示下次更新倒數（10→0）
- [ ] 15.5 建立 RealtimePanel.vue：即時報價面板（價/量/漲跌/漲幅/今高/今低/買賣五檔）
- [ ] 15.6 實作 updateTodayBar()：每次輪詢成功後修改 chartStore 今日最後一筆 OHLC
- [ ] 15.7 實作 ECharts animationDurationUpdate = 100、dataZoom 自動追尾/保持縮放狀態機
- [ ] 15.8 實作收盤後 EOD_SYNC：自動偵測 13:30，每 60s 嘗試 TWSE EOD API 補 DB
- [ ] 15.9 建立手動「下載今日資料」按鈕，已有資料時提示「今日資料已存在」
- [ ] 15.10 非交易時段顯示「非交易時間」文字，停止輪詢

## 16. Stock-Tauri：跨平台建置與驗證

- [ ] 16.1 設定 Tauri bundler 的 Windows (.msi) 組態，使用 NSIS installer
- [ ] 16.2 設定 Tauri bundler 的 macOS (.dmg) 組態，加入 code signing placeholder
- [ ] 16.3 設定 Tauri bundler 的 Linux (.deb + .AppImage) 組態
- [ ] 16.4 建立 GitHub Actions CI 工作流程，編譯 Rust + 前端 + Tauri
- [ ] 16.5 在開發機上驗證 `cargo tauri build` 成功
- [ ] 16.6 執行完整測試套件：`cargo test` + `npm run test`
- [ ] 16.7 驗證應用程式可開啟既有 market_data.db 並顯示真實股票資料
