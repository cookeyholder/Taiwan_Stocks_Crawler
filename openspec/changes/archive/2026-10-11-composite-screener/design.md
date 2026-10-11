# Design: 均線多頭排列、創波段新高複合選股器

## 1. 概述

本設計為 `Taiwan_Stocks_Crawler` 提供均線多頭排列（MA Bullish Alignment）、創波段新高（New High Breakout）以及多條件複合選股能力。透過將技術指標與價格動能模組化為符合 `Strategy` trait 之元件，並在選股器介面整合策略模式切換、參數配置與策略標籤徽章顯示。

## 2. 策略演算法與核心模組

### 2.1 均線多頭排列策略 (`MaBullishAlignmentStrategy`)

- **路徑**：`stock-core/src/backtest/ma_alignment.rs`
- **結構與參數**：
  ```rust
  pub struct MaBullishAlignmentStrategy {
      pub short_period: usize, // 預設 5
      pub mid_period: usize,   // 預設 10
      pub long_period: usize,  // 預設 20
      pub min_volume: u64,     // 預設 0
  }
  ```
- **評估邏輯**：
  1. `min_history = long_period`。若歷史資料筆數不足 `long_period` 或 `date_index + 1 < long_period`，回傳無訊號。
  2. 計算當前時點之短天期、中天期、長天期簡單移動平均線（SMA）：
     $$\text{MA}_N = \frac{1}{N} \sum_{i=0}^{N-1} \text{Close}_{t-i}$$
  3. 判定條件：
     - $\text{Close}_t \ge \text{MA}_{\text{short}}$
     - $\text{MA}_{\text{short}} > \text{MA}_{\text{mid}}$
     - $\text{MA}_{\text{mid}} > \text{MA}_{\text{long}}$
     - $\text{Volume}_t \ge \text{min\_volume}$
  4. 若全數滿足，產生買進訊號 `Signal { direction: 1, ... }`，並於 metadata 記錄均線數值。

### 2.2 創波段新高策略 (`NewHighBreakoutStrategy`)

- **路徑**：`stock-core/src/backtest/new_high.rs`
- **結構與參數**：
  ```rust
  pub struct NewHighBreakoutStrategy {
      pub lookback_period: usize, // 預設 20，可選 60 或 120
      pub use_high: bool,         // 預設 true（比對最高價），false（比對收盤價）
      pub min_volume: u64,        // 預設 0
  }
  ```
- **評估邏輯**：
  1. `min_history = lookback_period + 1`。若歷史天數不足，回傳無訊號。
  2. 取回過去 $N$ 日之最高價基準：
     $$\text{PriorMax} = \max_{i \in [1, N]} (\text{High}_{t-i})$$
  3. 當日價格 $\text{CurrPrice} = \text{High}_t$（若 `use_high=true`）或 $\text{Close}_t$。
  4. 判定條件：
     - $\text{CurrPrice} \ge \text{PriorMax}$
     - $\text{Volume}_t \ge \text{min\_volume}$
  5. 若滿足，產生買進訊號 `Signal { direction: 1, ... }`，並記錄突破之波段天數與基準價格。

### 2.3 策略登錄器擴充 (`stock-core/src/backtest/strategies.rs`)

- 在 `StrategyKind` 加入：
  - `MaBullishAlignment`
  - `NewHighBreakout`
- 擴充 `StrategyInfo` 清單，新增上述策略之名稱、說明、預設參數說明。
- 工廠方法 `create_strategy` 實體化對應策略物件。

### 2.4 模型層擴充 (`stock-core/src/models.rs`)

- `ScreenResult` 擴充：
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct ScreenResult {
      // 既有欄位保持不變
      pub date: String,
      pub symbol: String,
      pub name: String,
      pub close: f64,
      pub volume: u64,
      pub change_pct: f64,
      pub signal_date: String,
      pub score: f64,
      #[serde(default)]
      pub tags: Vec<String>, // 例如 ["多頭排列", "創20日新高"]
  }
  ```

## 3. 桌面應用狀態與選股協調 (`stock-gpui::state`)

### 3.1 選股模式定義

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ScreenerMode {
    #[default]
    Bollinger,      // 布林通道擠壓突破
    MaAlignment,    // 均線多頭排列
    NewHigh,        // 創波段新高
    Composite,      // 均線多頭排列 + 創波段新高
}
```

### 3.2 複合選股執行器

- 擴充 `CoreState::screen_composite_range`：
  - 根據傳入之 `mode` 與參數（均線週期、波段天數）評估各策略。
  - 對於單一策略模式：產生符合條件標的，並填入單一標籤。
  - 對於 `Composite` 模式：同時評估均線多頭排列與創波段新高，只有兩者皆產生買進訊號之標的才保留，並將兩者標籤合併賦予 `tags`。

## 4. GPUI 選股介面 (`ScreenerView`) 整合

1. **策略控制面板**：
   - 提供策略模式切換控制（布林通道擠壓 / 均線多頭排列 / 創波段新高 / 複合多頭+新高）。
   - 根據選擇之模式切換顯示對應之參數調整（如波段天數 20/60/120）。
2. **選股結果表格更新**：
   - 表頭新增「策略標籤」欄位。
   - 資料列以膠囊標籤（Badge）視覺化顯示標的符合之條件。

## 5. 測試規劃

1. **策略單元測試 (`stock-core/tests/ma_alignment_test.rs`, `stock-core/tests/new_high_test.rs`)**：
   - 驗證標準多頭排列觸發買進訊號與數值正確性。
   - 驗證糾結、跌破或歷史長度不足時不觸發訊號。
   - 驗證創 20 日新高之臨界點與未突破時之過濾。
2. **模型序列化測試 (`stock-core/tests/models_test.rs`)**：
   - 驗證 `ScreenResult` 舊 JSON 在反序列化時 `tags` 預設為空向量。
3. **複合選股整合測試 (`stock-gpui/tests/composite_screener_test.rs`)**：
   - 驗證 `screen_composite_range` 在複合模式下成功過濾交集並產生雙標籤。
