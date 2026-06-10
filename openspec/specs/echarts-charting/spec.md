# echarts-charting Specification

## Purpose
TBD - created by archiving change migrate-to-tauri-rust. Update Purpose after archive.
## Requirements
### Requirement: K 線圖含布林通道疊加
圖表需渲染 OHLC K 線，並可選疊加布林通道線。

#### Scenario: 基本 K 線渲染
- **WHEN** 圖表收到含日期的 OHLC 資料
- **THEN** 需以紅綠 K 線呈現，x 軸為日期，y 軸為價格

#### Scenario: 布林通道疊加
- **WHEN** 圖表收到上軌、中軌、下軌陣列
- **THEN** 需在 K 線圖上疊加三條線 series

### Requirement: 多圖格面板
圖表需支援價格、成交量、RSI、MACD 各自獨立的圖格，全部水平同步。

#### Scenario: 跨面板同步縮放
- **WHEN** 使用者對任一圖格進行縮放
- **THEN** 所有其他圖格需透過共用 dataZoom 縮放到相同日期範圍

### Requirement: 互動功能
圖表需支援十字線 tooltip、圖例切換與滑鼠滾輪縮放。

#### Scenario: 十字線 tooltip
- **WHEN** 使用者滑鼠懸停在圖表上
- **THEN** 需顯示十字線，並在 tooltip 中顯示該 x 位置的所有 series 數值

