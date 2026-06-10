# tauri-desktop-shell Specification

## Purpose
TBD - created by archiving change migrate-to-tauri-rust. Update Purpose after archive.
## Requirements
### Requirement: 跨平台視窗管理
應用程式需建立一個原生 OS 視窗，標題、尺寸與最小尺寸可設定。

#### Scenario: 啟動時開啟視窗
- **WHEN** 應用程式啟動
- **THEN** 需開啟標題為「台股布林通道擠壓選股系統」的視窗，預設 1200x800，最小 900x600

### Requirement: 優雅關閉
應用程式需處理視窗關閉事件，正確釋放資料庫連線。

#### Scenario: 關閉視窗釋放資源
- **WHEN** 使用者關閉視窗
- **THEN** 資料庫連線需被關閉，行程需乾淨退出

### Requirement: 跨平台打包
建置系統需產生各平台的安裝檔。

#### Scenario: 建置 Windows 版
- **WHEN** 執行 `cargo tauri build --target x86_64-pc-windows-msvc`
- **THEN** 需在 src-tauri/target/release/bundle/ 產生 .msi 安裝檔

#### Scenario: 建置 macOS 版
- **WHEN** 執行 `cargo tauri build --target x86_64-apple-darwin`
- **THEN** 需產生 .dmg 檔

#### Scenario: 建置 Linux 版
- **WHEN** 執行 `cargo tauri build --target x86_64-unknown-linux-gnu`
- **THEN** 需產生 .deb 套件與 .AppImage

