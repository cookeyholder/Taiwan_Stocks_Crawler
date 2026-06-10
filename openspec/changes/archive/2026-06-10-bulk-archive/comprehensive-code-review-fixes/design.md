## Context

在程式碼審查與測試驗證中發現了以下系統漏洞與不良設計：
1. **Unicode 寫入崩潰**：在 `repository.py` 將 raw_payload 寫入 `market_quote_extension` 表時，若 payload 內有 `\udcff` 等 lone surrogate，由於 `json.dumps(..., ensure_ascii=False)` 不會將 surrogate 跳脫為 ASCII 符號，而是保留其字元，此時 SQLite 連線以 UTF-8 寫入時會觸發 `UnicodeEncodeError`。
2. **測試紅燈 (Mock-Mismatch)**：測試中定義的 `mock_get` signature 與 `get_with_retry` 的實際 signature（新增了 `session` 關鍵字參數）不符，造成 `TypeError: mock_get() got an unexpected keyword argument 'session'`。
3. **複雜度警告 (C901)**：`make_fetcher` 與 `fetch_and_ingest_day` 由於內嵌閉包、連線與主檔管理邏輯，導致 Ruff 判定複雜度過高（超過 10）。

## Goals / Non-Goals

**Goals:**
- 修補 lone surrogate 代理字元寫入 UnicodeEncodeError 漏洞。
- 修復所有測試的 mock_get 簽名，使測試套件能正常且全綠燈執行。
- 將 `make_fetcher` 與 `fetch_and_ingest_day` 重構拆分，消除 C901 複雜度警告。
- 補齊 pipeline.py 中的缺漏導入，並確保 `make check` (Ruff) 無任何錯誤。

**Non-Goals:**
- 不變更現有資料庫 Schema 結構。
- 不變更與外部 orchestrator 的互動介面（`make_fetcher` 仍需回傳 Callable，且支援 `close()` 呼叫）。

## Decisions

### 1. 重構 `make_fetcher` 為 `MarketFetcher` 類別
- **決策內容**：原先的 `make_fetcher` 傳回一個閉包函式，且該閉包含有 nonlocal 資源管理與 `_close` 綁定。現在我們將其重構為 `MarketFetcher` 類別，其實作了 `__call__(self, date_str: str) -> bool | None` 與 `close(self) -> None`。
- **評估與理由**：
  - Class 設計更符合 SOLID 與 Clean Code，且能大幅減低 C901 複雜度。
  - 在 Python 中，具備 `__call__` 方法的類別實例會被視為一個 Callable，因此對於外部的呼叫者（`run_date_range_ingestion`）而言，其與原本的閉包 Callable 完全相容。

### 2. 拆分 `fetch_and_ingest_day` 主邏輯
- **決策內容**：
  - 將有價證券主檔的載入邏輯抽離至私有函式 `_resolve_security_master(security_master_path, security_master)`。
  - 將 SQLite 連線與 schema 初始化的邏輯抽離至私有函式 `_resolve_connection(db_path, conn)`。
  - 將 TWSE 與 TPEX 抓取入庫的內部邏輯，提取出私有的 try-except 封裝 `_fetch_and_ingest_twse_safe` 與 `_fetch_and_ingest_tpex_safe`。
- **評估與理由**：這樣能讓 `fetch_and_ingest_day` 的核心流程降至 10 行以內，消除 C901，並提高可讀性。

### 3. 在 `upsert_extension` 中清洗 raw_payload
- **決策內容**：
  - 在 `json.dumps(raw_payload, ensure_ascii=False)` 呼叫之前，先呼叫 `self._normalize_value(raw_payload)`。
- **評估與理由**：`_normalize_value` 會遞迴檢查 dict 內所有字串，並透過 `.encode("utf-8", errors="replace").decode("utf-8")` 將 lone surrogate 字元替換為 `?` 替代符號，進而避免 SQLite 寫入時的編碼錯誤。

### 4. 修復 `mock_get` 簽名
- **決策內容**：
  - 修改 `tests/` 底下所有的 `mock_get` 簽名，改為 `def mock_get(url: str, policy: Any, session: Any = None) -> MagicMock:`。
- **評估與理由**：確保 mock 的參數與實際 pipeline 中呼叫的參數完全一致，防止 parameter mismatch 的 Runtime Error。

## Risks / Trade-offs

- **[Risk]**：將閉包改成 Class `MarketFetcher` 後，如果在別的地方有類型提示註解 `make_fetcher` 回傳的是 function，可能會造成靜態分析的不一致。
- **[Mitigation]**：我們將 `MarketFetcher` 定義在 pipeline.py 中，並確保 `make_fetcher` 的傳回型別註解仍為 `Callable[[str], bool | None]`。由於 `MarketFetcher` 實作了 `__call__`，因此它能被正確解析為 Callable。
