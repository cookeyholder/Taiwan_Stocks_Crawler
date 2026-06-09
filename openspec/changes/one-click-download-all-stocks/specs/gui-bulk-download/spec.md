## ADDED Requirements

### Requirement: One-click download all stocks

GUI 必須提供一個「下載管理」分頁，讓使用者一鍵觸發 TWSE 與 TPEX 所有上市、上櫃股票的收盤行情下載，並將資料寫入本地 SQLite。

#### Scenario: Download today's data
- **WHEN** 使用者選取「下載今日」並點擊「開始下載」
- **THEN** 系統在背景執行緒呼叫 `fetch_and_ingest_day(today)` 下載今日 TWSE + TPEX 行情，完成後顯示結果摘要

#### Scenario: Download by specific date
- **WHEN** 使用者輸入指定日期 YYYY-MM-DD 並點擊「開始下載」
- **THEN** 系統在背景執行緒呼叫 `fetch_and_ingest_day(date_str)` 下載該日行情，完成後顯示結果摘要

#### Scenario: Download date range
- **WHEN** 使用者輸入起始與結束日期並點擊「開始下載」
- **THEN** 系統在背景執行緒呼叫 `run_date_range_ingestion()` 遍歷區間內每個交易日下載行情，逐日回報進度

### Requirement: Progress feedback during download

GUI 必須在下載過程中即時顯示當前進度，並在完成後顯示摘要。

#### Scenario: Progress shown during range download
- **WHEN** 系統正在下載多日期區間資料
- **THEN** 狀態列顯示「正在處理 YYYY-MM-DD (x/y)」，進度條設為確定模式反映完成比例

#### Scenario: Completion summary shown
- **WHEN** 下載完成後
- **THEN** 顯示成功天數、休市天數、失敗天數的摘要資訊

### Requirement: Non-blocking background execution

下載作業必須在背景執行緒中執行，不阻塞 GUI 主執行緒。

#### Scenario: UI remains responsive during download
- **WHEN** 背景正在下載資料
- **THEN** 使用者仍可切換分頁、瀏覽既有資料；下載按鈕在執行期間禁用防止重複觸發
