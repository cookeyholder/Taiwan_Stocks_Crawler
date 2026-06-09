## ADDED Requirements

### Requirement: Filter Squeezing Stocks

系統必須從本地 SQLite 資料庫讀取歷史股價，並篩選出布林頻寬（Bandwidth）低於使用者指定門檻值且持續 N 個交易日以上的股票。

#### Scenario: Squeezing stocks filtered successfully
- **WHEN** 使用者設定頻寬門檻值為「0.08」且持續天數為「5」，並點選「開始篩選」
- **THEN** 系統透過 SQLite 與 Pandas 計算，回傳所有符合條件的股票代號、名稱、目前收盤價與目前頻寬列表

### Requirement: Sync History via yfinance

系統必須提供一鍵同步與回補歷史股價（至少 60 天以上）的功能，將 `yfinance` 下載的資料以批次方式寫入本地 SQLite 的 `market_quote_core` 資料表。

#### Scenario: Sync history successfully
- **WHEN** 使用者在介面上點選「同步歷史資料」並輸入股票清單（如台灣50）
- **THEN** 系統透過 yfinance 批次下載歷史資料，並透過 `SQLiteMarketRepository` 寫入本地資料庫
