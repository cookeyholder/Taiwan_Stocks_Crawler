# Proposal: harden-twse-tpex-market-crawler

## Why
現行臺股成交行情抓取流程在上市（TWSE）與上櫃（TPEX）來源格式差異下，容易出現資料遺漏與欄位不一致問題：
- TWSE 與 TPEX 欄位格式不同，且代號格式（如 `="0050"`）需清洗後才能一致化。
- TWSE 使用西元日期參數；TPEX 常使用民國年格式，若未標準化，入庫日期會混亂。
- 現有流程對於網路失敗、來源格式變動與欄位缺值的韌性不足。

## What
建立一套更穩定、可追蹤、可擴充的行情擷取與標準化流程：
1. 分別實作 TWSE / TPEX 來源轉接器（兩套解析器）。
2. 輸出統一 Canonical 結構，並寫入同一個資料庫主表（方案 A：同表 + extension）。
3. 日期與時間全面標準化為西元年（ISO 8601）。
4. 保留代號原始值與清洗值（`symbol_raw` + `symbol_clean`）。
5. 增加市場別欄位（`market_code`：TWSE/TPEX）與商品型態欄位（stock/etf/warrant...）。
6. 導入穩健抓取策略（timeout、retry、退避、可觀測錯誤）。
7. 讓資料存取層具可攜性：先支援 SQLite3，未來可切換 PostgreSQL。

## Scope
- In scope:
  - 來源解析：TWSE/TPEX 異質格式整併
  - 年份正規化：民國年→西元年
  - Canonical 欄位字典與欄位對映
  - 同表 core + extension 寫入策略
  - 基本資料品質檢查與錯誤追蹤
- Out of scope:
  - 即時串流推播
  - 歷史資料全面回補（只提供回補流程，不在本次完成全部回補）
  - 交易策略本身調整

## Success Criteria
- 任一交易日可穩定抓取 TWSE + TPEX 並合併入同一結構。
- 資料庫中日期欄位一律為西元年格式。
- 每筆資料都可追溯來源（檔案/市場/批次）。
- 對於暫時性網路錯誤可自動重試且不中斷整體流程。
- 同一天重跑不會造成重複資料（具冪等寫入策略）。

## Risks
- 來源網站格式變更導致 parser 失效。
- 商品分類規則在少數代號可能有歧義。
- 跨資料庫可攜性若使用方言 SQL 會受限。

## Mitigations
- 建立 parser 健康檢查與欄位驗證。
- 分類採「section/cfi/代號樣式」多層決策並可人工覆寫。
- SQL 僅使用可攜子集，DB 方言隔離在 repository adapter。
