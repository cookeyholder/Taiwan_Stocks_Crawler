## ADDED Requirements

### Requirement: market_closed 作為獨立日期分類
`classify_day()` 函式 SHALL 回傳 `"market_closed"` 當以下條件**全部**成立：
1. 請求日期非週末（weekday）
2. 請求日期不在 `holidays` 集合中（或集合為空）
3. API 有收到回應（非網路錯誤、非格式解析失敗）
4. 解析結果中的資料日期（`trade_date`）≠ 請求日期

#### Scenario: 颱風假被標記為 market_closed
- **WHEN** 工作日且 API 正常回應，但回傳的 `trade_date` 是前一交易日
- **THEN** `classify_day()` 回傳 `"market_closed"`，而非 `"source_error"` 或 `"trading"`

#### Scenario: source_error 維持原語意（網路錯誤）
- **WHEN** API 呼叫拋出網路例外或 HTTP 非 200 狀態
- **THEN** `classify_day()` 回傳 `"source_error"`，不受 `market_closed` 邏輯影響

#### Scenario: 資料日期符合請求日期 → trading
- **WHEN** API 回傳的 `trade_date` 等於請求日期
- **THEN** `classify_day()` 回傳 `"trading"`

#### Scenario: 週末不進入 market_closed 判斷
- **WHEN** 請求日期為週六或週日
- **THEN** `classify_day()` 回傳 `"weekend"`，不做 API 呼叫

### Requirement: BatchQualityReport 新增 market_closed_count
`BatchQualityReport` dataclass SHALL 包含 `market_closed_count: int` 欄位，統計一次批次中被分類為 `market_closed` 的日期數量。

#### Scenario: report 包含 market_closed_count
- **WHEN** 批次執行包含 2 個 market_closed 日期
- **THEN** `report.market_closed_count == 2`

#### Scenario: 無 market_closed 時 count 為 0
- **WHEN** 批次執行不包含任何 market_closed 日期
- **THEN** `report.market_closed_count == 0`

### Requirement: market_closed 不計入 source_error_count
`BatchQualityReport.source_error_count` MUST 只統計真正失敗（網路錯誤、格式解析失敗）的日期，不包含 `market_closed` 日期。

#### Scenario: market_closed 和 source_error 各自獨立計數
- **WHEN** 批次包含 1 個 market_closed 日期與 1 個 source_error 日期
- **THEN** `report.market_closed_count == 1` 且 `report.source_error_count == 1`
