## ADDED Requirements

### Requirement: 自動抓取 TWSE holidaySchedule 當年假日清單
系統 SHALL 在執行爬蟲前自動嘗試呼叫 TWSE `holidaySchedule` API，將回傳的國定假日加入 `holidays` 集合，作為 `classify_day()` 的輔助依據。此呼叫為 best-effort：失敗時系統 SHALL 發出 warning 並繼續執行，不中斷作業。

#### Scenario: 自動抓取成功
- **WHEN** `holidaySchedule` API 正常回應
- **THEN** 當年國定假日被加入 `holidays` 集合，後續 `classify_day()` 可正確識別這些日期為 `"holiday"`

#### Scenario: 自動抓取失敗（網路錯誤）
- **WHEN** `holidaySchedule` API 呼叫拋出例外
- **THEN** 系統輸出 warning 訊息，不拋出例外，以空假日集合繼續執行

### Requirement: --no-auto-holidays 旗標停用自動抓取
使用者 SHALL 能透過 `--no-auto-holidays` CLI 旗標停用 `holidaySchedule` 自動抓取行為，系統改為只使用 `--holidays` 所指定的清單（若有）。

#### Scenario: 使用 --no-auto-holidays
- **WHEN** 使用者執行 `twse-crawler --today --no-auto-holidays`
- **THEN** 系統不呼叫 `holidaySchedule` API，直接以空假日集合（或 `--holidays` 指定清單）執行

#### Scenario: 同時指定 --holidays 與自動抓取
- **WHEN** 使用者指定 `--holidays path/to/holidays.json`，且未使用 `--no-auto-holidays`
- **THEN** 系統將自動抓取的假日與 `--holidays` 的清單合併，取聯集後使用

### Requirement: 已知假日應分類為 holiday 而非 market_closed
當 `holidays` 集合中包含某日期，`classify_day()` MUST 在 API 呼叫前即短路回傳 `"holiday"`，不進入 `market_closed` 判斷。

#### Scenario: 已知假日短路
- **WHEN** 請求日期存在於 `holidays` 集合中
- **THEN** `classify_day()` 回傳 `"holiday"`，不發出 API 呼叫
