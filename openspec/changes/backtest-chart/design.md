## Context

前一階段已實作 BacktestView.vue 含績效摘要卡片與表格。BacktestReport 的 equity_curve: Vec<f64> 已包含每日權益值，可用於視覺化。

## Goals / Non-Goals

**Goals:**
- 權益曲線圖（策略 vs Buy & Hold）
- 回撤區域圖
- 與既有 KlineChart 共用 ECharts 模式

**Non-Goals:**
- 不做互動式圖表（zooming 除外）
- 不做多策略疊加比較

## Decisions

### 1. 使用 ECharts

與 KlineChart.vue 相同技術棧。新增 ChartEquity.vue 元件：

```vue
<ChartEquity
  :equity-curve="report.equity_curve"
  :benchmark-curve="benchmarkCurve"
  :dates="tradeDates"
/>
```

### 2. 圖表佈局

```
權益曲線
═══════════════════════════════════════
 1.2M ┤
      │         ╱╲     ╱╲
 1.1M ┤   ╱╲  ╱  ╲──╱  ╲    ╱╲    ← 策略
      │  ╱  ╲╱          ╲  ╱  ╲
 1.0M ┤─╱                  ╲╱    ╲─ ← 基準
      │
      └────────────────────────────
      07-01  07-05  07-10  07-14

回撤
═══════════════════════════════════════
  0% ┤
      │    ████
 -5% ┤  ██    ██████
      │ █            ██
-10% ┤█               ███
      └────────────────────────────
```

## Risks / Trade-offs

- **[大量資料點] → 日回測可能上千點，ECharts 可順暢處理**
