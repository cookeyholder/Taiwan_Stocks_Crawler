# rsi Specification

## Purpose

提供 RSI 超買超賣策略之回測訊號計算邏輯。

## Requirements

### Requirement: RSI 超買超賣策略
系統 SHALL 提供一個 RSI 策略，在 RSI 穿越超買/超賣門檻時產生方向訊號。

#### Scenario: 超賣回升產生買進訊號
- **GIVEN** RSI 低於 oversold（例如 30）
- **WHEN** RSI 由 oversold 之下回升至 oversold 之上
- **THEN** 該日產生 `direction: 1` 且 `has_signal: true`
- **AND** 前一日無訊號（`has_signal: false`）

#### Scenario: 超買回落產生賣出訊號
- **GIVEN** RSI 高於 overbought（例如 70）
- **WHEN** RSI 由 overbought 之上回落至 overbought 之下
- **THEN** 該日產生 `direction: -1` 且 `has_signal: true`
- **AND** 前一日無訊號（`has_signal: false`）

#### Scenario: 未穿越不產生訊號
- **GIVEN** RSI 維持在 oversold 與 overbought 之間
- **WHEN** 逐日評估
- **THEN** 所有日期皆回傳 `has_signal: false`

#### Scenario: 參數傳遞
- **WHEN** 建立 `RsiStrategy { period: 14, overbought: 75, oversold: 25, min_volume: 0 }`
- **THEN** `min_history()` 回傳 `15`
