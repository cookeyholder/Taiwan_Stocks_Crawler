## ADDED Requirements

### Requirement: Rust 端 MACD 計算
系統需在 Rust 後端計算 MACD，支援可設定的快線、慢線與信號週期。

#### Scenario: MACD 計算
- **WHEN** 分析模組收到價格陣列、快線週期、慢線週期、信號週期
- **THEN** 需回傳 MACD 線、信號線與柱狀圖陣列，前方有適當的 None 填充

#### Scenario: 自訂 MACD 參數
- **WHEN** 使用者在 Vue 參數表單更改 MACD fast/slow/signal 值
- **THEN** Tauri 指令需以新參數重新計算 MACD

### Requirement: ECharts MACD 副圖
MACD 需在 RSI 圖格下方的獨立圖格渲染。

#### Scenario: MACD 面板渲染
- **WHEN** MACD 顯示時
- **THEN** 面板需以實線繪製 MACD 線、虛線繪製信號線、長條圖繪製柱狀圖（正值綠色、負值紅色）
