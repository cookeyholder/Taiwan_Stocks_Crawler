## Why

目前選股器（Screener）僅支援單一「布林通道擠壓突破」條件，無法滿足投資人對於順勢交易與動能選股的需求。實務上，「均線多頭排列」（短天期均線高於中長天期均線且股價站穩其上）與「創波段新高」（突破過去 N 日高點）是捕捉強勢股的核心指標，且兩者常搭配布林指標進行複合條件篩選。

## What Changes

- **核心策略實作 (`stock-core::backtest`)**：
  - 新增 `MaBullishAlignmentStrategy`（均線多頭排列策略，支援 MA5 > MA10 > MA20 與自訂週期）。
  - 新增 `NewHighBreakoutStrategy`（創波段新高策略，支援 20 / 60 / 120 日波段高點突破）。
  - 擴充 `StrategyKind` 與 `StrategyInfo` 清單，支援回測引擎與選股呼叫。
  - 擴充 `ScreenResult` 支援策略標籤（`tags`），記錄個股符合之策略特徵。
- **桌面應用狀態與多策略選股 (`stock-gpui::state`)**：
  - 在 `CoreState` 擴充支援多策略與複合條件選股介面 `screen_composite_range`。
- **選股介面整合 (`stock-gpui::views::screener`)**：
  - 在 `ScreenerView` 提供策略模式切換（「布林通道擠壓」、「均線多頭排列」、「創波段新高」、「複合選股：多頭排列 + 創波段新高」）。
  - 根據選擇之模式提供對應參數輸入（均線週期、波段天數、成交量濾網）。
  - 在結果表格新增「策略標籤」欄位，顯示觸發條件徽章。

## Capabilities

### New Capabilities

- `composite-screener`: 提供均線多頭排列、創波段新高與多條件複合選股能力，並支援視覺化策略標籤與參數自訂。

## Impact

- **核心模組**：
  - `stock-core/src/backtest/ma_alignment.rs`：新增均線多頭排列策略。
  - `stock-core/src/backtest/new_high.rs`：新增創波段新高策略。
  - `stock-core/src/backtest/strategies.rs`：擴充策略枚舉與工廠。
  - `stock-core/src/models.rs`：擴充 `ScreenResult` 支援 `tags`。
- **桌面應用**：
  - `stock-gpui/src/state.rs`：新增複合選股協調介面。
  - `stock-gpui/src/views/screener.rs`：整合策略選擇、參數與標籤呈現。
