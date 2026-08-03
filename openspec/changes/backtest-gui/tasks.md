## 1. Tauri Command

- [x] 1.1 新增 `run_backtest` command：接收 RunBacktestParams，建立 BacktestEngine + 策略，執行 run_range，傳入 PortfolioSimulator + calculate_metrics，回傳 BacktestReport
- [x] 1.2 新增 `list_strategies` command：委託 StrategyKind::list()
- [x] 1.3 在 invoke_handler 註冊新 command

## 2. 前端 Store

- [x] 2.1 新增 `stores/backtest.ts`：Pinia store，管理策略清單、參數、日期、資金、載入狀態、回測報告
- [x] 2.2 新增 `fetchStrategies()`：呼叫 list_strategies
- [x] 2.3 新增 `runBacktest()`：收集參數，呼叫 run_backtest，更新報告
- [x] 2.4 新增 `clearReport()`

## 3. 前端路由與導覽

- [x] 3.1 router.ts 新增 /backtest 路由 -> BacktestView.vue
- [x] 3.2 App.vue 導覽列新增「回測」頁籤

## 4. 前端 UI

- [x] 4.1 實作 BacktestView.vue：
      - 策略下拉選單（動態從 store.strategies 取得）
      - 動態參數表單（依選擇的策略 render 對應欄位）
      - 日期區間選擇器（起始/結束）
      - 資金輸入欄位、類型 checkbox
      - 執行按鈕（disabled 當 loading）
- [x] 4.2 績效摘要四個卡片（總報酬率、勝率、Sharpe、最大回撤）
- [x] 4.3 可排序的訊號表格
- [x] 4.4 交易明細表格

## 5. 格式化與驗證

- [x] 5.1 cargo check + cargo fmt
- [x] 5.2 手動驗證回測流程
