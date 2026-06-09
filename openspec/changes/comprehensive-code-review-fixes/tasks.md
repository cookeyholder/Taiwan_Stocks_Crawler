## 1. 核心程式碼重構 (Core Code Refactoring)

- [x] 1.1 於 `market_crawler/pipeline.py` 頂端補齊 `from typing import Any` 及 `import requests`，並移除未使用的 `import urllib3`，消除 Ruff 的 F821 與 F401 錯誤。
- [x] 1.2 在 `market_crawler/pipeline.py` 中，將 `make_fetcher` 重構為一個實作了 `__call__` 與 `close()` 的類別 `MarketFetcher`，解決 C901 警告。
- [x] 1.3 拆分 `market_crawler/pipeline.py` 中的 `fetch_and_ingest_day` 函數邏輯，將載入有價證券主檔、初始化連線與抓取封裝成私有 helper 函式，以修復 C901 警告。
- [x] 1.4 在 `market_crawler/repository.py` 的 `upsert_extension` 及 `upsert_extension_batch` 中對 `raw_payload` 進行 `_normalize_value` 處理，防範 lone surrogate 造成的 UnicodeEncodeError。

## 2. 測試與 Mock 規格修正 (Test Suite Refactoring)

- [x] 2.1 修正 `tests/test_pipeline.py` 中所有的 `mock_get` 簽名，使其支援 `session=None` 參數。
- [x] 2.2 修正 `tests/test_guardrails_and_regressions.py` 中 `mock_get` 的簽名，使其相容 `session` 參數。

## 3. 程式碼品質與測試驗證 (Verification & Linting)

- [x] 3.1 執行靜態檢查 `uv run ruff check .`，確保沒有任何 linter 警告與錯誤。
- [x] 3.2 執行所有測試 `.venv/bin/pytest`，確保原本失敗的 11 個測試全數通過，且其餘測試維持綠燈。
- [x] 3.3 執行 `make format`，移除程式碼中空行含有空白字元的格式警告 (W293)。
