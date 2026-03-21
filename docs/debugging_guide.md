# 排查指南（Debugging Guide）

本文件說明在來源格式變動、資料異常或分類問題時的標準排查流程。

## 1. market_closed 判斷邏輯

### 何時標記為 market_closed？

在整體匯入流程中，`"market_closed"` 主要由 `run_date_range_ingestion()` 根據 fetcher 回傳值判定（由上到下，第一個符合就短路）：

1. **非週末（工作日）**
2. **非已知假日（holidays 集合不含此日）**
3. **`fetch_and_ingest_day()` 回傳 `None`**（而非 `True`/`False`）

`fetch_and_ingest_day()` 回傳 `None` 的觸發條件：
- 使用 JSON API 路徑（今日路由）
- TWSE JSON API 有正常回應
- 但 API 回傳的日期 ≠ 請求日期（表示市場當日未開市，API 退回前一交易日資料）

### market_closed vs source_error 的差異

| 情況 | 分類 |
|------|------|
| 颱風假、選舉補假、臨時閉市（API 有回應但日期不符） | `market_closed` |
| 網路錯誤、逾時、格式解析失敗 | `source_error` |
| 已知國定假日（holidays 集合有此日） | `holiday` |
| 六日 | `weekend` |

`market_closed` **不計入** `source_error_count`，兩者獨立計數。

### 誤判風險

若 TWSE API 本身有 bug 導致回傳舊日期，也會被標 `market_closed` 而非 `source_error`。
業務上此偏差可接受；必要時可直接查 DB 的 `ingested_at` 確認。

## 2. 颱風假判斷流程

颱風假不在 `holidaySchedule` API 中，因此：

1. 當日 TWSE JSON API 仍有回應，但回傳的是前一交易日日期
2. `fetch_and_ingest_day()` 偵測日期不符
3. 回傳 `None`
4. `run_date_range_ingestion()` 看到當日 fetcher 回傳 `None`，將該日標記為 `market_closed`

若要讓颱風假被正確標為 `holiday`（而非 `market_closed`），需在執行前將該日期加入 holidays：

```bash
twse-crawler --today --no-auto-holidays --holidays 2026-07-25
```

## 3. --no-auto-holidays 使用時機

| 情境 | 建議 |
|------|------|
| 離線環境 / CI 環境（無法呼叫外部 API） | `--no-auto-holidays` |
| 歷史回補（`holidaySchedule` 只有當年） | `--no-auto-holidays --holidays 2025-01-01 ...` |
| 一般排程（需正確識別國定假日） | 不使用此旗標（預設啟用自動抓取） |

## 4. 新來源格式變動時的排查流程

### TWSE 格式變動

1. **確認錯誤** — 看 `BatchQualityReport.failure_reasons` 中的錯誤訊息
2. **取得樣本** — 下載 `MI_INDEX?response=csv&date=YYYYMMDD&type=ALL` 原始檔
3. **對照 parser** — 查 `market_crawler/parsers.py` 的 `parse_twse_daily_close()`
4. **更新測試** — 先在 `tests/fixtures/twse_sample.csv` 更新樣本，再修改 parser
5. **驗收** — `uv run pytest tests/test_parsers.py tests/test_guardrails_and_regressions.py`

### TPEX 格式變動

同上，樣本路徑為 `tests/fixtures/tpex_sample.csv`，parser 為 `parse_tpex_daily_close_json()`。

## 5. 資料庫查詢疑難排解

```sql
-- 查看某日各市場筆數
SELECT market_code, count(*) FROM market_quote_core WHERE trade_date = '2026-03-21' GROUP BY 1;

-- 查看 market_closed 比例（orchestrator outcomes 不在 DB，需查 logs）

-- 確認重複匯入未產生重複資料
SELECT trade_date, market_code, symbol_clean, count(*)
FROM market_quote_core
GROUP BY 1, 2, 3 HAVING count(*) > 1;
```
