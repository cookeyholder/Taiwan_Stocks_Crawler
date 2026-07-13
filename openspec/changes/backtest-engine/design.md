## Context

現況：`screen_bollinger_squeeze` 是一個 200 行的單體函式，同時負責 SQL 查詢、Bollinger 計算、squeeze/breakout 判斷、結果組裝。新增任一種選股策略都需要重複這些步驟。

設計目標：抽出一個可插拔的回測引擎，讓新策略只需要實作一個 trait 方法。

## Goals / Non-Goals

**Goals:**
- 定義 `Strategy` trait，讓新策略只需實作 `evaluate()`
- `BacktestEngine` 提供 `run_date(date)`（單日篩選）和 `run_range(start, end)`（區間回測）
- 現有 `screen_bollinger` Tauri command / CLI `screen` 行為完全不變
- `instrument_type` 過濾放在 Engine 層（SQL WHERE），策略不需要知道

**Non-Goals:**
- 不處理回測後的績效分析（Sharpe、drawdown 等）
- 不處理 portfolio 管理（部位追蹤、再平衡）
- 不新增第二個策略（但架構要讓新增策略不需改既有 code）

## Decisions

### 1. `evaluate()` 單一方法 vs `prepare()` + `check()` 兩階段

**決策**：單一 `evaluate()`。每個日期 × 每檔股票重新計算指標。

**理由**：維護性優先，效能第二。1000 檔 × 200 天 × O(20) 的運算量現代 CPU 亞秒級完成。等 profile 確認是瓶頸再加指標快取。

### 2. Engine 為純同步模組

**決策**：`BacktestEngine` 放在 `backtest/`，不碰 async，不依賴 Pipeline。

**理由**：Engine 的運算只有 CPU 和 SQLite（已持有 conn），沒有 IO 等待。同步讓 unit test 更容易撰寫。Pipeline 層負責 async 邊界，內部呼叫 Engine。

### 3. SignalRecord 採 `metadata: HashMap<String, f64>`

**決策**：策略自帶的數值（bandwidth、upper_band 等）以 `HashMap<String, f64>` 存放。

**理由**：`dyn Strategy` 不支援關聯型別，枚舉違反開放封閉原則。`HashMap` 序列化無障礙，可跨 Tauri boundary。

### 4. `instrument_type` 過濾由 Engine 在 SQL 層執行

**決策**：Engine 接受 `instrument_types: Option<&[String]>` 參數，下推到 SQL WHERE。

**理由**：策略不需要知道過濾存在。日後若有策略需要依類型判斷，再加欄位到 `OhlcvRecord`。

### 5. 資料一次載入，記憶體內迭代

**決策**：`run_range` 以一個 SQL 查詢載入全部範圍的 OHLCV（含 warmup buffer），分組後在記憶體內按日期迭代。

**理由**：避免 N+1 查詢，現有 batch-load 已證明有效。

### 6. Warmup buffer 由 `strategy.min_history()` 決定

**決策**：Engine 載入資料的起始日期由策略的 `min_history()` 決定，而非寫死 `consecutive_days + 40`（現有 `db/mod.rs:166` 的做法）。

```
run_date(date, strategy, ...):
  1. SELECT MAX(trade_date) WHERE trade_date <= date  → effective_date
  2. SELECT DISTINCT trade_date WHERE trade_date <= effective_date
     ORDER BY trade_date DESC LIMIT strategy.min_history()  → warmup_dates
  3. start_date = warmup_dates.last()  (最早的日期)
  4. 載入 [start_date, effective_date] 的全部 OHLCV

run_range(start, end, strategy, ...):
  1. 取得 [start, end] 區間內的所有交易日 dates_in_range
  2. 在 start 之前取 strategy.min_history() 筆交易日作為 warmup
  3. 載入 [warmup_start, end] 的全部 OHLCV（一次查詢）
```

**理由**：不同策略需要不同的 warmup 長度（Bollinger 需 20+N，MACD 需 26+9）。由策略自決避免寫死常數。

### 7. `security_name` 載入步驟

**決策**：Engine 在載入 OHLCV 後，另開一個 query 載入 `name_map`，邏輯與現有 `db/mod.rs:232-250` 相同。

```sql
SELECT symbol_clean, security_name FROM market_quote_core
WHERE (symbol_clean, trade_date) IN (
    SELECT symbol_clean, MAX(trade_date)
    FROM market_quote_core
    WHERE security_name IS NOT NULL AND trade_date <= ?1
    GROUP BY symbol_clean
)
```

`?1` 傳入 `effective_date`（run_date）或 `end`（run_range）。

**理由**：`security_name` 可能不在目標日期的 row 上（歷史資料補洞），需要取「截至該日期最近的非空名稱」。

### 8. BollingerSqueezeBreakout 策略參數

**決策**：策略 struct 包含以下欄位，固定參數與可調參數分明：

```rust
pub struct BollingerSqueezeBreakout {
    pub period: usize,           // 固定 20
    pub std_dev: f64,            // 固定 2.0
    pub bandwidth_limit: f64,    // 可調，預設 0.08
    pub consecutive_days: i32,   // 可調，預設 5
    pub min_volume: i64,         // 可調，預設 1_000_000
}
```

`min_history()` 回傳 `period + consecutive_days as usize`（= 25）。

**理由**：固定參數（period、std_dev）是布林通道定義的一部分，不應由呼叫端任意調整。可調參數對應現有 `Pipeline::screen` 的參數簽名，保持向後相容。

## Risks / Trade-offs

- **[取捨] 每次 evaluate 重新計算指標**：維護性 better than 極致效能。若實測發現延遲，可在 Engine 層加 computed indicators cache，策略不需改動。
- **[風險] run_range 對大量股票 × 長日期區間的記憶體使用**：1000 檔 × 500 天 × OhlcvRecord ≈ 10-20MB，可接受。若未來擴充到全市場（~2000 檔），約 40MB，仍在合理範圍。
- **[風險] 既有 db 測試 `screen_bollinger_squeeze` 需遷移**：現有兩個測試（`empty_on_no_data`、`finds_tight_symbol`）需改寫為 Engine + Strategy 的測試。
