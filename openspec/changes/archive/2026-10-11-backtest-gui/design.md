## Context

backtest-engine-enhancement（前一階段）已完成後端：StrategyKind、PerformanceMetrics、PortfolioSimulator、CLI。本階段將其帶入 Tauri GUI。

## Goals / Non-Goals

**Goals:**
- Tauri command: run_backtest 與 list_strategies
- 回測頁面：策略選擇器、動態參數表單、日期區間、執行按鈕
- 績效摘要卡片（4 個主要指標）
- 訊號表格（可排序）
- 交易明細表格
- 完成後自動刷新快取筆數

**Non-Goals:**
- 不做圖表（下一階段）
- 不做多策略同時比較

## Decisions

### 1. run_backtest command

接受所有參數為 flat 結構：

```rust
#[derive(Deserialize)]
struct RunBacktestParams {
    strategy: String,
    params: HashMap<String, f64>,
    start_date: String,
    end_date: String,
    capital: f64,
    instrument_types: Option<Vec<String>>,
}

fn run_backtest(state, params) -> Result<BacktestReport, String>
```

內部使用 Pipeline 的 repo 建立 BacktestEngine，依據 strategy 名稱建立對應策略，執行 run_range 後傳入 PortfolioSimulator、calculate_metrics。

### 2. list_strategies command

```rust
fn list_strategies() -> Vec<StrategyInfo>
```

直接委託 StrategyKind::list()。

### 3. GUI 佈局

```
┌──────────────────────────────────────────────────────────┐
│  設定區（水平排列，flex-wrap）                              │
│  策略 [▼]  快速週期 [5]  慢速週期 [20]  成交量 [1000000]   │
│  起始日 [picker]  結束日 [picker]  資金 [1,000,000]         │
│  類型 [✓]股票 [ ]ETF   [執行回測]                           │
├──────────────────────────────────────────────────────────┤
│  績效摘要（水平排列）                                       │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐    │
│  │總報酬率   │ │勝率      │ │Sharpe    │ │最大回撤   │    │
│  │+15.2%    │ │62.5%     │ │1.34      │ │-6.8%     │    │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘    │
├──────────────────────────────────────────────────────────┤
│  交易明細表格                                              │
│  買入日 │賣出日   │代號 │名稱   │股數 │損益   │報酬率 │    │
├──────────────────────────────────────────────────────────┤
│  訊號表格                                                  │
│  日期   │代號 │名稱    │方向 │價格   │                      │
└──────────────────────────────────────────────────────────┘
```

### 4. 參數表單動態切換

不同策略有不同的參數定義（來自 StrategyInfo.params）。前端根據選擇的策略動態 render 對應的輸入欄位。每個 ParamDef 包含 name、type、default、min、max。

## Risks / Trade-offs

- **[策略切換時參數保留] → 切換策略時重設為預設值**
- **[大量訊號渲染] → 訊號可能上千筆，表格需虛擬滾動或分頁**
