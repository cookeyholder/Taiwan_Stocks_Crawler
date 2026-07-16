## 1. 圖表元件

- [ ] 1.1 安裝 echarts npm 套件
- [ ] 1.2 新增 `components/ChartEquity.vue`：接受 equityCurve、benchmarkCurve、dates props
- [ ] 1.3 實作權益曲線折線圖 + Buy & Hold 基準線
- [ ] 1.4 實作回撤區域圖（在折線圖下方）

## 2. 整合至頁面

- [ ] 2.1 在 BacktestView.vue 的績效摘要與表格之間插入 ChartEquity 元件
- [ ] 2.2 計算 Buy & Hold 基準線：初始資金 * (current_close / first_close)

## 3. 驗證

- [ ] 3.1 確認圖表正確渲染
- [ ] 3.2 確認不同策略的權益曲線有差異
