# tauri-desktop-shell Specification (Deprecated / 遷移完成)

## Purpose

原先提供基於 Tauri 的桌面殼層。在完成 GPUI 遷移後全面由 `gpui-desktop-shell` 取代，此規格保留作為歷史記錄與遷移過渡參考。

## Requirements

### Requirement: 跨平台視窗管理
應用程式需建立一個原生 OS 視窗，標題、尺寸與最小尺寸可設定。在 GPUI 遷移完成後全面由 `gpui-desktop-shell` 原生殼層提供。

#### Scenario: 啟動時開啟視窗
- **WHEN** 應用程式啟動
- **THEN** 需開啟標題為「台股布林通道擠壓選股系統」的視窗，預設 1280x800，最小 960x640

### Requirement: 優雅關閉
應用程式需處理視窗關閉事件，正確釋放資料庫連線。

#### Scenario: 關閉視窗釋放資源
- **WHEN** 使用者關閉視窗
- **THEN** 資料庫連線需被關閉，行程需乾淨退出

### Requirement: 跨平台打包
建置系統需產生各平台的原生二進位檔或安裝包。在完成 GPUI 遷移後，建置流程由純 Cargo 與平臺打包工具驅動，不再依賴 `cargo-tauri` 與 Node.js 工具鏈。

#### Scenario: 建置 Linux 版
- **WHEN** 執行桌面端發行建置
- **THEN** 需產生原生 Linux 執行檔或發行安裝包

#### Scenario: 建置 Windows 版
- **WHEN** 執行桌面端發行建置
- **THEN** 需產生原生 Windows 執行檔或發行安裝包
