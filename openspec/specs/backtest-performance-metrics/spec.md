# backtest-performance-metrics Specification

## Purpose

提供回測交易紀錄之量化績效指標計算，包含總報酬率、勝率、Sharpe Ratio 與最大回撤。

## Requirements

### Requirement: 績效指標計算
系統 SHALL 從 `TradeRecord` 序列計算績效指標，包含總報酬率、勝率、Sharpe ratio、最大回撤。

#### Scenario: 計算勝率
- **WHEN** 10 筆已了結交易中 6 筆獲利、4 筆虧損
- **THEN** `win_rate` 回傳 `0.6`

#### Scenario: 計算總報酬率
- **WHEN** 初始資金 1,000,000，最終價值 1,150,000
- **THEN** `total_return_pct` 回傳 `15.0`

#### Scenario: 計算最大回撤
- **WHEN** 權益曲線最高點 1,200,000，最低點 1,100,000
- **THEN** `max_drawdown_pct` 回傳約 `8.33`

#### Scenario: 計算 Sharpe ratio
- **GIVEN** `equity_curve` 為 `[1_000_000, 1_010_000, 1_020_000, 1_015_000, 1_030_000]`
- **WHEN** 無風險利率 0%
- **THEN** Sharpe ratio 約為 `(mean_daily_return / std_daily_return) * sqrt(252)`
