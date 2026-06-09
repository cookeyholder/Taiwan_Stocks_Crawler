# 程式碼審查報告

> 語言規範：本報告必須使用「臺灣慣用語」的繁體中文撰寫（避免簡體中文與中國慣用詞）。

## 基本資訊
- 產生時間：2026-06-09 20:17:27
- 專案路徑：`/home/cookeyholder/Projects/Taiwan_Stocks_Crawler`
- 分支：`review-fix/comprehensive-code-review-fixes`
- 版本：`5981c35`
- 建議的 OpenSpec change：`comprehensive-code-review-fixes`

## 範圍快照
### 主要副檔名分布
- `.md`: 46
- `.py`: 24
- `.toml`: 5
- `.yaml`: 5
- `.csv`: 2
- `.json`: 2
- `<no_ext>`: 2
- `.md"`: 1
- `.pdf"`: 1
- `.sh`: 1
- `.txt"`: 1

### 自動化風險訊號（快速掃描）
| 風險訊號 | 命中數 | 範例 |
|---|---:|---|
| 疑似硬編碼機密字串 | 0 | - |
| 可能的 SQL 字串插值風險 | 0 | - |
| JS eval / Function 建構子風險 | 0 | - |
| Python subprocess 使用 shell=True | 0 | - |
| 過於寬鬆的例外處理 | 10 | ./market_crawler/orchestrator.py:81:            except Exception as exc:  # noqa: BLE001<br>./market_crawler/fetch_security_master.py:231:    except Exception:<br>./market_crawler/http_client.py:39:        except Exception as exc:  # noqa: BLE001<br>./market_crawler/pipeline.py:70:    except Exception:  # noqa: BLE001<br>./market_crawler/pipeline.py:355:            except Exception as exc:  # noqa: BLE001 |

## 審查發現

### P0_CRITICAL
- 無致命等級問題。

### P1_HIGH
1. **資料寫入異常 (UnicodeEncodeError)**
   - **問題描述**：在 `market_crawler/repository.py` 的 `upsert_extension` 與 `upsert_extension_batch` 中，當寫入 `raw_payload` (透過 `json.dumps(..., ensure_ascii=False)`) 時，如果 payload 內含 lone surrogate 代理字元（例如 `\udcff`），SQLite 寫入會拋出 `UnicodeEncodeError: 'utf-8' codec can't encode character '\udcff' in position 12: surrogates not allowed`，導致整筆記錄寫入失敗。
   - **影響範圍**：[repository.py](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/repository.py#L225-L270)
   - **修復建議**：在進行 `json.dumps` 序列化前，先對 `raw_payload` 進行 `_normalize_value` 規格化清洗，替換掉不合規的代理字元。

2. **測試 Mock 缺陷導致紅燈**
   - **問題描述**：測試檔案（如 `tests/test_pipeline.py` 與 `tests/test_guardrails_and_regressions.py`）中 mock 了 `get_with_retry`，定義的 `mock_get` 只接收 `(url: str, policy: Any)`。然而，`market_crawler/pipeline.py` 實際呼叫為 `get_with_retry(..., session=session)`，導致測試執行時拋出 `TypeError: mock_get() got an unexpected keyword argument 'session'`。
   - **影響範圍**：[test_pipeline.py](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/tests/test_pipeline.py) 與 [test_guardrails_and_regressions.py](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/tests/test_guardrails_and_regressions.py)
   - **修復建議**：將所有測試中的 `mock_get` 簽名修改為支援 `session` 或 `**kwargs` 關鍵字參數，確保與實際被 mock 的函式相容。

### P2_MEDIUM
1. **模組複雜度過高 (C901)**
   - **問題描述**：`market_crawler/pipeline.py` 中的 `make_fetcher` 與 `fetch_and_ingest_day` 因為內嵌多個閉包與多層異常擷取邏輯，使得 Ruff 靜態檢查發出 C901 警告（複雜度大於 10）。
   - **影響範圍**：[pipeline.py](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/pipeline.py#L304-L497)
   - **修復建議**：
     - 將 `make_fetcher` 重構為一個實作了 `__call__` 與 `close()` 的類別 `MarketFetcher`。
     - 將 `fetch_and_ingest_day` 中載入有價證券主檔、初始化資料庫連線等流程重構抽離成獨立的 helper 函式，如 `_resolve_security_master` 與 `_resolve_connection`，降低單一函式的認知負擔。

2. **未定義名稱與未使用之匯入 (F821 / F401)**
   - **問題描述**：`market_crawler/pipeline.py` 中有對 `requests` 與 `Any` 的引用，但未在檔案頂端導入，導致 Ruff 靜態檢查拋出 `F821 Undefined name 'requests'` 與 `F821 Undefined name 'Any'`；同時，檔案中引入了 `urllib3` 但未使用，拋出 `F401 'urllib3' imported but unused`。
   - **影響範圍**：[pipeline.py](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/pipeline.py)
   - **修復建議**：在 `pipeline.py` 頂部補上 `import requests` 與 `from typing import Any`，並移除未使用的 `import urllib3`。

### P3_LOW
1. **程式碼格式微調 (W293)**
   - **問題描述**：多處程式碼與測試檔案中，空行含有空白符號，導致 Ruff W293 警告。
   - **影響範圍**：`market_crawler/pipeline.py` 與 `tests/test_repository.py` 等。
   - **修復建議**：使用 `ruff format` 或者是 `ruff check --fix` 進行自動化程式碼格式修整。

## 修復規劃
- 建立專用分支：已切換至 `review-fix/comprehensive-code-review-fixes` 分支。
- 建立 OpenSpec change 並撰寫提案與設計。
- 逐一修復：
  1. 於 `market_crawler/pipeline.py` 頂端補齊 `requests` 與 `typing.Any`，移除 `urllib3`。
  2. 重構 `make_fetcher` 為 `MarketFetcher` 類別，並拆分 `fetch_and_ingest_day` 為複數個 helper 函式以滿足 C901。
  3. 於 `market_crawler/repository.py` 的 `upsert_extension` 及 `upsert_extension_batch` 中對 `raw_payload` 進行 `_normalize_value` 處理，防範 lone surrogate 造成的寫入錯誤。
  4. 修改測試檔案中的 `mock_get` 簽名，新增 `session=None` 關鍵字參數支援。
- 執行所有測試並進行程式碼格式化。

## 待確認事項
- 無。

## OpenSpec 指令清單
```bash
openspec new change "comprehensive-code-review-fixes"
openspec instructions proposal --change "comprehensive-code-review-fixes"
openspec instructions design --change "comprehensive-code-review-fixes"
openspec instructions tasks --change "comprehensive-code-review-fixes"
```
