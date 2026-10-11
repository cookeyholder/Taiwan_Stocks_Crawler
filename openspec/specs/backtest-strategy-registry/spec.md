# backtest-strategy-registry Specification

## Purpose

提供集中式的策略註冊與列舉機制，支援依名稱建立策略實例與查詢參數規格。

## Requirements

### Requirement: 策略列舉
系統 SHALL 提供一個集中註冊點（`StrategyKind` enum），列出所有可用的回測策略。

#### Scenario: 列出所有策略
- **WHEN** 呼叫 `StrategyKind::list()`
- **THEN** 回傳至少包含 `bollinger_squeeze`、`ma_cross`、`rsi` 三個策略的 `StrategyInfo` 陣列
- **AND** 每個 `StrategyInfo` 包含唯一 id、顯示名稱、參數定義

#### Scenario: 依名稱建立策略
- **WHEN** 傳入 `"ma_cross"` 與 `{ fast_period: 5, slow_period: 20 }`
- **THEN** 回傳 `Ok(MaCrossStrategy { fast_period: 5, slow_period: 20 })`
- **AND** `min_history()` 回傳 `20`
