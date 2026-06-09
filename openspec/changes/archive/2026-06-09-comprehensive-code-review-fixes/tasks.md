## 1. 隱私與靜態檔案清理 (Privacy and Cleanup)

- [ ] 1.1 修改 `pyproject.toml`，移除開發者個人真實 Email，改用姓名或 placeholder。
- [ ] 1.2 移除 `sqlite3/` 底下的所有 Windows 二進位執行檔（`.exe`、`.dll`），並在 `.gitignore` 中忽略該目錄。

## 2. 基礎工具優化 (Base Utilities Optimization)

- [ ] 2.1 修改 `market_crawler/http_client.py`，重構 `get_with_retry` 支援傳入 `session: requests.Session` 複用。
- [ ] 2.2 修改 `market_crawler/normalize.py`，為 `to_int` 加入 `ValueError` 異常捕捉防禦。
- [ ] 2.3 在 `market_crawler/normalize.py` 中，使用 `functools.lru_cache` 快取 `roc_to_ad_date`，避免對同一交易日重複 parsing。
- [ ] 2.4 在 `market_crawler/normalize.py` 與 `market_crawler/classification.py` 中預編譯靜態正則表達式，移至模組級別常數。

## 3. 資料庫與寫入重構 (Database & Ingestion Refactoring)

- [ ] 3.1 修改 `market_crawler/repository.py`，將 `_normalize_quote_payload` 移出寫入流程，並新增 `upsert_core_batch` 和 `upsert_extension_batch`，內部使用 `self.connection.executemany(...)`。
- [ ] 3.2 修改 `market_crawler/pipeline.py` 中的 `_ingest_records`，收集一整批先做完規整化（Normalize）的 quotes，然後呼叫 Repository 的批次寫入方法。

## 4. 生命週期與連線複用 (Lifecycle & Connection Reuse)

- [ ] 4.1 修改 `market_crawler/pipeline.py` 中的 `fetch_and_ingest_day`，支援傳入可選的 `conn`、`security_master` 和 `session` 參數。
- [ ] 4.2 啟用安全憑證驗證：在 `market_crawler/pipeline.py` 中將預設 `ssl_verify=False` 設為 `ssl_verify=True`，並移除 `urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)`。
- [ ] 4.3 重構 `market_crawler/pipeline.py` 中的 `make_fetcher` 與 `main` 函數。在整個批次補檔的生命週期中，共用 Session、DB Connection，且僅載入一次 `security_master`，並確保能正確關閉連線。

## 5. 測試與驗證 (Testing & Verification)

- [ ] 5.1 撰寫單元測試以驗證 `to_int` 的防禦性，確保傳入 `"N/A"` 等字串不會崩潰。
- [ ] 5.2 撰寫測試以驗證批次寫入的正確性。
- [ ] 5.3 執行所有既有測試，確保沒有功能退化（Regression）。
