## Why

目前的爬蟲缺少方便的 CLI 入口點，每次執行需要手動指定 `--start`/`--end`，且歷史資料只能透過 TWSE JSON API（無日期參數），歷史回補因此不可行。此外，颱風假、選舉補假等臨時休市無法與真正的資料來源錯誤區分，影響品質報告的可信度。

## What Changes

- **新增 CLI 指令**：加入 `--today`、`--date`、`--backfill N` 三種快速指定方式，方便排程與回補
- **新增 `pyproject.toml` entrypoint**：讓 `twse-crawler` 成為可直接執行的 CLI 指令
- **新增歷史資料抓取路由**：`date == today` → TWSE JSON API；`date < today` → TWSE CSV API（含日期參數）
- **新增 `market_closed` 日期分類**：工作日 + API 正常回應 + 回傳日期不符請求日期 → 標記為 `market_closed`（而非錯誤）
- **假日改為非必要**：自動嘗試從 TWSE `holidaySchedule` API 抓取當年國定假日；`market_closed` 涵蓋颱風假等無法預知的休市，不再強制要求假日清單

## Capabilities

### New Capabilities

- `cli-entrypoint`：強化 CLI 介面，包含 `--today`、`--date`、`--backfill` 引數與 `main()` 函式，並在 `pyproject.toml` 新增 `[project.scripts]` entrypoint
- `twse-date-routing`：依據請求日期決定使用 TWSE JSON API（當日）或 CSV API（歷史），自動切換資料來源
- `market-closed-classification`：在 orchestrator 新增 `market_closed` 日期類型，於 `BatchQualityReport` 新增對應計數欄位
- `auto-holiday-fetch`：在 pipeline 啟動時自動嘗試從 TWSE `holidaySchedule` API 下載當年假日清單，並提供 `--no-auto-holidays` escape hatch

### Modified Capabilities

## Impact

- `market_crawler/pipeline.py`：新增 `main()` 函式、`--today`/`--date`/`--backfill` CLI 引數、TWSE 歷史 CSV 抓取函式 `_fetch_and_parse_twse_csv()`、日期路由邏輯、自動假日抓取
- `market_crawler/orchestrator.py`：`classify_day()` 回傳值新增 `"market_closed"`
- `market_crawler/quality.py`：`BatchQualityReport` 新增 `market_closed_count` 欄位
- `pyproject.toml`：新增 `[project.scripts]` section
- `tests/test_orchestrator.py`、`tests/test_pipeline.py`：新增對應測試案例
