## ADDED Requirements

### Requirement: Render Bollinger Bands on Tkinter GUI

系統必須在 Tkinter 視窗內嵌入的 Matplotlib 畫布上，繪製出個股收盤價折線、布林中軌（20 MA）、上軌與下軌，並在上軌與下軌之間填滿半透明的背景陰影。

#### Scenario: Render chart successfully
- **WHEN** 使用者輸入有效的股票代號（例如 2330）並點選「開始分析」
- **THEN** 系統計算 20 日布林通道，並在 Tkinter 介面上呈現股價與布林通道陰影圖表

### Requirement: Display Volume Chart

系統必須在股價圖下方，繪製對應交易日期的成交量柱狀圖。

#### Scenario: Render volume successfully
- **WHEN** 股價圖成功繪製時
- **THEN** 系統在副圖上呈現對應的成交量柱狀圖
