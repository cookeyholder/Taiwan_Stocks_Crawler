## Context

本專案是一個用來爬取臺股交易行情並存入 SQLite 資料庫的爬蟲。目前的實現中，基於對政府網站 SSL 憑證異常的顧慮，硬編碼關閉了 SSL/TLS 憑證檢查，這造成了顯著的資安風險。此外，系統在大資料量批次處理（例如補檔數十天）時，每一天都會重新開啟/關閉資料庫連線、重複載入並解析相同的有價證券主檔 CSV、以及對每筆報價資料執行逐筆 SQL `execute` 寫入，造成嚴重的 CPU 效能黑洞與 I/O 資源浪費。

## Goals / Non-Goals

**Goals:**
1. **安全性修補**：啟用 SSL 憑證驗證，移除抑制警告的設定，並在 pyproject.toml 與 Git 中清理敏感隱私和未驗證二進位檔。
2. **批次 SQL 寫入優化**：重構 Repository 與 Pipeline 支援 `executemany` 批次寫入，使單日數千筆報價在極少數 C API 呼叫下完成入庫。
3. **資源生命週期複用**：在批次處理期間，共用同一資料庫連線、同一 HTTP Session 連線池，且僅載入並解析一次 `StockID.csv`，大幅縮減連線握手與實體檔案 I/O 時間。
4. **細節效能與健壯性優化**：優化日期解析邏輯（快取或提前解析），預編譯 Regex，並為 `to_int` 加入 `ValueError` 異常捕捉。

**Non-Goals:**
1. 不改變現有 SQLite 資料表 Schema（維持 `market_quote_core` 與 `market_quote_extension` 表結構）。
2. 本階段不引入多執行緒/多程序併發（維持單執行緒循序執行，以確保 SQLite 連線的 Thread-safety）。

## Decisions

### 1. SQLite 批次寫入與規整化分離
- **決策**：在 `SQLiteMarketRepository` 中實作 `upsert_core_batch(quotes)` 與 `upsert_extension_batch(quotes)`。
- **具體實作**：
  - 內部使用 `self.connection.executemany(...)`。
  - 將規整化（Normalization）邏輯 `_normalize_quote_payload` 從 Repository 的寫入方法中抽離出來。在寫入前，於 Pipeline 階段對所有 quotes 做一次規整化，然後分別傳入 `upsert_core_batch` 和 `upsert_extension_batch`。這避免了同一筆 Quote 資料被核心表與擴充表寫入時重複規整兩次。

### 2. 共用 DB 連線、HTTP Session 與 Security Master (依賴注入)
- **決策**：擴展 `fetch_and_ingest_day` 的簽章，使其支援可選的 `conn`、`security_master` 和 `session` 參數。
- **具體實作**：
  - 若外部傳入 `conn`、`security_master`、`session`，則直接複用；若為 `None`，則在該方法內部自行建立並關閉。這可以完美相容既有的 CLI 單日執行與現有單元測試。
  - 重構 `pipeline.py#make_fetcher`：在該閉包內部建立一個共享的 `sqlite3.Connection`（並跑一次 `init_schema`）、載入一次 `security_master`、以及建立一個共享的 `requests.Session`。將這三者傳入每次 `_fetcher` 對 `fetch_and_ingest_day` 的呼叫中。
  - 確保在 `make_fetcher` 返回的 `_fetcher` 生命週期結束後，或者在 `main` 函數的最後，能正確關閉資料庫連線與 HTTP Session。
  - 在 `http_client.py` 中，將 `get_with_retry` 重構為支援傳入 `session: requests.Session | None = None`，若有則調用 `session.get`，無則調用 `requests.get`。

### 3. 日期轉換重複解析優化
- **決策**：避免在 `normalize_record` 的迴圈中對每一筆 Quote 都重複解析同一個交易日。
- **具體實作**：
  - 在 `_ingest_records` 的迴圈外部，先對 `date_str` 解析一次，然後以參數傳遞給 `normalize_record`（或使用 `functools.lru_cache(maxsize=32)` 裝飾 `roc_to_ad_date`），使日期的 strptime 解析開銷從 $O(N)$ 降至 $O(1)$。

## Risks / Trade-offs

- **[Risk] SQLite Connection 共享導致 Thread-safety 問題**  
  *Mitigation*: 目前 `orchestrator.py` 為單執行緒執行。我們在代碼註解中明確警告：若未來引入多執行緒併發，必須改為 Thread-local 或連線池，防止併發操作同一個 sqlite3 連線發生 Crash。
- **[Risk] 部分老舊部署環境 SSL 驗證失敗**  
  *Mitigation*: 啟用 `ssl_verify=True` 後，若有系統因缺少根憑證失敗，提供 `--no-ssl-verify` 命令列參數，或在 `HttpPolicy` 中支持傳入自訂 `verify` 憑證路徑，而不是強硬硬編碼 `ssl_verify=False`。
