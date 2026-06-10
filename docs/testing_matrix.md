# 測試矩陣（Testing Matrix）

本文件描述測試分類、案例數與執行方式。

---

## Rust 測試（主力，29 項）

### 測試檔案總覽

| 模組 | 案例數 | 主要涵蓋 |
|------|--------|---------|
| `analysis::tests` | 8 | SMA、EMA、Bollinger、RSI、MACD 純函式計算 |
| `db::tests` | 4 | SQLite schema migration、upsert 冪等性 |
| `fetcher::tests` | 6 | TWSE JSON/CSV 解析、TPEX JSON 解析、Big5 解碼 |
| `pipeline::tests` | 11 | TWSE/TPEX 路由、fetch_and_ingest_day、假日偵測 |

**合計：29 項（截至 2026-06-10）**

### 執行方式

```bash
# 完整測試
cargo test -p stock-core

# 特定模組
cargo test -p stock-core -- analysis
cargo test -p stock-core -- db
cargo test -p stock-core -- fetcher
cargo test -p stock-core -- pipeline

# Lint
cargo clippy -p stock-core
```

---

## 歷史參考：Python 測試（已停止維護）

Python 測試套件保留於 `tests/` 目錄，共約 **181 項**（截至 2026-03-21）。

### 原測試檔案

| 測試檔案 | 案例數 | 主要涵蓋 |
|---------|--------|---------|
| `test_parsers.py` | ~35 | TWSE/TPEX CSV 與 JSON 解析、編碼回退 |
| `test_normalize.py` | ~25 | 民國年轉換、數值清理 |
| `test_classification.py` | ~20 | instrument_type 分類 |
| `test_orchestrator.py` | ~20 | 日期分類、區間批次 |
| `test_quality_and_encoding.py` | ~15 | BatchQualityReport、encoding |
| `test_repository.py` | ~15 | SQLite upsert 冪等性 |
| `test_pipeline.py` | 36 | 日期路由、CLI 引數 |
| `test_guardrails_and_regressions.py` | 26 | 防呆測試 |
| `test_http_client.py` | ~8 | retry/backoff |
| `test_fetch_security_master.py` | ~8 | StockID.csv 載入 |
| `test_verification_samples.py` | 3 (skipped) | 煙霧測試 |

### 原執行指令

```bash
.venv/bin/python -m pytest tests/ -q
.venv/bin/python -m pytest tests/ --cov=market_crawler --cov-report=term-missing -q
```
