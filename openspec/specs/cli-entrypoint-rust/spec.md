# cli-entrypoint-rust Specification

## Purpose
TBD - created by archiving change migrate-to-tauri-rust. Update Purpose after archive.
## Requirements
### Requirement: 使用 clap 解析 CLI 參數
CLI 需使用 clap derive API 解析子指令：fetch、screen、sync、classify。

#### Scenario: 下載今日資料
- **WHEN** 執行 `stock-cli fetch --today`
- **THEN** CLI 需下載今日市場資料並寫入 SQLite

#### Scenario: 篩選股票
- **WHEN** 執行 `stock-cli screen --max-bandwidth 0.08 --min-days 5`
- **THEN** CLI 需執行布林通道選股並將結果輸出至 stdout

#### Scenario: 同步 yfinance 歷史資料
- **WHEN** 執行 `stock-cli sync --symbol 2330`
- **THEN** CLI 需從 Yahoo Finance 下載歷史資料並 upsert 至 SQLite

### Requirement: CLI 與桌面版共用程式碼
CLI 需使用相同的 stock-core 函式庫執行所有商業邏輯。

#### Scenario: 相同的分析函式
- **WHEN** CLI 或桌面版任一呼叫布林通道計算
- **THEN** 兩者皆需呼叫同一份 `stock-core::analysis::bollinger()` 函式

