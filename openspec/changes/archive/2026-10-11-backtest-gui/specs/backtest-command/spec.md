## ADDED Requirements

### Requirement: 執行回測
系統 SHALL 提供 run_backtest Tauri command，接受策略名稱、參數、日期區間、資金、類型，回傳 BacktestReport。

#### Scenario: 執行成功
- **WHEN** 呼叫 run_backtest 帶入有效參數
- **THEN** 回傳 BacktestReport 包含 signals、metrics、trades、equity_curve

### Requirement: 列舉策略
系統 SHALL 提供 list_strategies Tauri command，回傳可用策略清單。

#### Scenario: 回傳策略資訊
- **WHEN** 呼叫 list_strategies
- **THEN** 回傳至少 3 個策略的 StrategyInfo 陣列
