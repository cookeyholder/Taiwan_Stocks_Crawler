# rust-core-library Specification

## Purpose
TBD - created by archiving change migrate-to-tauri-rust. Update Purpose after archive.
## Requirements
### Requirement: SQLite CRUD 操作
函式庫需透過 rusqlite 提供讀寫市場資料的 repository 模組。

#### Scenario: 依股票代號查詢 OHLCV
- **WHEN** repository 收到股票代號與日期區間
- **THEN** 需回傳依 trade_date 升序排列的 OHLCV 記錄列表

#### Scenario: 批次 upsert 行情資料
- **WHEN** repository 收到一批行情資料列
- **THEN** 需對相同 symbol+date 的資料執行新增或更新

### Requirement: 含 retry 機制的 HTTP 客戶端
函式庫需提供可設定重試次數、逾時時間與 User-Agent 輪換的 HTTP 客戶端模組。

#### Scenario: 成功請求
- **WHEN** 客戶端發送 GET 請求
- **THEN** 成功時需回傳回應內容

#### Scenario: 暫時性失敗時自動重試
- **WHEN** 伺服器回傳 5xx 或連線逾時
- **THEN** 客戶端需以 exponential backoff 重試最多 3 次

### Requirement: 技術指標計算
函式庫需提供 SMA、EMA、布林通道、RSI、MACD 的純函式計算。

#### Scenario: SMA 計算
- **WHEN** 傳入價格陣列與週期
- **THEN** 需回傳 Vec<Option<f64>>，前 period-1 筆為 None

#### Scenario: 布林通道計算
- **WHEN** 傳入價格陣列、週期與標準差倍數
- **THEN** 需回傳與輸入等長的上軌、中軌、下軌陣列

