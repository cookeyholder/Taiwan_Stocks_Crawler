## ADDED Requirements

### Requirement: Rust 端 RSI 計算
系統需在 Rust 後端以 Wilder's smoothing 方法計算 RSI。

#### Scenario: RSI 計算
- **WHEN** 分析模組收到價格陣列與週期
- **THEN** 需回傳 RSI 陣列，前 period 筆為 None，其餘值在 0.0–100.0 之間

#### Scenario: 透過參數表單調整週期
- **WHEN** 使用者在 Vue 參數表單中更改 RSI 週期
- **THEN** Tauri 指令需以新週期重新計算 RSI 並回傳更新資料

### Requirement: ECharts RSI 副圖
RSI 需在價格主圖下方的獨立圖格渲染。

#### Scenario: RSI 面板含超買/超賣線
- **WHEN** RSI 顯示時
- **THEN** 面板需在 70（超買）與 30（超賣）位置繪製水平參考線並標示文字
