## Context

目前的爬蟲（`market_crawler/pipeline.py`）已有基礎的 argparse 區段（`--start`、`--end`、`--db`、`--holidays`、`--timeout`、`--retries`），但缺乏：

1. `main()` 函式與 `pyproject.toml` entrypoint，無法作為系統指令使用
2. `--today`、`--date`、`--backfill N` 等常見使用情境的快速語法
3. TWSE JSON API（`STOCK_DAY_ALL`）只提供當日資料，缺少歷史回補能力
4. `classify_day()` 尚無法區分颱風假等臨時休市與真正的資料來源錯誤

相關模組：`pipeline.py`、`orchestrator.py`、`quality.py`、`pyproject.toml`

## Goals / Non-Goals

**Goals:**
- 讓 `twse-crawler --today` 成為一次排程呼叫的標準形式
- 支援 `--date YYYY-MM-DD` 單日、`--start/--end` 區間、`--backfill N` 三種補充模式
- 歷史日期（< today）自動改用 TWSE CSV API（有日期參數）
- 工作日 + API 回應正常 + 回傳日期不符 → 標 `market_closed`，不計入 `source_error`
- 假日清單改為 best-effort（自動抓取 `holidaySchedule` API）

**Non-Goals:**
- 不改動 TPEX 的資料來源（已有日期參數，維持原 CSV 路徑）
- 不加 Web UI 或 TUI
- 不對外暴露 API endpoint
- 不支援多年度假日歷史查詢（API 限制）

## Decisions

### 決策一：CLI 入口改用 `main()` 函式而非直接執行模組

**選擇**：將 `if __name__ == "__main__":` 的 argparse 邏輯重構為 `def main()` 函式，並在 `pyproject.toml` 中的 `[project.scripts]` 聲明 entrypoint。

**理由**：entrypoint 是 Python 套件的標準做法，安裝後可直接執行 `twse-crawler`，與 cron 排程整合最為簡單。`click` 等框架雖然功能更完整，但本專案是單人使用的小型工具，標準 `argparse` 加 `main()` 已足夠，不引入新依賴。

**拒絕方案**：`click`（多餘依賴）、`typer`（多餘依賴）、shell wrapper script（難以隨 pip/uv 安裝）

### 決策二：TWSE 資料來源依日期路由

**選擇**：在 `pipeline.py` 中的 `fetch_and_ingest_day()` 加入路由邏輯：
- `date == today` → `_fetch_and_parse_twse()`（JSON API）
- `date < today` → `_fetch_and_parse_twse_csv(date)`（CSV API，含日期參數）

**理由**：TWSE JSON API（`STOCK_DAY_ALL`）不支援日期參數，只回傳最近一筆交易日資料；若歷史回補也走 JSON，會造成日期偏移錯誤。CSV API（`MI_INDEX?date=YYYYMMDD&type=ALL`）則支援任意歷史日期。

**注意**：全市場 JSON（`STOCK_DAY_ALL`）也只有最新一筆，因此「今天」的定義以 `datetime.date.today()` 決定，不依使用者本地時區。

### 決策三：`market_closed` 作為獨立日期分類

**選擇**：在 `orchestrator.py` 的 `classify_day()` 中增加 `"market_closed"` 回傳值。

**判斷條件**：
1. 非週末（weekday）
2. `--holidays` 清單中無此日（或清單為空）
3. API 有正常回應（非網路錯誤）
4. 回傳的資料日期 ≠ 請求日期（表示當日無交易、API 回退到最近一筆）

**理由**：颱風假、選舉補假、臨時閉市均不在 `holidaySchedule` API 中，若攔截後標 `source_error` 會讓品質報告失真。`market_closed` 語意清晰：有請求、有回應，只是當天真的沒開市。

### 決策四：假日清單改為 best-effort，`--no-auto-holidays` 作為 escape hatch

**選擇**：`main()` 或 `run_date_range_ingestion()` 啟動時自動嘗試呼叫 `holidaySchedule` API；失敗時發出 warning 並繼續（不阻斷作業）。保留 `--no-auto-holidays` 旗標讓使用者停用。

**理由**：API 只回傳當年國定假日，歷史年度無法查詢。透過 `market_closed` 語意已可涵蓋未知休市；假日清單只是輔助，不強制要求。

## Risks / Trade-offs

- **排程時間點問題**：若 cron 在收盤前跑 `--today`，JSON API 可能回傳前一日資料 → 日期不符 → 標 `market_closed`。建議排程設 18:00 後，或加文件說明。不在本 change 內自動重試。

- **CSV API 格式變動**：TWSE CSV API 格式偶有調整，`parse_twse_daily_close()` 需要維護。若格式大幅變動會導致 `source_error`。緩解：測試覆蓋 CSV parser + `test_guardrails_and_regressions.py` 防呆測試。

- **`market_closed` 判斷可能誤判**：若 API 本身 bug 導致回傳舊日期，也會被標 `market_closed` 而非 `source_error`。業務上此偏差可接受，嚴格必要時可人工查 DB 確認。

- **假日 API 只有當年**：歷史回補時無法自動認定哪些日期是國定假日 → 全標 `market_closed`（已知 workaround：進行歷史回補前手動提供 `--holidays` 清單）。

## Migration Plan

1. 新增 `market_closed_count` 欄位至 `BatchQualityReport`（向後相容，不改既有欄位）
2. 新增 `classify_day()` 的 `market_closed` 路徑（不改動現有 `holiday`/`weekend`/`trading`/`source_error`）
3. 新增 `_fetch_and_parse_twse_csv()` 後，修改 `fetch_and_ingest_day()` 路由邏輯
4. 重構 CLI：`__main__` 區段保留，同時提供 `main()` 函式，不 breaking existing `python -m market_crawler.pipeline` 用法
5. `pyproject.toml` 加 `[project.scripts]`，`uv sync` 後即可使用 `twse-crawler`
6. 所有步驟皆有對應測試，採 TDD 流程

**Rollback**：所有改動均向後相容，若有問題 `git revert` 即可，不影響 DB schema。

## Open Questions

- 無。資料來源路由邏輯與分類語意已在 explore 模式完整確認。
