## ADDED Requirements

### Requirement: 安裝說明文件

`guide/installation.md` 必須涵蓋終端使用者取得與執行此軟體所需的所有步驟。

#### Scenario: 涵蓋系統需求
- **WHEN** 使用者閱讀安裝說明
- **THEN** 文件列出支援的作業系統與最低 Rust 版本需求

#### Scenario: 涵蓋編譯方式
- **WHEN** 使用者從原始碼編譯
- **THEN** 文件提供 `cargo build --release` 的完整指令與預期產出

#### Scenario: 涵蓋預編譯二進位檔
- **WHEN** 使用者不想自行編譯
- **THEN** 文件說明如何從 GitHub Releases 下載預編譯的二進位檔

#### Scenario: 涵蓋初次執行指引
- **WHEN** 使用者第一次執行軟體
- **THEN** 文件提供 CLI 或 GUI 的啟動方式與驗證步驟

### Requirement: CLI 指令參考文件

`guide/cli.md` 必須完整記錄 `stock-cli` 的所有子指令、參數、與使用範例。

#### Scenario: 涵蓋所有子指令
- **WHEN** 使用者查閱 CLI 文件
- **THEN** 文件列出 `fetch`、`screen`、`sync` 三個子指令及其用途

#### Scenario: 涵蓋 fetch 子指令
- **WHEN** 使用者查閱 `fetch` 指令
- **THEN** 文件說明 `--today` 與 `--date` 參數，並提供 JSON/CSV API 的使用範例

#### Scenario: 涵蓋 screen 子指令
- **WHEN** 使用者查閱 `screen` 指令
- **THEN** 文件說明 `--bandwidth-limit`、`--consecutive-days`、`--min-volume`、`--instrument-types` 等篩選參數

#### Scenario: 涵蓋 sync 子指令
- **WHEN** 使用者查閱 `sync` 指令
- **THEN** 文件說明 `--start` 與 `--end` 參數與批次補檔的使用方式

#### Scenario: 涵蓋共用選項
- **WHEN** 使用者查閱 CLI 文件
- **THEN** 文件說明 `--db-path`、`--help` 等共用選項

#### Scenario: 提供實際範例
- **WHEN** 使用者閱讀 CLI 文件
- **THEN** 每個子指令至少有一個完整的執行範例（含輸入與預期輸出）
