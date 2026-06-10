## ADDED Requirements

### Requirement: 下載進度事件
Rust 後端在批次下載期間需透過 `app.emit()` 發送進度事件。

#### Scenario: 批次下載時發送進度事件
- **WHEN** 批次下載正在處理各日期
- **THEN** 系統需對每個日期發送 `download-progress` 事件，包含 current/total 與日期字串

### Requirement: Vue 前端監聽進度事件
Vue 前端需監聽 Tauri 事件，並響應式更新進度條與狀態文字。

#### Scenario: 進度條更新
- **WHEN** 前端收到 `download-progress` 事件
- **THEN** 進度條需更新為 `current / total`，狀態文字需顯示正在處理的日期

#### Scenario: 元件卸載時清理監聽器
- **WHEN** 下載頁簽元件被卸載
- **THEN** 事件監聽器需被正確取消註冊，避免記憶體洩漏
