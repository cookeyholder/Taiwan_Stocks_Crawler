# adjustable-params-form Specification

## Purpose
TBD - created by archiving change migrate-to-tauri-rust. Update Purpose after archive.
## Requirements
### Requirement: MA 週期設定
表單需提供 MA5、MA10、MA20 的數字輸入框。

#### Scenario: 變更 MA 週期
- **WHEN** 使用者將 MA5 輸入從 5 改為 8
- **THEN** 系統需以 SMA 週期 8 重新取得圖表資料並更新圖表

### Requirement: RSI 週期設定
表單需提供 RSI 週期的數字輸入框與顯示切換 checkbox。

#### Scenario: 開啟 RSI 顯示
- **WHEN** 使用者勾選「顯示 RSI」checkbox
- **THEN** RSI 副圖面板需出現

### Requirement: MACD 參數設定
表單需提供 MACD 快線、慢線、信號週期的數字輸入框與顯示切換。

#### Scenario: 變更 MACD 慢線週期
- **WHEN** 使用者將慢線週期從 26 改為 30
- **THEN** 系統需以新的慢線週期重新計算 MACD 並更新圖表

### Requirement: 參數變更防抖動
表單需對快速的輸入變更進行防抖（debounce），避免過多 Tauri IPC 呼叫。

#### Scenario: 快速輸入防抖
- **WHEN** 使用者在參數輸入框中快速輸入
- **THEN** 系統需等待最後一次輸入後 300ms 才呼叫 Rust 指令

