## Why

目前的專案是 Python + tkinter + matplotlib 的桌面應用，雖然功能完整，但在跨平台打包（PyInstaller 約 150MB）、UI 互動性（靜態圖表、無縮放平移）、以及長期維護上越來越吃力。Tauri 2.0 + Rust 後端 + Vue 3 前端可以將安裝檔縮小到 ~10MB，圖表原生支援互動，並獲得真正的跨平台編譯支援。

## What Changes

- **新專案結構**：在同一個 repo 中建立 Tauri 2.0 專案，Python 程式碼保留作為參考與過渡
- **Rust 後端**：將所有 Python 模組（repository、pipeline、screener、classification、yfinance_sync、http_client、parsers、orchestrator）移植到 Rust，分為 `stock-core` 庫與 `stock-cli` CLI 兩個 crate
- **Vue 3 前端**：取代 tkinter 1407 行 GUI，三個分頁（單股分析、選股篩選、下載管理）以 Vue 3 元件重建
- **ECharts 取代 matplotlib/mplfinance**：K 線圖、布林通道、成交量以 ECharts 繪製，原生支援縮放、十字線讀值、繁體中文
- **AG Grid 取代 ttk.Treeview**：篩選結果表格以 AG Grid 渲染，內建排序、欄位 resize、虛擬滾動
- **所有技術指標在 Rust 計算**：SMA、EMA、布林通道、RSI、MACD 都在 Rust 端算好後傳遞純資料給前端，ECharts 只負責繪圖
- **Vue 可調整參數表單**：技術指標的週期（如 MA 天數、RSI 週期、MACD 參數）以 Vue 表單元件呈現，使用者可即時調整
- **SQLite Migration 機制**：以版本號管理 schema 升級，取代手動 DDL
- **Tauri 事件系統取代 queue.Queue**：長時間任務（批次下載）的進度回報以 Tauri event emit/listen 實作
- **即時股價輪詢**：盤中每 10 秒從 TWSE 基本市況報導 API 抓取即時成交價、量、五檔，K 線今日 bar 隨即時價格跳動
- **跨平台打包**：Windows (.msi)、macOS (.dmg)、Linux (.deb/.AppImage)

## Capabilities

### New Capabilities

- `tauri-desktop-shell`: Tauri 2.0 桌面視窗管理，包含視窗選單、系統 tray、跨平台打包
- `rust-core-library`: stock-core Rust 庫，涵蓋 SQLite CRUD、HTTP 客戶端、HTML 解析、技術指標計算
- `vue-desktop-frontend`: Vue 3 前端應用，以 Pinia 管理狀態，Vite 建置
- `echarts-charting`: 以 ECharts 繪製 K 線圖、布林通道、RSI、MACD 等技術圖表，支援互動縮放
- `ag-grid-table`: 以 AG Grid 呈現篩選結果表格，支援排序、欄位調整、行點擊連動圖表
- `rsi-indicator`: RSI 技術指標計算（Rust）與圖表渲染（ECharts 副圖）
- `macd-indicator`: MACD 技術指標計算（Rust）與圖表渲染（ECharts 副圖）
- `adjustable-params-form`: Vue 可調整技術指標參數表單（MA 天數、RSI 週期、MACD 參數等）
- `twse-big5-scraper`: 以 reqwest + scraper crate 處理 TWSE 老系統 Big5 編碼回應
- `sqlite-migration`: 以版本號管理 SQLite schema 升級
- `tauri-event-progress`: 以 Tauri 事件系統實作長時間任務進度回報
- `cli-entrypoint-rust`: 以 clap 重製 CLI 指令，與 stock-core 共用邏輯
- `twse-realtime-quote`: 盤中每 10 秒從 TWSE 基本市況報導 API 輪詢即時股價，K 線今日 bar 隨之跳動

### Modified Capabilities

- 無（全新實作，向下相容既有 SQLite 資料庫格式）

## Impact

- **新目錄結構**：`stock-core/`、`stock-cli/`、`stock-tauri/` 三個 crate 加 `stock-tauri/src/` Vue 前端
- **檔案異動**：Python 程式碼移至 `legacy/` 目錄；無既有檔案被刪除或修改
- **相依套件**：新增 Rust crate（rusqlite、reqwest、scraper、chrono、serde、clap、tauri、wiremock dev-dep）、npm package（vue、echarts、ag-grid-vue3、pinia、vue-sonner、@tauri-apps/api）
- **資料庫相容**：既有 `market_data.db` 沿用，schema 維持不變
