## Why

本變更旨在解決全面程式碼審查中發現的問題，包含資料寫入異常、測試 Mock 缺陷、模組複雜度過高、未定義變數與格式警告，藉此提升系統穩定度與可維護性。

## What Changes

- **修正資料寫入異常**：在寫入擴充表 (extension table) 之前，將 raw_payload 內含有 lone surrogate 代理字元的欄位進行 utf-8 編碼清洗，避免 SQLite 寫入時觸發 `UnicodeEncodeError`。
- **重構高複雜度代碼**：
  - 將 `market_crawler/pipeline.py` 中的 `make_fetcher` 重構為一個實作了 `__call__` 的類別 `MarketFetcher`，降低閉包帶來的複雜度警告 (C901)。
  - 將 `fetch_and_ingest_day` 進行拆分，把有價證券主檔加載、資料庫連線初始化等流程獨立為 helper 函式，以修復 C901 警告。
- **補齊未定義名稱與未使用之匯入**：在 `pipeline.py` 頂端導入 `requests` 與 `Any`，並移除未使用的 `urllib3`，使 Ruff 靜態檢查無錯誤。
- **修復測試 Mock 規格不符問題**：調整 `tests/` 下所有的 `mock_get` 簽名，使其支援與真實被 mock 的 `get_with_retry` 函式相同的 `session` 關鍵字參數，讓整個測試套件得以正常執行。
- **消除程式碼格式警告**：移除多處空行含空白字元的格式問題。

## Capabilities

### New Capabilities
- 無（本變更僅為內部實作品質與邏輯修補，未新增外部業務功能）。

### Modified Capabilities
- 無。

## Impact

受影響的模組包含：
- `market_crawler/pipeline.py`（依賴導入、 make_fetcher 重構、 fetch_and_ingest_day 拆分）
- `market_crawler/repository.py`（upsert_extension 的 Unicode 安全處理）
- `tests/test_pipeline.py` 及 `tests/test_guardrails_and_regressions.py`（mock_get 簽名調整）
