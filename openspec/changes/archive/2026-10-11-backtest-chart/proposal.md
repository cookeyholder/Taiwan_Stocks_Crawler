## Why

回測的績效數字（勝率、Sharpe 等）只能反應整體表現，無法直觀看到資金變化的軌跡。加入權益曲線圖與回撤圖後，使用者可以一眼看出策略在哪段時間表現好、哪段時間發生較大虧損。

## What Changes

- 在 BacktestView.vue 的績效摘要下方加入 ECharts 權益曲線圖
- 圖表包含兩條線：策略權益曲線 vs Buy & Hold 基準線
- 可選：回撤區域圖（在權益曲線下方）

## Capabilities

### New Capabilities
- `backtest-chart`: 權益曲線與回撤圖表

## Impact

- stock-tauri/package.json：新增 echarts 依賴
- stock-tauri/src/views/BacktestView.vue：加入 ECharts 元件
