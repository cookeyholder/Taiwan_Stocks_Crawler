# 測試矩陣

## 摘要

- 測試框架：pytest
- 總案例數：83
- 防呆案例數：10+（空檔、缺欄、異常格式、`="0050"`、`--` 等）
- 覆蓋率（market_crawler）：96%

## 矩陣

| 測試類型 | 主要檔案 | 案例重點 | 風險 |
|---|---|---|---|
| Parser 單元測試 | tests/test_parsers.py | TWSE 區段抽取、TPEX 17 欄、編碼 fallback、短列錯誤 | 來源格式變動導致解析失效 |
| Normalize 單元測試 | tests/test_normalize.py | `symbol_raw`/`symbol_clean`、數值清洗、ROC→AD | 資料清洗錯誤、日期年制混用 |
| 分類單元測試 | tests/test_classification.py | override > section > cfi > ticker fallback | 商品型態誤判 |
| Repository 單元/整合 | tests/test_repository.py | schema、upsert 冪等、跨市場查詢一致性 | 重跑重複、查詢不一致 |
| 流程測試 | tests/test_orchestrator.py | 交易日/週末/臨時休市/來源錯誤 | 排程容錯不足 |
| 防呆與回歸 | tests/test_guardrails_and_regressions.py | 空檔、缺欄、`="0050"` 與 `--` 回歸 | 低級輸入錯誤造成崩潰 |
| 樣本驗證 | tests/test_verification_samples.py | `MI_INDEX_ALL_20260318.csv`、`SQUOTE_AL_1150318.csv` 對映驗證 | 真實樣本不相容 |
| HTTP 韌性 | tests/test_http_client.py | timeout/retry/backoff、5xx 重試、最終失敗 | 外部來源不穩定 |
| 品質與編碼工具 | tests/test_quality_and_encoding.py | quality 報表與編碼替代路徑 | 失敗統計不完整 |

## 對應需求

- 來源解析、正規化、分類、持久化：全覆蓋
- 日期區間分流（交易日/例假日/臨時休市/來源錯誤）：覆蓋
- 冪等寫入：覆蓋
- 樣本檔驗證：覆蓋
