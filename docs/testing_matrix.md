# 測試矩陣（Testing Matrix）

本文件描述測試分類、案例數、涵蓋風險與對應需求。

## 測試檔案總覽

| 測試檔案 | 案例數 | 主要涵蓋 |
|---------|--------|---------|
| `test_parsers.py` | ~35 | TWSE/TPEX CSV 與 JSON 解析、欄位映射、編碼回退 |
| `test_normalize.py` | ~25 | normalize_record()、民國年轉換、數值清理 |
| `test_classification.py` | ~20 | instrument_type 分類、CFICode、StockID.csv 優先順序 |
| `test_orchestrator.py` | ~20 | 日期分類（trading/holiday/weekend/market_closed/source_error）、區間批次 |
| `test_quality_and_encoding.py` | ~15 | BatchQualityReport 計數、as_dict()、encoding fallback |
| `test_repository.py` | ~15 | SQLite schema、upsert 冪等性、ON CONFLICT DO UPDATE |
| `test_pipeline.py` | 36 | TWSE 日期路由、CLI 引數解析、自動假日抓取 |
| `test_guardrails_and_regressions.py` | 26 | 防呆測試、整合測試 |
| `test_http_client.py` | ~8 | retry/backoff、5xx 處理 |
| `test_fetch_security_master.py` | ~8 | StockID.csv 載入 |
| `test_verification_samples.py` | 3 (skipped) | 真實樣本煙霧測試 |

**合計：約 181 通過 + 2 跳過（截至 2026-03-21）**

---

## 測試類型分類

### 單元測試

| 測試類別 | 案例說明 |
|---------|---------|
| Parser 正常路徑 | TWSE CSV 正確解析欄位、日期轉換 |
| Parser 編碼回退 | cp950 → big5hkscs → utf-8-sig → utf-8 |
| Parser 錯誤收集 | 格式異常列產生 ParseError，不中斷解析 |
| normalize_record() | 民國年 → ISO 西元、數值清理、symbol_raw 保留 |
| classify_instrument() | StockID.csv 優先、CFICode 次之、pattern fallback |
| classify_day() | trading / holiday / weekend / market_closed / source_error |
| BatchQualityReport | 計數、source_error_count property、as_dict() |

### 整合測試

| 測試類別 | 案例說明 |
|---------|---------|
| `--today` 完整流程 | CLI 解析 → 假日抓取 → run() → 報告輸出 |
| SQLite 冪等寫入 | 重複匯入同一日不產生重複資料 |
| 日期路由整合 | today → JSON API；historical → CSV API |

### 防呆測試（Guardrails）

| 測試案例 | 風險 |
|---------|------|
| TWSE CSV URL YYYYMMDD 格式 | 日期格式錯誤導致 API 404 |
| market_closed 不計入 source_error | 語意混淆 |
| 已知假日短路 → holiday | 假日被誤標 market_closed |
| 週末不受 api_date 影響 | 週末被誤標 market_closed |
| fetch_and_ingest_day 回傳 None | market_closed 訊號丟失 |
| --backfill 負數/0 → 報錯 | 誤操作 |
| --today + --start 互斥 → 報錯 | 引數衝突 |
| --date 非 YYYY-MM-DD 格式 → 報錯 | 日期解析問題（Python 3.11+ fromisoformat 接受 YYYYMMDD） |
| 空/無效 TWSE 檔案 → ParseError | 格式變動不崩潰 |
| 缺必要欄位 → ParseError | 欄位映射失敗不崩潰 |
| 數值 "--" 正確轉 None | 極端值處理 |
| ="0050" 代號不被截掉 | Excel quote 前置符號 |

---

## 未涵蓋的已知限制

- **多年度假日**：`holidaySchedule` API 只回傳當年，歷史回補無法自動認定假日
- **TPEX 歷史路由**：TPEX CSV API 已有日期參數，不需路由，未另立測試
- **真實網路整合測試**：`test_verification_samples.py` 中 3 個 smoke test 由於網路依賴預設跳過

---

## 執行測試的指令

```bash
# 完整測試套件
.venv/bin/python -m pytest tests/ -q

# 僅防呆測試
.venv/bin/python -m pytest tests/test_guardrails_and_regressions.py -q

# 含覆蓋率
.venv/bin/python -m pytest tests/ --cov=market_crawler --cov-report=term-missing -q
```
