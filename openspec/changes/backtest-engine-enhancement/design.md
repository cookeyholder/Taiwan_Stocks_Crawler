## Context

目前 `BacktestEngine::run_date` / `run_range` 回傳 `BacktestOutput { signals: Vec<SignalRecord> }`。每個 `SignalRecord` 有 `direction`（正=做多、負=做空）、`strength`、`close_price`。但沒有進一步分析這些訊號的績效表現。

`BollingerSqueezeBreakout` 是唯一實作的策略，硬編碼在 CLI 和 `Pipeline::screen()` 中。新增策略需要修改多個檔案。

## Goals / Non-Goals

**Goals:**
- `PerformanceMetrics`：勝率、Sharpe ratio、最大回撤、總報酬率
- `PortfolioSimulator`：從 `SignalRecord` 序列模擬交易，支援初始資金、部位管理
- `StrategyKind` enum + `list()` / `create()`：所有策略集中註冊
- 兩個新策略：MaCross（黃金/死亡交叉）、RSI（超買/超賣）
- CLI `backtest --strategy <name> --param key=value --capital <N>`

**Non-Goals:**
- 不修改 `BacktestEngine::run_date` / `run_range`（策略 trait 不變）
- 不做 GUI（那是下一階段）
- 不做圖表（那是下下階段）

## Decisions

### 1. PortfolioSimulator：訊號→交易（方向 A1）

```
SignalRecord { direction: 1, ... }  →  買進（若尚未持有）
SignalRecord { direction: -1, ... } →  賣出（若持有）
SignalRecord { direction: 0, ... }  →  無動作（跳過）
```

對同一個 symbol 的訊號以時間排序處理。當有買進訊號且未持有該股時，以當時 `close_price` 買入。當有賣出訊號且持有時賣出，計算單筆損益。

權益曲線（`equity_curve: Vec<f64>`）每日計算：對回測區間內**每個交易日**（從 signals 中推導出的完整日期範圍），記錄當日收盤後的總權益（現金 + 所有持倉以當日收盤價計算的市值）。回傳的 `Vec<f64>` 順序與日期排序一致（由舊到新）。

`PortfolioSimulator::simulate()` 需要以下參數：
- `signals: &[SignalRecord]` — 事件驅動的訊號序列
- `initial_capital: f64` — 初始資金
- `prices_map: &HashMap<String, BTreeMap<String, f64>>` — symbol → (日期 → 當日收盤價)，用於權益曲線計算與交易價格查閱
- `dates: &[String]` — 回測區間內所有交易日（已排序），用於生成每日權益曲線

```
struct TradeRecord {
    symbol: String,
    name: Option<String>,
    buy_date: String,
    buy_price: f64,
    sell_date: Option<String>,
    sell_price: Option<f64>,
    shares: i64,
    pnl: f64,         // 已實現損益
    pnl_pct: f64,     // 報酬率百分比
}
```

### 2. PerformanceMetrics

從 `TradeRecord` 序列與 `equity_curve` 計算：

```rust
pub struct PerformanceMetrics {
    pub total_return_pct: f64,
    pub annualized_return_pct: f64,
    pub win_rate: f64,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub sharpe_ratio: f64,
    pub max_drawdown_pct: f64,
    pub final_value: f64,
}
```

- Sharpe ratio 以無風險利率 0% 簡化計算，從 `equity_curve` 推導每日報酬率序列：`daily_return_i = (equity_curve[i] - equity_curve[i-1]) / equity_curve[i-1]`
- 最大回撤從每日權益值序列（`equity_curve`）計算
- 年化報酬率以實際交易日數 / 252 換算

### 3. StrategyKind enum + 註冊

沿用 `classify_by_symbol` 的集中註冊模式：

```rust
pub enum StrategyKind {
    BollingerSqueeze(BollingerSqueezeBreakout),
    MaCross(MaCrossStrategy),
    Rsi(RsiStrategy),
}

impl StrategyKind {
    /// 回傳所有可用的策略清單（供 CLI/GUI 列舉）
    pub fn list() -> Vec<StrategyInfo> { ... }

    /// 根據名稱與參數建立策略實例
    pub fn create(id: &str, params: &HashMap<String, f64>) -> Result<Self, String> { ... }

    /// 取得 min_history
    pub fn min_history(&self) -> usize { ... }

    /// 執行策略評估（委託給內部 strategy）
    pub fn evaluate(&self, ohlcv: &[OhlcvRecord], date_index: usize) -> Signal { ... }
}
```

### 4. 新策略：MaCross

黃金交叉與死亡交叉僅在「交叉發生的當天」產出訊號，不連續觸發。

```
id: "ma_cross"
參數: { fast_period: 5, slow_period: 20, min_volume: 0 }
邏輯:
  - 計算 fast SMA 與 slow SMA
  - fast 由下往上突破 slow → direction: 1, has_signal: true（買進訊號）
  - fast 由上往下跌破 slow → direction: -1, has_signal: true（賣出訊號）
  - 其餘 → has_signal: false, direction: 0（不產出任何 SignalRecord）
min_history: slow_period
```

### 5. 新策略：RSI

RSI 穿越超買/超賣訊號僅在「穿越當天」產出，不連續觸發。

```
id: "rsi"
參數: { period: 14, overbought: 70, oversold: 30, min_volume: 0 }
邏輯:
  - 計算 RSI
  - RSI 由 oversold 之下回升至 oversold 之上 → direction: 1, has_signal: true
  - RSI 由 overbought 之上回落至 overbought 之下 → direction: -1, has_signal: true
  - 其餘 → has_signal: false, direction: 0（不產出任何 SignalRecord）
min_history: period + 1
```

### 6. CLI 改動

```
stock-cli backtest \
  --strategy ma_cross \
  --param fast_period=5 \
  --param slow_period=20 \
  --capital 1000000 \
  --start 2026-07-01 --end 2026-07-14 \
  --types stock

輸出：
  ─── 績效摘要 ───
  總報酬率: +12.3%    勝率: 58.2%    Sharpe: 1.24
  最大回撤: -8.7%     交易次數: 187  最後價值: 1,123,000

  ─── 交易明細 ───
  日期       │ 代號 │ 買/賣 │ 價格   │ 損益
  ...
```

## Risks / Trade-offs

- **[訊號頻率] → 若不正確實作，MaCross 與 RSI 可能在每個交易日都產出 signal**（例如將 SMA 連續狀態誤判為每次比較都觸發）。策略必須只在「交叉 / 穿越發生的當天」回傳 `has_signal: true`，其餘回傳 `false` 以確保 BacktestEngine 不會產生無意義的 SignalRecord。
- **[參數傳遞] → HashMap<String, f64> 無法表達字串參數**，但現有策略的參數皆為數值，未來有需要可改為 serde_json::Value
- **[Sharpe 簡化] → 使用 0% 無風險利率**，對於策略比較而言相對值仍有意義
