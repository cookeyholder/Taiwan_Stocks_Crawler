## Why

CLI 的回測功能需要使用者操作命令列，無法直覺地比較不同策略與參數組合。加入 GUI 頁面後，使用者可以透過視覺化介面選擇策略、調整參數、觀看績效摘要與訊號表格。

## What Changes

- 新增 Tauri command `run_backtest`：接受策略名稱、參數、日期區間、資金、類型，回傳 BacktestReport
- 新增 Tauri command `list_strategies`：回傳可用策略清單（含參數定義）
- 新增路由 /backtest 與 BacktestView.vue
- App.vue 導覽列新增「回測」頁籤
- 策略選擇器：下拉選單，選擇後動態切換參數表單
- 績效摘要：四個卡片顯示總報酬率、勝率、Sharpe、最大回撤
- 訊號表格：可排序，顯示日期、代號、名稱、買賣方向、價格
- 交易明細表格：顯示每筆交易的買入/賣出日期、價格、損益

## Capabilities

### New Capabilities
- `backtest-command`: run_backtest / list_strategies 兩個 Tauri command
- `backtest-ui`: 回測頁面 UI（策略選擇、參數表單、績效摘要、訊號表格、交易明細）

## Impact

- stock-tauri/src-tauri/src/lib.rs：新增 run_backtest、list_strategies command
- stock-tauri/src/router.ts：新增 /backtest 路由
- stock-tauri/src/App.vue：導覽列新增「回測」頁籤
- stock-tauri/src/views/BacktestView.vue：回測頁面
- stock-tauri/src/stores/backtest.ts：回測 store
