## 1. 建立下載管理分頁 UI

- [ ] 1.1 在 `StockScreenerApp._create_widgets()` 中新增第三個 Notebook 分頁（下載管理），包含日期模式選擇、日期輸入、下載按鈕、進度日誌區
- [ ] 1.2 實作 `_build_tab3()` 方法，建立下拉選單（下載今日/指定日期/日期區間）、日期輸入框、開始下載按鈕、日誌文字框
- [ ] 1.3 在 `_disable_all_buttons` / `_enable_all_buttons` 中加入新分頁按鈕的狀態切換

## 2. 背景下載執行緒

- [ ] 2.1 實作下載 worker 函式：根據使用者選擇的模式呼叫 `fetch_and_ingest_day()`（單日）或 `run_date_range_ingestion()`（區間），以獨立 `requests.Session` 與 `sqlite3.Connection` 執行
- [ ] 2.2 透過 `task_queue` 回傳進度訊息（單日完成/區間逐日進度/最終摘要）
- [ ] 2.3 單日模式完成後顯示成功/失敗結果；區間模式逐日回報「正在處理 YYYY-MM-DD (x/y)」，完成後顯示成功天數、休市天數、失敗天數

## 3. 佇列處理與 UI 更新

- [ ] 3.1 在 `_process_queue()` 中新增 `BULK_DOWNLOAD_PROGRESS`、`BULK_DOWNLOAD_DONE` 等訊息類型的處理分支
- [ ] 3.2 進度更新時：設進度條為確定模式（`mode="determinate"`）、更新 `status_var`、寫入日誌框
- [ ] 3.3 完成時：恢復按鈕、進度條歸零、顯示結果摘要（成功/休市/失敗天數）

## 4. Holiday 邏輯整合

- [ ] 4.1 下載區間模式時自動抓取 TWSE holiday schedule（沿用 CLI 的邏輯），跳過非交易日
- [ ] 4.2 提供 checkbox 讓使用者選擇是否啟用自動假日偵測（預設啟用）
