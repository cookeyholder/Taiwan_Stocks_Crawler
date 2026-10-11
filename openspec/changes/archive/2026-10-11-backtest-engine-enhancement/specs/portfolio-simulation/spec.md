## ADDED Requirements

### Requirement: 交易模擬
系統 SHALL 接收 `SignalRecord` 序列與初始資金，模擬買賣交易並產生 `TradeRecord` 序列與每日權益曲線。

#### Scenario: 買進後賣出
- **WHEN** 某 symbol 在 `2026-07-01` 有 `direction: 1` 的訊號，且在 `2026-07-10` 有 `direction: -1` 的訊號
- **THEN** 系統在 `07-01` 以該日收盤價買入，`07-10` 以該日收盤價賣出
- **AND** 產生一筆 `TradeRecord` 包含買賣日期、價格、股數、損益

#### Scenario: 不足資金不買入
- **WHEN** 訊號要求買入某股，但剩餘資金不足買入 1 股
- **THEN** 跳過該訊號

#### Scenario: direction=0 不做動作
- **WHEN** 收到 `direction: 0` 的訊號
- **THEN** 不產生交易

#### Scenario: 未持倉時收到賣出訊號
- **WHEN** 收到 `direction: -1` 但未持有該 symbol
- **THEN** 跳過該訊號

#### Scenario: 權益曲線每日計算
- **GIVEN** 回測區間有 5 個交易日，訊號僅在第 1 天買進、第 4 天賣出
- **WHEN** `simulate()` 執行
- **THEN** `equity_curve` 回傳 5 個 `f64` 值，第 1 筆與初始資金相等或是買入後扣款值
- **AND** 第 5 筆等於最終總權益（現金 + 持倉市值以當日收盤價計算）
