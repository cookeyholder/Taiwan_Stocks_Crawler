## Why

目前 backtest 基礎建設只支援單一策略（BollingerSqueezeBreakout），缺乏績效評估指標與交易模擬功能。使用者無法比較不同策略的優劣，也無法量化回測結果的效益。

## What Changes

- 新增 `PerformanceMetrics` 與 `BacktestReport` 結構體
- 新增 `PortfolioSimulator`：根據方向訊號模擬買賣交易，計算持倉損益
- 新增 `StrategyKind` enum + 策略註冊機制（沿用 classify_by_symbol 的集中註冊模式）
- 新增 `MaCrossStrategy`（移動平均線交叉）與 `RsiStrategy`（RSI 超買超賣）兩個策略
- CLI `backtest` 子命令新增 `--strategy` 參數，支援選擇策略與傳入參數
- CLI 輸出新增績效摘要

## Capabilities

### New Capabilities
- `backtest-performance-metrics`: 績效評估指標計算（總報酬率、勝率、Sharpe、最大回撤）
- `portfolio-simulation`: 從訊號序列模擬買賣交易與持倉管理（方向 A1：訊號 = 事件，PortfolioSimulator 決定進出場）
- `backtest-strategy-registry`: 策略註冊與列舉，支援新增策略不需改動 engine

## Impact

- **stock-core/src/backtest/**：新增 `performance.rs`、`portfolio.rs`、`strategies.rs`、`ma_cross.rs`、`rsi.rs`
- **stock-core/src/backtest/mod.rs**：新增 `pub use` 匯出
- **stock-core/src/models.rs**：新增 `PerformanceMetrics`、`TradeRecord`、`BacktestReport`、`StrategyInfo`
- **stock-cli/src/main.rs**：`backtest` 子命令新增 `--strategy`、`--capital`、動態策略參數
