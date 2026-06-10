## ADDED Requirements

### Requirement: 當日 TWSE 資料使用 JSON API
系統 SHALL 在請求日期等於 `datetime.date.today()` 時，使用 TWSE OpenAPI JSON endpoint（`STOCK_DAY_ALL`）取得 TWSE 市場資料。

#### Scenario: 今日日期走 JSON API
- **WHEN** 呼叫 `fetch_and_ingest_day(date=today)` 且 `market == "twse"`
- **THEN** 系統呼叫 `_fetch_and_parse_twse()`（JSON API），不呼叫 CSV API

### Requirement: 歷史 TWSE 資料使用 CSV API
系統 SHALL 在請求日期早於 `datetime.date.today()` 時，使用 TWSE CSV endpoint（`MI_INDEX?response=csv&date=YYYYMMDD&type=ALL`）取得 TWSE 歷史市場資料。

#### Scenario: 歷史日期走 CSV API
- **WHEN** 呼叫 `fetch_and_ingest_day(date=past_date)` 且 `market == "twse"` 且 `past_date < today`
- **THEN** 系統呼叫 `_fetch_and_parse_twse_csv(date)`（CSV API），不呼叫 JSON API

#### Scenario: CSV API 日期格式正確
- **WHEN** 路由決定使用 CSV API 並帶入日期 `2026-01-02`
- **THEN** URL 中的日期參數為 `20260102`（YYYYMMDD，無連字號）

### Requirement: TPEX 資料路由維持不變
系統 MUST 持續使用現有 TPEX CSV API（含日期參數）取得市場資料，無論請求日期為今日或歷史。

#### Scenario: TPEX 所有日期均走既有 CSV 路徑
- **WHEN** 呼叫 `fetch_and_ingest_day(date=any_date)` 且 `market == "tpex"`
- **THEN** 系統呼叫與舊行為相同的 TPEX fetch 函式，不做路由變更

### Requirement: 歷史 TWSE CSV Parser 使用既有 parse_twse_daily_close()
系統 SHALL 使用現有的 `parse_twse_daily_close()` 函式解析從 CSV API 取得的歷史資料，不新增獨立的 parser。

#### Scenario: 歷史 CSV 資料解析流程
- **WHEN** 從 TWSE CSV API 取得歷史資料
- **THEN** 使用 `parse_twse_daily_close(csv_text)` 解析，回傳 `ParseResult(records, errors)`
