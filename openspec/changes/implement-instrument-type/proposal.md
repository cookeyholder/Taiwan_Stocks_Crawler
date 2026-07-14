## Why

`instrument_type` 欄位目前為半成品狀態：DB schema 與程式碼管線皆已建立，但 `normalize_records` 硬編碼為 `"other"`，導致篩選條件 `WHERE instrument_type IN (...)` 永遠不會命中。同時 TPEX 端非「上櫃股票」表格（如 ETF、興櫃）被整張跳過未擷取。這使得 `instrument_types` 篩選參數實際上無作用。

## What Changes

- 在 `normalize_records` 中根據 symbol prefix 推斷 instrument_type（股票、ETF、權證等分類規則）
- 修改 TPEX parser：不要跳過非「上櫃股票」表格，改為根據 table title 設定對應的 instrument_type
- TWSE 端也適用 symbol prefix 分類規則
- `instrument_type` 的值包含：`stock`、`etf`、`etn`、`warrant`、`other`
- 前端 ScreenerView 新增 instrument_type 篩選 UI（多選 checkbox 或下拉選單）

## Capabilities

### New Capabilities
- `instrument-type-classification`: 在資料擷取階段自動分類 instrument_type，並在布林選股頁面提供類型篩選功能

### Modified Capabilities

(無)

## Impact

- `stock-core/src/pipeline/mod.rs` — `normalize_records()` 的 instrument_type 邏輯
- `stock-core/src/fetcher/tpex.rs` — 放寬 table title 篩選，新增分類規則
- `stock-tauri/src/stores/screener.ts` — 現有 instrumentTypes state 變為真正可用
- `stock-tauri/src/views/ScreenerView.vue` — 新增類型篩選 UI
