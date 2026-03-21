## 1. market_closed 分類（orchestrator + quality）

- [ ] 1.1 在 `tests/test_orchestrator.py` 新增 `market_closed` failing tests：工作日 + API 正常回應 + 日期不符 → `market_closed`；網路錯誤維持 `source_error`；已知假日維持 `holiday`；日期符合維持 `trading`；週末維持 `weekend`（至少 5 個案例）
- [ ] 1.2 在 `market_crawler/orchestrator.py` 的 `classify_day()` 加入 `market_closed` 回傳路徑（判斷條件：非週末 + 非已知假日 + API 有回應 + trade_date ≠ requested_date）
- [ ] 1.3 在 `tests/test_quality_and_encoding.py`（或新增 `tests/test_quality.py`）新增 failing tests：`BatchQualityReport` 有 `market_closed_count` 欄位；`market_closed` 不計入 `source_error_count`；兩者各自獨立計數（至少 3 個案例）
- [ ] 1.4 在 `market_crawler/quality.py` 的 `BatchQualityReport` 加入 `market_closed_count: int = 0` 欄位，並更新彙總邏輯
- [ ] 1.5 確認所有第 1 組測試通過，執行 `uv run pytest tests/test_orchestrator.py tests/test_quality_and_encoding.py -q` 全數通過
- [ ] 1.6 commit：`feat(orchestrator): 新增 market_closed 日期分類與 BatchQualityReport 計數欄位`

驗收標準：
- `classify_day()` 能正確區分 `market_closed` / `source_error` / `holiday` / `trading` / `weekend`
- `BatchQualityReport.market_closed_count` 正確計數，不混入 `source_error_count`
- 所有第 1 組測試通過，無 skip 或 xfail

## 2. TWSE 歷史 CSV API 路由

- [ ] 2.1 在 `tests/test_pipeline.py`（或新增）新增 failing tests：`date == today` → mock 呼叫 JSON API；`date < today` → mock 呼叫 CSV API；`market == "tpex"` → 不路由（至少 4 個案例）
- [ ] 2.2 在 `market_crawler/pipeline.py` 新增 `_fetch_and_parse_twse_csv(date_str: str, policy: RetryPolicy) -> FetchResult` 函式，以 `MI_INDEX?response=csv&date=YYYYMMDD&type=ALL` 下載並呼叫 `parse_twse_daily_close()`
- [ ] 2.3 修改 `fetch_and_ingest_day()` 加入路由判斷：`if market == "twse" and date < today: _fetch_and_parse_twse_csv(date)` else 原 JSON 路徑
- [ ] 2.4 確認 TWSE CSV URL 格式正確（日期轉 YYYYMMDD）且 encoding fallback 邏輯沿用 `market_crawler.encoding`
- [ ] 2.5 確認所有第 2 組測試通過，並確認既有 tests 無退步
- [ ] 2.6 commit：`feat(pipeline): 新增 TWSE 歷史資料 CSV 路由，歷史日期改走 MI_INDEX CSV API`

驗收標準：
- `date == today` 走 JSON、`date < today` 走 CSV，TPEX 維持不變
- CSV API URL 中的日期格式為 `YYYYMMDD`
- 現有所有測試通過，無退步

## 3. CLI 強化（--today / --date / --backfill / main()）

- [ ] 3.1 在 `tests/test_pipeline.py` 新增 CLI failing tests：`--today` 等同今日區間；`--date 2026-03-17` 等同單日區間；`--backfill 30` 等同 30 天區間；`--today` 與 `--start` 互斥報錯；`--date` 格式非 YYYY-MM-DD 報錯；`--backfill 0` 報錯（至少 6 個案例）
- [ ] 3.2 將 `market_crawler/pipeline.py` 中 `if __name__ == "__main__":` 的 argparse 邏輯重構為 `def main()` 函式，`__main__` 區段改為 `main()` 呼叫
- [ ] 3.3 在 `main()` 的 argparse 加入 `--today`（`store_true`）、`--date`（字串）、`--backfill`（整數）引數
- [ ] 3.4 實作互斥驗證：`--today`/`--date`/`--backfill`/`--start`/`--end` 彼此互斥（同時指定時 argparse 報錯）
- [ ] 3.5 在 `main()` 中將 `--today`、`--date`、`--backfill` 解析為 `(start_date, end_date)` 後傳入執行邏輯
- [ ] 3.6 確認所有第 3 組測試通過，並確認 `python -m market_crawler.pipeline --help` 可正常執行
- [ ] 3.7 commit：`feat(pipeline): 新增 --today/--date/--backfill CLI 引數並重構為 main() 函式`

驗收標準：
- `twse-crawler --today`、`--date`、`--backfill` 均可正確解析並執行
- 引數互斥邏輯正確，非法組合有清晰錯誤訊息
- `--help` 顯示所有支援引數
- `python -m market_crawler.pipeline --help` 持續可用

## 4. pyproject.toml entrypoint

- [ ] 4.1 在 `pyproject.toml` 新增 `[project.scripts]` section：`twse-crawler = "market_crawler.pipeline:main"`
- [ ] 4.2 執行 `uv sync --all-extras` 確認安裝無誤
- [ ] 4.3 執行 `twse-crawler --help` 確認 entrypoint 可用
- [ ] 4.4 commit：`feat(pyproject): 新增 twse-crawler CLI entrypoint 至 pyproject.toml`

驗收標準：
- `uv sync` 後 `twse-crawler --help` 可正常執行
- entrypoint 指向 `market_crawler.pipeline:main`

## 5. 自動假日抓取（auto-holiday-fetch）

- [ ] 5.1 在 `tests/test_pipeline.py` 新增 failing tests：自動抓取成功 → 假日加入集合；API 失敗 → warning 且繼續；`--no-auto-holidays` → 不呼叫 API；`--holidays` 與自動抓取結果合併（至少 4 個案例）
- [ ] 5.2 在 `market_crawler/pipeline.py` 新增 `_fetch_twse_holidays() -> set[datetime.date]` 函式，呼叫 `holidaySchedule` API，失敗時 log warning 並回傳空集合
- [ ] 5.3 在 `main()` 中整合自動假日抓取：若未指定 `--no-auto-holidays`，在啟動時呼叫 `_fetch_twse_holidays()` 並與 `--holidays` 清單合併
- [ ] 5.4 在 argparse 加入 `--no-auto-holidays` 旗標（`store_true`）
- [ ] 5.5 確認所有第 5 組測試通過，並確認與 orchestrator 整合正確
- [ ] 5.6 commit：`feat(pipeline): 新增自動假日抓取（holidaySchedule API）與 --no-auto-holidays 旗標`

驗收標準：
- 自動假日抓取成功時假日被納入 classify_day 判斷
- API 失敗不中斷作業，只發出 warning
- `--no-auto-holidays` 可正確停用自動抓取
- 手動 `--holidays` 與自動結果正確合併

## 6. 整合測試與防呆測試

- [ ] 6.1 在 `tests/test_guardrails_and_regressions.py` 新增至少 5 個防呆測試：日期前後顛倒（`--start > --end`）報錯；`--backfill` 負數報錯；假日集合空時工作日 API 回傳舊日期標 `market_closed`；TWSE CSV API URL 含正確 YYYYMMDD；`market_closed` 不出現在 `source_error_count`
- [ ] 6.2 新增至少 1 個端對端整合測試（mock 全部 HTTP），驗證 `--today` 完整流程：CLI 解析 → 路由判斷 → 假日抓取 → ingest → BatchQualityReport 輸出正確計數
- [ ] 6.3 執行完整測試套件 `uv run pytest tests -q`，確認全數通過（目標：所有測試通過，覆蓋率 ≥ 90%）
- [ ] 6.4 執行 `uv run ruff check market_crawler tests`，確認無 lint 錯誤
- [ ] 6.5 commit：`test(guardrails): 新增防呆測試與端對端整合測試`

驗收標準：
- 測試案例總數 ≥ 原有數量 + 25 個新增案例（防呆測試 ≥ 10 個）
- 完整測試套件通過，無 skip 或 xfail
- ruff check 無錯誤

## 7. 文件更新

- [ ] 7.1 更新 `docs/debugging_guide.md`：新增 `market_closed` 的說明、颱風假判斷邏輯、`--no-auto-holidays` 使用時機
- [ ] 7.2 更新（或新增）`docs/cli_usage.md`：列出所有 CLI 引數範例，包含排程設定建議（cron 18:00 後）
- [ ] 7.3 更新 `docs/testing_matrix.md`：加入 CLI、路由、market_closed、假日抓取等新測試案例的矩陣描述
- [ ] 7.4 commit：`docs: 更新 CLI 使用說明、debugging guide 與 testing matrix`

驗收標準：
- `docs/debugging_guide.md` 包含 `market_closed` 判斷條件說明
- `docs/cli_usage.md`（或 README）包含所有新 CLI 引數的使用範例
- `docs/testing_matrix.md` 反映最新測試案例分類
