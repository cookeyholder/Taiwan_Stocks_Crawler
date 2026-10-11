## MODIFIED Requirements

### Requirement: 跨平台視窗管理
應用程式需建立一個原生 OS 視窗，標題、尺寸與最小尺寸可設定。在 GPUI 遷移過渡期內，桌面端可由 Tauri 或 GPUI 殼層提供；遷移完成後全面由 `gpui-desktop-shell` 原生殼層提供。

#### Scenario: 啟動時開啟視窗
- **WHEN** 應用程式啟動
- **THEN** 需開啟標題為「台股布林通道擠壓選股系統」的視窗，預設 1280x800，最小 960x640

### Requirement: 跨平台打包
建置系統需產生各平台的原生二進位檔或安裝包。在完成 GPUI 遷移後，建置流程由純 Cargo 與平臺打包工具驅動，不再依賴 `cargo-tauri` 與 Node.js 工具鏈。

#### Scenario: 建置 Linux 版
- **WHEN** 執行桌面端發行建置
- **THEN** 需產生原生 Linux 執行檔或發行安裝包

#### Scenario: 建置 Windows 版
- **WHEN** 執行桌面端發行建置
- **THEN** 需產生原生 Windows 執行檔或發行安裝包
