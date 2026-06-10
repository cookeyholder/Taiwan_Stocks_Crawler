## ADDED Requirements

### Requirement: CLI 提供 --today 快速語法
系統 SHALL 支援 `--today` 旗標，等同於將 `--start` 與 `--end` 皆設為今日日期（`datetime.date.today()` in local time）。

#### Scenario: --today 執行爬蟲
- **WHEN** 使用者執行 `twse-crawler --today`
- **THEN** 系統以今日日期為目標執行單日爬蟲，不需額外指定 `--start`/`--end`

#### Scenario: --today 與 --start/--end 互斥
- **WHEN** 使用者同時指定 `--today` 與 `--start`
- **THEN** 系統 SHALL 拒絕執行並顯示錯誤訊息，說明兩者不可同時使用

### Requirement: CLI 提供 --date 單日語法
系統 SHALL 支援 `--date YYYY-MM-DD` 引數，等同於 `--start YYYY-MM-DD --end YYYY-MM-DD`。

#### Scenario: --date 執行特定單日
- **WHEN** 使用者執行 `twse-crawler --date 2026-03-17`
- **THEN** 系統以 2026-03-17 為目標執行單日爬蟲

#### Scenario: --date 格式錯誤
- **WHEN** 使用者執行 `twse-crawler --date 20260317`（非 YYYY-MM-DD 格式）
- **THEN** 系統 SHALL 拒絕執行並顯示格式錯誤訊息

### Requirement: CLI 提供 --backfill N 回補語法
系統 SHALL 支援 `--backfill N` 引數，以今日為終點，回溯 N 個日曆天為起點，執行區間爬蟲。

#### Scenario: --backfill 30 回補 30 天
- **WHEN** 使用者執行 `twse-crawler --backfill 30`
- **THEN** 系統以 `today - 30 days` 至 `today` 為區間執行爬蟲

#### Scenario: --backfill 非正整數
- **WHEN** 使用者執行 `twse-crawler --backfill 0` 或 `--backfill -1`
- **THEN** 系統 SHALL 拒絕執行並顯示錯誤訊息

### Requirement: pyproject.toml 提供 twse-crawler entrypoint
系統 SHALL 在 `pyproject.toml` 的 `[project.scripts]` 中聲明 `twse-crawler = "market_crawler.pipeline:main"`，使 `uv sync` 後可直接使用 `twse-crawler` 指令。

#### Scenario: uv sync 後可直接執行
- **WHEN** 使用者執行 `uv sync --all-extras` 後執行 `twse-crawler --help`
- **THEN** 系統顯示 CLI 說明，並列出所有支援的引數（`--today`、`--date`、`--start`、`--end`、`--backfill`、`--db`、`--holidays`、`--no-auto-holidays`、`--timeout`、`--retries`）

### Requirement: main() 函式封裝 CLI 邏輯
系統 SHALL 提供 `market_crawler.pipeline.main()` 函式，封裝全部 argparse 解析與執行邏輯，同時保留 `if __name__ == "__main__": main()` 以相容 `python -m` 執行方式。

#### Scenario: python -m 執行方式持續可用
- **WHEN** 使用者執行 `python -m market_crawler.pipeline --today`
- **THEN** 系統正常執行，無任何錯誤
