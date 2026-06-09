## Why

本變更旨在修復臺股成交資訊收集系統中的安全性風險與效能瓶頸。包含修補停用 SSL 驗證所帶來的中間人攻擊 (MITM) 風險，以及優化逐筆 SQL 寫入、重複 CSV 檔案 I/O、頻繁連線握手等重大效能黑洞，提升系統的健壯性與在大資料量批次補檔時的執行效率。

## What Changes

- **資安防護強化**：
  - 啟用 SSL/TLS 憑證驗證，預設 `ssl_verify=True`，並移除抑制 `InsecureRequestWarning` 的警告忽略程式碼。
  - 在 `pyproject.toml` 中將作者個人真實 Email 隱藏，避免垃圾郵件與精準釣魚風險。
  - 從 Git 庫中移除並忽略 `sqlite3/` 目錄下的 Windows 二進位執行檔（`.exe`、`.dll`），防範惡意二進位替換風險。
- **寫入效能優化**：
  - 支援 SQLite 批次寫入，將 `SQLiteMarketRepository` 重構為以 `executemany` 方式批次匯入每日報價資料。
  - 將 Quote 规整化（Normalize）移至寫入外層，對同一筆 Quote Payload 僅規整化一次，消除 core 與 extension 表寫入時的重複規整化開銷。
- **連線與資源複用**：
  - 在批次補檔期間，共用同一資料庫連線，避免每一天都重新建立連線、初始化 schema 與關閉。
  - 共用同一個 `requests.Session` 連線池進行 HTTP 抓取，啟用 TCP Keep-Alive 以消除重複的 TLS 握手延遲。
  - 批次補檔期間僅載入一次 `StockID.csv` 有價證券主檔並於記憶體複用，避免每一天都重新進行檔案 I/O 與 CSV 解析。
- **運算與細節優化**：
  - 引入日期解析快取或重構 `_ingest_records` 的迴圈，使同一個交易日的 `strptime` 呼叫次數從 $O(N)$ 降至 $O(1)$。
  - 為 `to_int` 數值清洗函式加入 `ValueError` 異常捕捉防禦，防止 API 回傳未預期的非數字字串導致爬蟲崩潰。
  - 將分類規則中的靜態正則表達式（Regex）移至 module 級別預編譯，降低 runtime 重複解析開銷。

## Capabilities

### New Capabilities
- 無（本案為現有系統的內部重構與缺陷修復）

### Modified Capabilities
- 無（不影響現有 API 與業務邏輯協議，僅重構實作細節）

## Impact

- `market_crawler/pipeline.py`：調整 HTTP 策略、重構批次 fetcher 閉包、複用 Session 與 DB 連線。
- `market_crawler/repository.py`：新增批次寫入介面 `upsert_core_batch`、`upsert_extension_batch`。
- `market_crawler/http_client.py`：擴展 `get_with_retry` 支援 `requests.Session` 傳入。
- `market_crawler/normalize.py`：最佳化 `to_int`、優化日期轉換重複解析、預編譯正則。
- `market_crawler/classification.py`：預編譯正則。
- `pyproject.toml`：清除作者 Email。
- `sqlite3/`：自 Git 庫中刪除該目錄，並更新 `.gitignore`。
