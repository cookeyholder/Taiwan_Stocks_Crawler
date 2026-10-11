# backtest-chart Specification

## Purpose

在回測系統提供策略整體權益曲線（Equity Curve）與回撤百分比（Drawdown）視覺化圖表。

## Requirements

### Requirement: 權益曲線圖
系統 SHALL 在回測結果頁面顯示策略權益曲線與基準線的比較圖。

#### Scenario: 顯示權益曲線
- **WHEN** 回測完成且有 equity_curve 資料
- **THEN** 顯示折線圖，包含策略權益線與基準線
- **AND** X 軸為日期、Y 軸為資金金額

### Requirement: 回撤區域圖
系統 SHALL 在權益曲線下方顯示回撤百分比圖表。

#### Scenario: 顯示回撤
- **WHEN** 回測完成且有 equity_curve 資料
- **THEN** 在權益曲線圖下方顯示回撤圖，紅色線條/區域標示回撤
