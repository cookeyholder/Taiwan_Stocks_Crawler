# Tasks: harden-twse-tpex-market-crawler

## 1. Source Parsers
- [x] 1.1 建立 TWSE adapter，僅解析 `每日收盤行情(全部)` 區段。
- [x] 1.2 建立 TPEX adapter，解析 17 欄主資料列。
- [x] 1.3 加入來源編碼處理（cp950 / big5hkscs）。

## 2. Canonical Normalize
- [x] 2.1 建立 canonical schema（core + extension）。
- [x] 2.2 實作 `symbol_raw` / `symbol_clean` 雙欄。
- [x] 2.3 實作數值清洗（逗號、`--`、空值）。
- [x] 2.4 實作年制統一（ROC → AD，DB 僅存西元年）。

## 3. Classification
- [x] 3.1 匯入 ISIN 清單（上市/上櫃）作為 `security_master`。
- [x] 3.2 實作分類優先序：override > section > cfi > ticker pattern > other。
- [x] 3.3 依《中華民國證券市場編碼原則》補齊 ticker fallback 規則。

## 4. Persistence Layer
- [x] 4.1 定義 repository 介面與資料模型。
- [x] 4.2 完成 SQLite adapter。
- [x] 4.3 設計可攜 SQL 寫法（避免方言滲透）。
- [x] 4.4 實作冪等寫入（同日同市場同代號不重複）。

## 5. Resilience & Quality
- [x] 5.1 加入 HTTP timeout/retry/backoff。
- [x] 5.2 加入 parser 欄位驗證與錯誤記錄。
- [x] 5.3 產出批次品質報表（成功/失敗筆數與原因）。

## 6. Testing Strategy (TDD + pytest)
- [x] 6.1 先寫測試再實作（Red → Green → Refactor），每個子任務需有對應 failing test。
- [x] 6.2 建立 pytest fixtures（來源檔樣本、日期區間、DB 測試資料）。
- [x] 6.3 建立 parser 單元測試（TWSE 區段抽取、TPEX 17 欄、cp950/big5hkscs 編碼）。
- [x] 6.4 建立 normalize 單元測試（`symbol_raw`/`symbol_clean`、數值清洗、ROC→AD）。
- [x] 6.5 建立分類單元測試（override > section > cfi > ticker pattern > other）。
- [x] 6.6 建立 persistence 單元/整合測試（冪等寫入、唯一鍵、跨查詢一致性）。
- [x] 6.7 建立日期區間流程測試（交易日、週末、颱風假/臨時休市、來源暫時失敗）。
- [x] 6.8 建立防呆測試（空檔案、欄位缺失、格式異常、`="0050"` 代號、`--` 值）。
- [x] 6.9 建立回歸測試，覆蓋既有已知錯誤（例如 `i[0] != '='` 導致資料遺失）。

## 7. Verification
- [x] 7.1 以 `MI_INDEX_ALL_20260318.csv` 驗證 TWSE 對映。
- [x] 7.2 以 `SQUOTE_AL_1150318.csv` 驗證 TPEX 對映。
- [x] 7.3 驗證 DB 日期均為西元格式。
- [x] 7.4 驗證查詢同表即可涵蓋股票/ETF/權證。

## 8. Documentation
- [x] 8.1 撰寫 canonical 欄位字典與對映表文件。
- [x] 8.2 撰寫分類規則決策表與維護準則。
- [x] 8.3 撰寫新來源格式變動時的排查流程。
- [x] 8.4 撰寫測試矩陣（測試類型、案例數、涵蓋風險、對應需求）。

## 9. 驗收標準 (Acceptance Criteria)
- [x] 9.1 `pytest` 測試全數通過（83 個），樣本驗證測試在缺少真實來源檔時採 skip（CI 友善設計）。
- [x] 9.2 解析/正規化/分類/持久化核心模組測試覆蓋率達 90% 以上。
- [x] 9.3 至少具備 40 個以上測試案例，且包含 10 個以上防呆（低級錯誤）案例。
- [x] 9.4 日期區間抓取可區分：正常交易日、例假日、颱風假/臨時休市、來源錯誤。
- [x] 9.5 樣本驗證測試在有樣本檔時執行完整驗證，缺少時以 skip 記錄而非 FileNotFoundError 中止 CI；TWSE `=` 開頭代號保留無遺失。
- [x] 9.6 DB 寫入冪等，重跑同區間不得產生重複資料。
