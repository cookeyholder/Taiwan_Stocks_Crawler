# 臺股成交資訊收集

臺灣股市每日收盤行情自動抓取與入庫系統。支援 TWSE（臺灣證券交易所）與 TPEX（證券櫃買中心）資料，提供命令列工具 `twse-crawler`，可排程執行、回補歷史資料並自動辨識例假日與休市日。

## 功能特色

- **雙來源支援**：自動抓取 TWSE 與 TPEX 每日收盤行情
- **智慧路由**：今日資料走 JSON API，歷史資料走 CSV API（日期前後自動切換）
- **休市偵測**：自動辨識 `holiday`、`weekend`、`market_closed`（颱風假等臨時休市）、`source_error`
- **自動假日抓取**：每次執行自動從 TWSE `holidaySchedule` API 取得當年假日清單
- **冪等寫入**：重複匯入同一日期不會產生重複資料（SQLite `UPSERT`）
- **編碼容錯**：cp950 → big5hkscs → utf-8-sig → utf-8 依序嘗試解碼
- **品質報告**：每批次執行後輸出 `BatchQualityReport`（trading／source_error／market_closed 計數），holiday／weekend 會保留在 outcomes 明細

## 快速開始

### 環境需求

- Python 3.13+
- [uv](https://docs.astral.sh/uv/) 套件管理工具
- Docker Desktop（使用 Dev Container 時）

### 安裝

**建議：使用 Dev Container（含 Python 3.13、uv、pre-commit）**

1. 安裝 VS Code 與 [Dev Containers 擴充套件](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)
2. 開啟此資料夾，執行 `Dev Containers: Reopen in Container`
3. 同步相依套件：

```bash
uv sync --all-extras
```

**本機安裝（替代方案）**

```bash
pip install uv
uv sync --all-extras
```

安裝後可直接執行 `twse-crawler --help` 確認安裝成功。

## 命令列工具

`twse-crawler` 提供四種執行模式，互斥擇一使用。

### 抓取今日資料（排程推薦）

建議在每日 **18:00 後**執行，確保收盤資料已更新。

```bash
twse-crawler --today
twse-crawler --today --db /data/market.db
```

或等效使用：

```bash
python -m market_crawler.pipeline --today
```

### 抓取指定日期

```bash
twse-crawler --date 2026-03-17
```

注意：`--date` 格式必須為 `YYYY-MM-DD`，不接受 `YYYYMMDD`。

### 回補近 N 天資料

```bash
twse-crawler --backfill 30    # 從今日往前 30 天
twse-crawler --backfill 90    # 初次建立歷史資料
```

### 指定任意日期區間

```bash
twse-crawler --start 2026-01-01 --end 2026-03-31
```

### 共用選項

| 選項 | 說明 | 預設值 |
|------|------|--------|
| `--db PATH` | SQLite 資料庫路徑 | `market_data.db` |
| `--holidays DATE …` | 手動補充假日（可多個，ISO 格式） | 無 |
| `--no-auto-holidays` | 停用自動假日抓取 | 否 |
| `--timeout SECS` | HTTP 請求逾時秒數 | `30` |
| `--retries N` | HTTP 重試次數 | `3` |

### 排程範例（cron）

```cron
# 每日 18:10 自動抓取今日收盤資料
10 18 * * 1-5 /path/to/.venv/bin/twse-crawler --today --db /data/market.db
```

## 資料來源路由

| 請求日期 | TWSE 來源 | TPEX 來源 |
|----------|----------|----------|
| 今日 | JSON API（`openapi.twse.com.tw`） | JSON API（`dailyQuotes?response=json`） |
| 歷史日期 | CSV API（`MI_INDEX?response=csv&date=YYYYMMDD`） | JSON API（`dailyQuotes?response=json`） |

> TWSE JSON API 固定回傳最新交易日資料，不接受日期參數。若回傳日期與請求日期不符，系統自動標記為 `market_closed`。

## 架構概述

```
market_crawler/
├── http_client.py        # HTTP 抓取（RetryPolicy、get_with_retry）
├── parsers.py            # TWSE / TPEX 原始資料解析，回傳 ParseResult(records, errors)
├── encoding.py           # 編碼 fallback：cp950 → big5hkscs → utf-8-sig → utf-8
├── normalize.py          # 統一格式化（民國年轉西元、數值清理）→ CanonicalQuote
├── classification.py     # 商品分類（override → StockID.csv → CFICode → 樣式規則）
├── repository.py         # SQLiteMarketRepository（market_quote_core / _extension）
├── orchestrator.py       # 批次匯入流程、classify_day()、BatchQualityReport
├── pipeline.py           # 端對端整合、make_fetcher()、fetch_and_ingest_day()、main()
├── quality.py            # BatchQualityReport 資料類別
├── models.py             # CanonicalQuote、ParseError 資料類別
└── fetch_security_master.py  # 下載最新 StockID.csv
```

### 處理流程

```
HTTP 抓取 (http_client)
    ↓
原始解析 (parsers)  ←── 收集 ParseError，不中斷流程
    ↓
正規化 (normalize)  ←── 民國年→西元、數值清理、symbol_raw & symbol_clean
    ↓
商品分類 (classification)  ←── StockID.csv 優先，fallback 樣式規則
    ↓
SQLite 寫入 (repository)  ←── UPSERT，重複匯入不產生重複資料
    ↓
品質報告 (orchestrator)  ←── trading/holiday/market_closed/source_error 計數
```

## 資料庫 Schema

資料分兩張表儲存：

**`market_quote_core`** — 標準化報價欄位（主鍵：`trade_date + market_code + symbol_clean`）

| 欄位 | 說明 |
|------|------|
| `trade_date` | 西元格式交易日（`YYYY-MM-DD`） |
| `market_code` | `TWSE` 或 `TPEX` |
| `symbol_clean` | 標準股票代號（去除 Excel `="..."` 格式） |
| `symbol_raw` | 原始代號（保留 `="0050"` 格式） |
| `instrument_type` | 商品類型（`stock`、`etf`、`warrant` 等） |
| `close_price` | 收盤價 |
| `volume_shares` | 成交股數 |
| `turnover_value` | 成交金額 |

**`market_quote_extension`** — 原始 payload 與附加中繼資料（外鍵關聯 `market_quote_core`）

## 休市分類邏輯

`classify_day()` 依下列優先順序判斷：

1. **`holiday`** — 日期在已知假日集合中（國定假日或手動指定）
2. **`weekend`** — 星期六或日
3. **`market_closed`** — 工作日但 API 回傳日期與請求日期不符（颱風假等臨時休市）
4. **`trading`** — 正常交易日

`market_closed` **不計入** `source_error_count`，兩者分開統計。

## 開發

### 執行測試

```bash
make test              # 所有測試
make test-cov          # 含覆蓋率報告
uv run pytest tests/test_pipeline.py -v   # 單一測試檔
```

### Lint 與格式化

```bash
make check             # ruff check（只檢查）
make fix               # ruff check --fix
make format            # ruff format
make pre-commit        # 執行全部 pre-commit hooks
```

### 測試結構

| 測試檔案 | 涵蓋範圍 |
|----------|---------|
| `test_parsers.py` | TWSE / TPEX 解析邏輯、ParseError 收集 |
| `test_normalize.py` | 民國年轉換、數值清理、symbol_clean |
| `test_classification.py` | 商品分類優先順序 |
| `test_repository.py` | SQLite UPSERT 冪等性 |
| `test_orchestrator.py` | classify_day()、BatchQualityReport |
| `test_pipeline.py` | 日期路由、CLI 引數、假日抓取 |
| `test_guardrails_and_regressions.py` | 防呆測試、端對端整合測試 |
| `test_quality_and_encoding.py` | 品質報告計數、編碼 fallback |

## 貢獻

詳見 [CONTRIBUTING.md](CONTRIBUTING.md)。

本專案採用：
- **TDD 流程**：先寫 failing test，再實作
- **Conventional Commits**：`feat:`、`fix:`、`test:`、`docs:`、`refactor:`
- **高頻提交**：每完成一個子任務即提交，確保每次提交都通過所有測試

## 授權

本專案為個人開發學習用途。
