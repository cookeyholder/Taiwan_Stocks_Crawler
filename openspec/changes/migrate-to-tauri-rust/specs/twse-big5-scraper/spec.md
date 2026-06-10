## ADDED Requirements

### Requirement: Big5 編碼偵測與轉換
爬蟲需偵測 TWSE 回傳的 Big5 編碼回應，並使用 encoding_rs crate 轉換為 UTF-8。

#### Scenario: Big5 回應正確解碼
- **WHEN** 爬蟲收到 Big5 編碼的回應
- **THEN** 需在 HTML 解析前將原始位元組轉換為 UTF-8

#### Scenario: UTF-8 回應直接通過
- **WHEN** 爬蟲收到 UTF-8 編碼的回應
- **THEN** 需直接解析回應，不進行編碼轉換

### Requirement: 使用 scraper crate 解析 HTML 表格
爬蟲需使用 scraper crate 的 CSS selector API 解析 TWSE HTML 表格。

#### Scenario: 解析行情表格
- **WHEN** 爬蟲收到 TWSE 市場快照 HTML 頁面
- **THEN** 需從資料表格中提取所有列並回傳結構化記錄
