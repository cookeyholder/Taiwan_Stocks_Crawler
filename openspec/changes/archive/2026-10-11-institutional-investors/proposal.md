## Why

三大法人（外資及陸資、投信、自營商）為台股市場之主力資金風向球。投資人在技術分析與籌碼面判斷時，極度依賴法人買賣超動向以驗證趨勢突破或背離。目前系統僅儲存與展示量價 OHLCV 資料，缺少三大法人之爬取、資料庫持久化、歷史比對與圖表副圖呈現能力。

## What Changes

- **資料庫遷移 (Migration V7)**：
  - 新增 `institutional_investors` 資料表，記錄每日各檔個股之外資、投信、自營商買進、賣出、買賣超股數與三大法人合計買賣超。
  - 建立 `(symbol_clean, trade_date)` 索引，支援高速時序查詢。
- **爬蟲與資料管線 (`stock-core::fetcher`, `stock-core::pipeline`)**：
  - 於 `fetcher::twse` 實作 TWSE `T86`（三大法人買賣超日報）解析器。
  - 於 `MarketRepository` 新增三大法人買賣超批次寫入與查詢介面 (`upsert_institutional_batch`, `get_institutional_records`)。
  - 於 `Pipeline::fetch_date` 與定向修補流程中整合法人資料抓取與入庫。
- **狀態層與圖表資料 (`stock-gpui::state`)**：
  - 於 `InstitutionalRecord` 模型化法人買賣超數據。
  - 擴充 `AnalysisData` 包含對應交易日之三大法人資料。
  - 在 `CoreState::get_analysis` 關聯並對齊 OHLCV 與法人數據。
- **分析介面與副圖繪製 (`stock-gpui::views::analysis`)**：
  - 於工具列新增「三大法人」切換控制項 (`show_institutional`)。
  - 於副圖區塊新增三大法人買賣超柱狀圖（紅漲綠跌 / 買超紅色、賣超綠色）。
  - 於頂部十字游標即時資訊列（HUD）連動展示外資、投信、自營與合計買賣超張數。

## Capabilities

### New Capabilities

- `institutional-investors`: 支援台股三大法人（外資、投信、自營商）買賣超之每日資料爬取、持久化儲存、查詢，以及於 GPUI 走勢分析介面繪製副圖與十字游標即時數值連動。

## Impact

- **核心模組**：
  - `stock-core/src/db/migrations.rs`：新增 Migration V7。
  - `stock-core/src/db/mod.rs`：新增法人資料 CRUD 與查詢。
  - `stock-core/src/fetcher/twse.rs`：新增三大法人報表爬蟲與解析。
  - `stock-core/src/models.rs`：新增 `InstitutionalRecord`。
  - `stock-core/src/pipeline/mod.rs`：整合至每日管線。
- **桌面應用**：
  - `stock-gpui/src/state.rs`：擴充 `AnalysisData`。
  - `stock-gpui/src/views/analysis.rs`：繪製法人副圖與頂部 HUD 數值顯示。
