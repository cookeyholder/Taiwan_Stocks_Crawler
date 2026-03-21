# Copilot 開發指引

## 建置、測試與 lint 指令

本專案使用 `uv` 進行開發，目標 Python 版本為 3.13（見 `pyproject.toml`、`.devcontainer/devcontainer.json`）。

安裝相依套件：

```bash
uv sync --all-extras
```

`Makefile` 中的主要指令：

```bash
make test
make test-cov
make check
make format
make fix
make pre-commit
```

如果 `ruff` 或 `pytest` 還不能直接執行，可改用下面這些等價指令：

```bash
uv run pytest
uv run pytest tests/test_parsers.py
uv run pytest tests/test_parsers.py::test_parse_twse_extracts_daily_close_section
uv run ruff check .
uv run ruff format .
pre-commit run --all-files
```

## OpenSpec 注意事項
- 規劃任務時，最後一項要是「更新文件」，並且在每項任務後加上「驗收標準」，明確定義在什麼情況下任務可以視為完成。請盡量採用可量化的標準；如果是質性的標準，也要條列清楚。
- 採用 TDD 流程，先寫測試再實作，確保每個子任務都有對應的 failing test，並且在實作後驗證所有測試都通過。
- 除了單元測試之外，也要有整合測試、流程測試與防呆測試，並且盡量涵蓋已知的錯誤案例。覆蓋率目標 100%，測試案例數量至少 40 個，其中防呆測試至少 10 個。
- 請務必在 `/memories/session/plan.md` 中撰寫完整的實作計畫，並在呈現給使用者的版本中保持清晰與簡潔。
- 在撰寫計畫時，請確保涵蓋所有必要步驟、相關檔案、驗證方式、決策與後續考量，讓使用者能完整理解並有效執行。
- 在使用 #tool:vscode/askQuestions 進行澄清時，請確保問題具體且提供足夠的背景資訊，以便使用者能夠給出明確的回答。
- 在撰寫驗證步驟時，請盡量具體，包含明確的測試案例、指令或 MCP 工具，讓驗證流程有效且可執行。
- 採取高頻提交的策略，每完成一個子任務就提交一次，確保每次提交都能通過所有測試，並且在 commit message 清楚說明變更內容與目的。


## 提交 commit 時的注意事項
- 使用繁體中文撰寫 commit message，必須使用臺灣慣用語，避免出現中國常用語。
- 請遵循專案既有的 conventional commit 風格，使用 `feat:`、`fix:`、`docs:`、`test:`、`refactor:` 等類型標籤。
- 在 commit message 中清楚說明變更內容與目的，可以的話也提到相關測試或文件更新。
- 如果 commit 涉及特定任務或問題，請在提交訊息中引用相關 issue 或任務編號，方便追蹤管理。
- 提交前請確認所有相關測試都已通過，且程式碼也已通過 lint 檢查，確保提交品質符合專案標準。


## 測試與驗證的注意事項
- 在撰寫測試時，請確保每個測試案例都有明確的目的與預期結果，並且涵蓋各種正常與異常情況。
- 在驗證過程中，請確認所有測試都已通過，並且沒有用任何 `xfail` 或 `skip` 來規避驗證。
- 在驗證日期區間抓取功能時，請確認能夠區分正常交易日、例假日、颱風假／臨時休市，以及來源錯誤等不同情況，確保功能夠穩定。
- 在驗證資料解析與標準化功能時，請確認 TWSE `=` 開頭的代號不會遺失，且所有日期都已正確轉成西元格式，以維持資料完整性與一致性。
- 在驗證資料庫寫入功能時，請確認重複匯入同一區間不會產生重複資料，且資料更新符合預期，以確保資料庫內容正確可靠。
- 如果是 CI 驗證，請確保所有測試與驗證步驟都能在 CI 環境中順利執行，並提供清楚的報告。一旦出錯就中止 CI 並立即修正，修正後重新執行 CI 確認問題已排除，直到所有錯誤都解決為止。

## 整體架構

目前主要維護的程式碼在 `market_crawler/` 套件。除非任務明確指定那些檔案，否則應優先擴充這個套件，而不是把新邏輯加到根目錄裡那些較早期的腳本與 notebooks。

主要流程如下：

1. `market_crawler.http_client.get_with_retry()` 依照 retry/backoff 政策抓取上游來源的市場資料檔。
2. `market_crawler.parsers` 解析 TWSE 與 TPEX 的每日收盤資料，輸出原始資料字典，以及結構化的 `ParseError` 項目。
3. `market_crawler.normalize.normalize_record()` 將各市場格式的資料列轉成統一的 canonical quote 格式，包含民國年轉西元日期與數值清理。
4. `market_crawler.classification.classify_instrument()` 會先使用 `StockID.csv` 補充分類資訊，若不足再退回到代號樣式規則。
5. `market_crawler.repository.SQLiteMarketRepository` 使用兩張表把資料寫入 SQLite：`market_quote_core` 存放標準化後的報價欄位，`market_quote_extension` 存放原始 payload、raw change 與其他附加中繼資料。
6. `market_crawler.orchestrator` 負責日期區間的批次匯入流程、交易日分類（`trading`、`holiday`、`weekend`、`source_error`），以及透過 `BatchQualityReport` 產生品質報告。

`tests/` 中的測試反映了目前實際維護的行為，應視為這個套件 API 最可靠的規格來源。

## 重要慣例

- 將 `market_crawler/` 視為主要實作。根目錄的 crawler 腳本如 `crawler_twse.py`、`crawler_otc.py` 與多數 notebooks，多半是較早期留下的流程，並不是目前主要的測試路徑。
- 同時保留 `symbol_raw` 與 `symbol_clean`。TWSE 資料可能包含像 `="0050"` 這種 Excel 風格代號；parser 會保留原值，而 normalization 會把它整理成專案內使用的標準 key。
- 解析流程採「收集錯誤」而不是「遇錯即停」。`parse_twse_daily_close()` 與 `parse_tpex_daily_close()` 會回傳 `ParseResult(records, errors)`，並以 `ParseError` 記錄格式異常的資料列，而不是每碰到一筆壞資料就直接拋出例外。
- 編碼處理很重要。來源檔案預期會依照 `market_crawler.encoding` 的 fallback 順序解碼：`cp950`、`big5hkscs`、`utf-8-sig`，最後才是 `utf-8`。
- 商品分類有固定的優先順序：先看明確的 override mapping，再看 `StockID.csv` 的產業／區段標籤，再看 `CFICode`，最後才套用 `classification.py` 中的代號樣式 fallback 規則。
- SQLite schema 的設計具備冪等性。`market_quote_core` 與 `market_quote_extension` 都使用 `ON CONFLICT ... DO UPDATE`，因此重複匯入時應更新既有資料列，而不是產生重複資料。
- 測試會在 `tests/conftest.py` 中把專案根目錄加入 `sys.path`，且匯入方式預設採用套件路徑，例如 `from market_crawler...`。
- 如果要調整 parser 或 normalization 行為，請同步更新 `tests/test_parsers.py`、`tests/test_normalize.py`、`tests/test_guardrails_and_regressions.py` 這些對應測試，必要時也更新 `tests/fixtures/` 下的範例資料。
- 請遵循 `CONTRIBUTING.md` 既有的工具慣例：優先使用 dev container、以 `uv` 管理相依套件，並延續現有的 conventional commit 風格（`feat:`、`fix:`、`docs:`、`test:`、`refactor:`）。


## 文件更新
- 在 `docs/` 中撰寫與更新文件，並且在 commit message 中清楚說明文件更新的內容與目的。
- 重要文件包括但不限於：`docs/canonical_schema.md`（canonical 欄位字典與對映表）、`docs/classification_rules.md`（分類規則決策表與維護準則）、`docs/debugging_guide.md`（新來源格式變動時的排查流程）、`docs/testing_matrix.md`（測試矩陣：測試類型、案例數、涵蓋風險、對應需求）等。
- 文件內容應該清晰、具體且易於理解，並且盡量包含實際的範例與參考，讓使用者能夠有效地使用這些文件來理解專案的規格、流程與最佳實踐。


## 其他注意事項
- AI agents（例如 Plan Agent、Ask Agent）應該嚴格遵守專案的規範與流程，並且在執行任務時保持透明與可追蹤，確保每個步驟都有明確的記錄與驗證。
- 在察覺到自己的 context window 快要滿了時，應該主動使用 #tool:vscode/memory 把重要的上下文資訊保存到 `/memories/session/` 中，以便在後續的對話或任務中能夠繼續參考這些資訊。
- 在使用工具（例如 search、read、askQuestions）來收集資訊或澄清問題時，應該確保問題具體且提供足夠的背景資訊，以便使用者能夠給出明確的回答，並且在收集到資訊後能夠有效地整合這些資訊來完成任務。
- 在撰寫計畫或回答問題時，應該盡量具體且提供明確的步驟、相關檔案、驗證方式、決策與後續考量，讓使用者能夠完整理解並有效執行，而不是提供模糊或過於抽象的建議。