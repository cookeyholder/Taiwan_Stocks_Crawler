## ADDED Requirements

### Requirement: 三頁簽佈局
應用程式需以三個頁簽呈現：單股分析、布林擠壓選股器、下載管理。

#### Scenario: 切換頁簽
- **WHEN** 使用者點擊頁簽標題
- **THEN** 對應的內容面板需顯示，其他面板需隱藏

### Requirement: Pinia 狀態管理
應用程式需使用 Pinia store 管理分析資料、篩選結果與下載進度。

#### Scenario: Store 提供響應式狀態
- **WHEN** store 屬性被更新
- **THEN** 使用該屬性的所有元件需自動重新渲染

### Requirement: Tauri IPC 整合
應用程式需使用 `@tauri-apps/api/core` 的 invoke 呼叫 Rust 指令，並監聽事件。

#### Scenario: 呼叫指令並接收結果
- **WHEN** 前端呼叫 `invoke('command_name', { params })`
- **THEN** 需收到 Rust 後端序列化後的回應
