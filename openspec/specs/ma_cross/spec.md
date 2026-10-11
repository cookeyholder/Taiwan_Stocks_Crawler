# ma_cross Specification

## Purpose

提供移動平均線交叉策略（均線黃金交叉買進、死亡交叉賣出）之回測邏輯。

## Requirements

### Requirement: 移動平均線交叉策略
系統 SHALL 提供一個移動平均線交叉策略（MaCross），在快 SMA 突破慢 SMA 時產生方向訊號。

#### Scenario: 黃金交叉產生買進訊號
- **GIVEN** 價格序列處於上升趨勢
- **WHEN** 快 SMA（5 期）由下往上突破慢 SMA（20 期）
- **THEN** 該日產生 `direction: 1` 且 `has_signal: true`
- **AND** 前一日無訊號（`has_signal: false`）

#### Scenario: 死亡交叉產生賣出訊號
- **GIVEN** 價格序列處於下降趨勢
- **WHEN** 快 SMA（5 期）由上往下跌破慢 SMA（20 期）
- **THEN** 該日產生 `direction: -1` 且 `has_signal: true`
- **AND** 前一日無訊號（`has_signal: false`）

#### Scenario: 無交叉不產生訊號
- **GIVEN** 快 SMA 與慢 SMA 維持固定差距
- **WHEN** 逐日評估
- **THEN** 所有日期皆回傳 `has_signal: false`

#### Scenario: 參數傳遞
- **WHEN** 建立 `MaCrossStrategy { fast_period: 10, slow_period: 50, min_volume: 1000 }`
- **THEN** `min_history()` 回傳 `50`
