## Why

目前「下載今日」按鈕只會下載今日一天的資料。如果資料庫已經有數天前的資料，使用者必須手動計算缺失的日期區間，再透過「批次下載」手動輸入起訖日期來補齊。加入「補至今日資料」功能後，只需按一個按鈕即可自動從資料庫最新日期的隔天開始下載到今日。

## What Changes

- 「下載今日」按鈕改為「補至今日資料」
- 按鈕點擊後自動查詢資料庫最新日期，計算 `nextDay = maxDate + 1`，呼叫 `batch_download(nextDay, today)`
- 若資料庫尚無資料則提示使用者手動指定日期
- 若最新日期已是今日則提示「資料已是最新」
- 移除舊的 `fetchToday` 方法，改為 `fillToToday`

## Capabilities

### New Capabilities
- `download-fill-to-today`: 自動補齊資料庫缺失的交易日資料，從最新日期的隔天起下載至今日

### Modified Capabilities

（無）

## Impact

- **stock-tauri/src/views/DownloadView.vue**：`fetchToday` → `fillToToday`，按鈕文字與行為變更
- **stock-tauri/src-tauri/src/lib.rs**：無變更（全前端實作，不需新增後端 command）
