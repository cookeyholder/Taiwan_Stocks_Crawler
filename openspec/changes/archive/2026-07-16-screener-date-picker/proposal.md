## Why

布林通道選股頁面目前固定以最新交易日篩選突破訊號。使用者需要能夠指定特定日期，查看歷史上某個交易日發生突破的股票，以便回溯驗證策略或查找特定日期的標的。

## What Changes

- `Pipeline::screen()` 新增 `date: Option<&str>` 參數，傳入時直接跑該日期，不傳時維持最新交易日（向後相容）
- `screen_bollinger` Tauri command 新增 `date: Option<String>` 參數
- 前端 ScreenerView 新增日期選擇器輸入欄位，預設填入最新交易日
- screener.ts store 新增 `date` 狀態，invoke 時傳入
- CLI `screen` 子命令新增 `--date` 選項

## Capabilities

### New Capabilities
- `screener-date-filter`: 在布林通道選股頁面提供日期選擇功能，支援以指定交易日篩選突破訊號

### Modified Capabilities

(無)

## Impact

- `stock-core/src/pipeline/mod.rs` — `screen()` 簽名變更
- `stock-tauri/src-tauri/src/lib.rs` — `screen_bollinger` 指令簽名變更
- `stock-tauri/src/stores/screener.ts` — 新增 date state 與 invoke 參數
- `stock-tauri/src/views/ScreenerView.vue` — 新增日期輸入 UI
- `stock-cli/src/main.rs` — Screen 子命令新增 `--date`
