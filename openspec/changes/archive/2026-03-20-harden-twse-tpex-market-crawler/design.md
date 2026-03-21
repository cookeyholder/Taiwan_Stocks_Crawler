# Design: harden-twse-tpex-market-crawler

## Architecture

```text
            +-------------------+
            |   Scheduler/CLI   |
            +---------+---------+
                      |
                      v
        +-------------+-------------+
        | Ingestion Orchestrator    |
        +------+--------------------+
               |
     +---------+---------+
     |                   |
     v                   v
+----+----+         +----+----+
| TWSE    |         | TPEX    |
| Adapter |         | Adapter |
+----+----+         +----+----+
     |                   |
     +---------+---------+
               v
      +--------+---------+
      | Normalizer       |
      | - symbol_raw     |
      | - symbol_clean   |
      | - ROC->AD date   |
      +--------+---------+
               v
      +--------+---------+
      | Validator        |
      | - required cols  |
      | - numeric/date   |
      +--------+---------+
               v
      +--------+---------+
      | Repository       |
      | (portable SQL)   |
      +---+----------+---+
          |          |
          v          v
   market_quote_core  market_quote_extension
```

## Data Model

### 1) `market_quote_core` (single table strategy)
- 主鍵：`(trade_date, market_code, symbol_clean)`
- 核心欄位：
  - identity: `trade_date`, `market_code`, `symbol_raw`, `symbol_clean`, `security_name`
  - classification: `instrument_type`, `instrument_subtype`, `isin_code`, `cfi_code`, `section_label`
  - quote: `open_price`, `high_price`, `low_price`, `close_price`
  - trading: `volume_shares`, `turnover_value`, `trade_count`
  - movement: `change_sign`, `change_value`
  - optional: `last_bid_price`, `last_bid_qty_raw`, `last_ask_price`, `last_ask_qty_raw`, `pe_ratio`, `issued_shares`, `next_limit_up`, `next_limit_down`
  - lineage: `source_file`, `source_encoding`, `source_line_no`, `ingest_run_id`, `ingested_at`

### 2) `market_quote_extension`
- 用於低頻、來源專屬、或需要保真欄位（例如原始漲跌字串、單位換算、raw json）
- 透過 `(trade_date, market_code, symbol_clean)` 關聯 `market_quote_core`

## Canonical Mapping Strategy

### TWSE (`MI_INDEX_ALL`)
- 僅解析「每日收盤行情(全部)」區段。
- 代號可能為 `="0050"`，必須：
  - `symbol_raw = ="0050"`
  - `symbol_clean = 0050`
- `漲跌(+/-)` + `漲跌價差` 對映為 `change_sign` / `change_value`。

### TPEX (`SQUOTE_AL`)
- 使用表頭列（`代號,名稱,收盤,...`）後的 17 欄主資料列。
- `漲跌`（如 `+0.18`）拆為：
  - `change_sign = +`
  - `change_value = 0.18`

## Year Normalization
- 系統內所有日期統一為西元（AD）ISO 格式：`YYYY-MM-DD`。
- 若來源或流程使用民國年（ROC），先轉換：`AD = ROC + 1911`。
- DB 僅儲存西元日期，避免跨來源混用年制。

## Instrument Classification Decision
優先序：
1. manual override
2. ISIN `section_label`
3. `cfi_code`
4. ticker pattern（依《中華民國證券市場編碼原則》）
5. fallback `other`

## Reliability Requirements
- HTTP：`timeout`、可設定 `retry` 次數與 backoff。
- Parser：欄位數/欄位名驗證，若不符寫入錯誤佇列而非整體中止。
- Idempotency：同 key 重跑不重複（upsert 或 staging + merge）。
- Observability：記錄每次批次的成功筆數、失敗筆數、失敗原因。

## Database Portability
- 資料存取採 repository interface。
- SQL 僅使用可攜子集（SELECT/INSERT/UPDATE/DELETE/JOIN/CASE/CTE）。
- DB 方言差異集中在 adapter，不滲透到業務邏輯。

## Trade-offs
- 同表可降低查詢成本與維護成本，但會有較多 nullable 欄位。
- extension 表可降低主表複雜度，代價是寫入流程多一步關聯。

## Rollout Plan
1. 先完成 parser + normalize + validator（不入庫 dry-run）。
2. 啟用入庫（SQLite）。
3. 完成回歸測試與品質報表。
4. 再評估 PostgreSQL adapter。
