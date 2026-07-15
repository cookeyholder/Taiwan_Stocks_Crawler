## ADDED Requirements

### Requirement: Signal 型別定義

系統 SHALL 定義 `Signal` 結構，作為 `Strategy::evaluate()` 的回傳值，攜帶策略判斷結果。

```rust
pub struct Signal {
    pub has_signal: bool,
    pub direction: i8,
    pub strength: f64,
    pub metadata: HashMap<String, f64>,
}
```

#### Scenario: 策略回傳無訊號
- **WHEN** 策略判斷某股票在某日期不符合條件
- **THEN** 回傳 `Signal { has_signal: false, direction: 0, strength: 0.0, metadata: empty }`

#### Scenario: 策略回傳買進訊號
- **WHEN** 策略判斷某股票在某日期符合買進條件
- **THEN** 回傳 `Signal { has_signal: true, direction: 1, strength: 0.0~1.0, metadata: ... }`

### Requirement: Strategy trait 定義

系統 SHALL 定義 `Strategy` trait，讓選股策略可以獨立於 Engine 實作。

```rust
pub trait Strategy: Send + Sync {
    fn id(&self) -> &str;
    fn min_history(&self) -> usize;
    fn evaluate(&self, ohlcv: &[OhlcvRecord], date_index: usize) -> Signal;
}
```

#### Scenario: 策略實作通過編譯
- **WHEN** 一個 struct 實作 `Strategy` trait
- **THEN** 編譯器接受該實作，且該策略可傳入 `BacktestEngine`

### Requirement: BacktestOutput 型別定義

系統 SHALL 定義 `BacktestOutput` 結構，作為 Engine 的回傳值，包含全部訊號記錄。

```rust
pub struct BacktestOutput {
    pub signals: Vec<SignalRecord>,
}
```

#### Scenario: 無訊號時回傳空列表
- **WHEN** 回測區間內無任何股票觸發訊號
- **THEN** 回傳 `BacktestOutput { signals: vec![] }`

### Requirement: EngineError 型別定義

系統 SHALL 定義 `EngineError` 列舉，包裝 Engine 內部可能的錯誤。

```rust
pub enum EngineError {
    Db(rusqlite::Error),
    InvalidDate(String),
}
```

#### Scenario: 日期格式錯誤
- **WHEN** 傳入無法解析的日期字串（如 "not-a-date"）
- **THEN** 回傳 `EngineError::InvalidDate`

#### Scenario: 資料庫錯誤
- **WHEN** SQLite 查詢失敗
- **THEN** 回傳 `EngineError::Db`

### Requirement: BacktestEngine 提供單日篩選

系統 SHALL 提供 `run_date()` 方法，針對「一個日期」篩選所有股票。

```rust
pub fn run_date(
    &self,
    date: &str,
    strategy: &dyn Strategy,
    instrument_types: Option<&[String]>,
) -> Result<BacktestOutput, EngineError>;
```

#### Scenario: 指定日期篩選
- **WHEN** 呼叫 `run_date("2024-03-15", strategy, None)`
- **THEN** 回傳的訊號只包含 `trade_date == "2024-03-15"` 的結果

#### Scenario: 指定日期無交易日自動回退
- **WHEN** 呼叫 `run_date("2024-03-16", strategy, None)`，且資料庫無該日資料
- **THEN** Engine 自動使用 `trade_date <= 2024-03-16` 的最大可用日期

#### Scenario: 過濾 instrument_type
- **WHEN** 呼叫 `run_date("2024-03-15", strategy, Some(&["stock"]))`
- **THEN** 結果中不包含 `instrument_type != "stock"` 的股票

### Requirement: BacktestEngine 提供區間回測

系統 SHALL 提供 `run_range()` 方法，對一段日期範圍內每一天分別篩選。

```rust
pub fn run_range(
    &self,
    start: &str,
    end: &str,
    strategy: &dyn Strategy,
    instrument_types: Option<&[String]>,
) -> Result<BacktestOutput, EngineError>;
```

#### Scenario: 區間內每個交易日都產生一批訊號
- **WHEN** 呼叫 `run_range("2024-03-01", "2024-03-10", strategy, None)`
- **THEN** 回傳訊號中包含多個不同 `trade_date` 的結果

#### Scenario: run_range 一次載入全部資料
- **WHEN** 呼叫 `run_range` 進行回測
- **THEN** Engine SHALL 以一個 SQL 查詢載入全部範圍（含 warmup buffer）的 OHLCV 資料，不在迴圈內發送額外查詢

### Requirement: SignalRecord 含日期欄位

系統 SHALL 定義 `SignalRecord` 結構，含 `trade_date` 欄位以區分訊號發生的日期。

```rust
pub struct SignalRecord {
    pub trade_date: String,
    pub symbol: String,
    pub name: Option<String>,
    pub close_price: Option<f64>,
    pub volume: Option<i64>,
    pub direction: i8,
    pub strength: f64,
    pub strategy: String,
    pub metadata: HashMap<String, f64>,
}
```

#### Scenario: SignalRecord 可轉換為 ScreenResult
- **WHEN** 呼叫 `signal.to_screen_result()`
- **THEN** 回傳的 `ScreenResult` 欄位值與 `SignalRecord` 一致

### Requirement: BollingerSqueezeBreakout 的 metadata key 映射

`BollingerSqueezeBreakout` 策略 SHALL 在 `Signal.metadata` 中填入以下固定 key，供 `to_screen_result()` 轉換使用：

| ScreenResult 欄位 | metadata key | 轉換方式 |
|---|---|---|
| `max_bandwidth` | `"max_bandwidth"` | 直接取值，預設 `0.0` |
| `latest_bandwidth` | `"bandwidth"` | 直接取值，預設 `0.0` |
| `latest_upper_band` | `"upper_band"` | `Option::map(Some)` |
| `break_upper` | `"break_upper"` | `value > 0.5` 為 `true` |

#### Scenario: BollingerSqueezeBreakout 產出含完整 metadata 的 Signal
- **WHEN** `BollingerSqueezeBreakout::evaluate()` 觸發訊號
- **THEN** `Signal.metadata` 包含 `"max_bandwidth"`、`"bandwidth"`、`"upper_band"`、`"break_upper"` 四個 key

#### Scenario: to_screen_result 從 metadata 正確轉換
- **WHEN** `SignalRecord { metadata: {"max_bandwidth": 0.05, "bandwidth": 0.04, "upper_band": 100.0, "break_upper": 1.0, ..}, .. }.to_screen_result()`
- **THEN** 回傳 `ScreenResult { max_bandwidth: 0.05, latest_bandwidth: 0.04, latest_upper_band: Some(100.0), break_upper: true, .. }`

### Requirement: 現有 screen 行為不變

Tauri `screen_bollinger` command 與 CLI `screen` 子命令的回傳型別與簽名 SHALL 保持不變。內部改用 `BacktestEngine::run_date` + `BollingerSqueezeBreakout` 策略。

#### Scenario: Tauri screen_bollinger 回傳結果不含 trade_date
- **WHEN** Tauri 前端呼叫 `screen_bollinger`（參數不變）
- **THEN** 回傳的 `Vec<ScreenResult>` 與改動前格式一致

#### Scenario: CLI screen 回傳結果不含 trade_date
- **WHEN** 執行 `stock-cli screen --max-bandwidth 0.08 --min-days 5`
- **THEN** 輸出格式與改動前一致（Symbol、Name、Close、Volume、Bandwidth 欄位）

### Requirement: CLI backtest 子命令

CLI SHALL 新增 `backtest` 子命令，支援 `--start` 和 `--end` 日期範圍參數。

#### Scenario: CLI backtest 輸出多日結果
- **WHEN** 執行 `stock-cli backtest --start 2024-03-01 --end 2024-03-10`
- **THEN** 輸出中包含 `trade_date`，且涵蓋區間內多個交易日
