## Why

現有 CLI (`twse-crawler`) 已經支援一次下載所有上市、上櫃股票，但 GUI 使用者仍需手動逐個日期操作命令列，無法在圖形介面中一鍵觸發完整市場資料下載。

## What Changes

- **GUI 新增「下載全部」按鈕**：在 GUI 主視窗中加入一個按鈕，點擊後在背景執行緒中依序下載所有上市（TWSE）與上櫃（TPEX）股票，並以進度條或文字回饋顯示執行狀態。
- **支援多種下載模式**：支援「下載今日」、「下載指定日期」、「下載區間」等選項，讓使用者透過 GUI 選擇抓取範圍。
- **共用現有 Pipeline**：直接呼叫現有的 `pipeline.py` 中的 `fetch_and_ingest_day()` 和 `orchestrator.py` 中的日期迭代邏輯，不重複實作爬蟲邏輯。
- **進度回饋**：透過 queue + `after()` 定時器機制，將背景執行緒的進度訊號（正在下載哪一天、已完成筆數等）傳回 GUI 主執行緒更新狀態列。

## Capabilities

### New Capabilities
- `gui-bulk-download`: 提供 GUI 一鍵下載所有上市、上櫃股票的功能，包含日期範圍選擇與即時進度回饋。

### Modified Capabilities
- 無。

## Impact

- **新增/修改檔案**：
  - `market_crawler/gui.py`（新增下載按鈕、日期選擇器、進度列；調整視窗佈局）
- **相容性**：不影響現有 CLI、資料庫 Schema 與爬蟲邏輯，完全向下相容。
