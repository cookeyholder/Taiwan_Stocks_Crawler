# Design: 三大法人買賣超爬取、入庫與副圖繪製

## 1. 概述

本設計為 `Taiwan_Stocks_Crawler` 提供三大法人（外資及陸資、投信、自營商）買賣超之資料擷取、資料庫持久化、關聯查詢與圖表副圖呈現能力。

## 2. 資料庫設計 (Migration V7)

### 2.1 資料表結構

```sql
CREATE TABLE IF NOT EXISTS institutional_investors (
    trade_date       TEXT NOT NULL,
    symbol_clean     TEXT NOT NULL,
    foreign_buy      INTEGER NOT NULL DEFAULT 0,
    foreign_sell     INTEGER NOT NULL DEFAULT 0,
    foreign_net      INTEGER NOT NULL DEFAULT 0,
    trust_buy        INTEGER NOT NULL DEFAULT 0,
    trust_sell       INTEGER NOT NULL DEFAULT 0,
    trust_net        INTEGER NOT NULL DEFAULT 0,
    dealer_buy       INTEGER NOT NULL DEFAULT 0,
    dealer_sell      INTEGER NOT NULL DEFAULT 0,
    dealer_net       INTEGER NOT NULL DEFAULT 0,
    total_net        INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (trade_date, symbol_clean)
);

CREATE INDEX IF NOT EXISTS idx_inst_investors_symbol_date
 ON institutional_investors(symbol_clean, trade_date);
```

### 2.2 資料模型 (`stock-core::models::InstitutionalRecord`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstitutionalRecord {
    pub trade_date: String,
    pub symbol: String,
    pub foreign_buy: i64,
    pub foreign_sell: i64,
    pub foreign_net: i64,
    pub trust_buy: i64,
    pub trust_sell: i64,
    pub trust_net: i64,
    pub dealer_buy: i64,
    pub dealer_sell: i64,
    pub dealer_net: i64,
    pub total_net: i64,
}
```

## 3. 資料擷取與管線 (`stock-core::fetcher`, `stock-core::pipeline`)

### 3.1 TWSE T86 報表爬取與解析
- 端點：`https://www.twse.com.tw/rwd/zh/fund/T86?response=json&date={date_compact}&selectType=ALLBUT0999`
- 清洗邏輯：
  - 去除 `="2330"` 中的等號與引號，保留純數字/字母股票代號。
  - 將逗號千分位整數字串（例如 `"12,345,000"`）解析為 `i64`。
  - 欄位對應：外資買賣、投信買賣、自營商買賣（自行+避險）、三大法人買賣超合計。

### 3.2 倉儲介面擴充 (`MarketRepository`)
- `upsert_institutional_batch(&self, records: &[InstitutionalRecord]) -> SqlResult<usize>`
- `get_institutional_records(&self, symbol: &str, start_date: &str, end_date: &str) -> SqlResult<Vec<InstitutionalRecord>>`

## 4. 狀態層與走勢分析 (`stock-gpui::state`)

- `AnalysisData` 新增：
  ```rust
  pub institutional: Vec<Option<InstitutionalRecord>>,
  ```
- `CoreState::get_analysis` 實作：
  - 查詢該標的在 OHLCV 期間內的法人記錄。
  - 依交易日建立 `HashMap<String, InstitutionalRecord>`。
  - 循序對齊每一筆 `OhlcvRecord.trade_date`，無資料之日期填入 `None`。

## 5. 圖表介面與副圖繪製 (`stock-gpui::views::analysis`)

- **副圖區塊**：
  - 於下方副圖網格加入三大法人買賣超柱狀圖（`inst_canvas`）：
    - 買超（`net >= 0`）以台股慣例紅色呈現（`rgb(0xdc2626)` 或 `rgb(0xef4444)`）。
    - 賣超（`net < 0`）以綠色呈現（`rgb(0x16a34a)` 或 `rgb(0x10b981)`）。
  - 工具列新增 Checkbox「三大法人」（`show_institutional: bool`，預設為 true）。
- **HUD 即時資訊列**：
  - 當十字游標懸浮於特定 K 線棒時，即時在頂部顯示：
    - `外資: +1,234張`、`投信: -56張`、`自營: +12張`、`法人合計: +1,190張`。

## 6. 測試規劃

1. **資料庫與遷移測試 (`stock-core/tests/`)**：
   - 驗證 Migration V7 正確建立表與索引。
   - 驗證 `upsert_institutional_batch` 與 `get_institutional_records` 讀寫。
2. **爬蟲與解析測試 (`stock-core/src/fetcher/twse.rs`)**：
   - 驗證 T86 JSON 範例解析器能夠正確提取代號與各法人買賣超。
3. **走勢對齊測試 (`stock-gpui/tests/institutional_test.rs`)**：
   - 驗證 `CoreState::get_analysis` 將法人時序精確對齊 OHLCV 交易日。
4. **無頭 UI 測試 (`stock-gpui/tests/institutional_test.rs`)**：
   - 驗證 `AnalysisView` 工具列存在「三大法人」控制項，副圖與 HUD 正確呈現。
