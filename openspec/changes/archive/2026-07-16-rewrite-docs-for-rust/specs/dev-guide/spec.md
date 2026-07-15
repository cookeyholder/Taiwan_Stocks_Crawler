## ADDED Requirements

### Requirement: 系統架構文件

`development/architecture.md` 必須讓開發者在 5 分鐘內理解專案的整體架構與資料流。

#### Scenario: 涵蓋 Workspace 架構
- **WHEN** 開發者閱讀架構文件
- **THEN** 文件中以圖文說明 `stock-core`、`stock-cli`、`stock-tauri` 三個 crate 的職責與依賴關係

#### Scenario: 涵蓋資料流
- **WHEN** 開發者閱讀架構文件
- **THEN** 文件中以流程圖或文字描述從 TWSE/TPEX API 到 SQLite 資料庫的完整資料流向

#### Scenario: 涵蓋模組職責
- **WHEN** 開發者閱讀架構文件
- **THEN** 文件說明 `fetcher`、`db`、`analysis`、`pipeline` 各模組的邊界與關鍵資料型別

### Requirement: 資料庫 Schema 文件

`development/database-schema.md` 必須完整記錄 SQLite 資料庫的表格結構與 migration 歷史。

#### Scenario: 涵蓋表格定義
- **WHEN** 開發者查閱資料庫文件
- **THEN** 文件中列出 `market_quote_core`、`market_quote_extension`、`securities` 三個表格的所有欄位、型別、約束條件

#### Scenario: 涵蓋索引
- **WHEN** 開發者查閱資料庫文件
- **THEN** 文件中列出所有已建立的索引及其用途

#### Scenario: 涵蓋 Migration 歷史
- **WHEN** 開發者查閱資料庫文件
- **THEN** 文件中列出 4 個 migration 版本的變更摘要

### Requirement: 測試策略文件

`development/testing.md` 必須說明如何執行測試、測試分類方式、以及新增測試的準則。

#### Scenario: 涵蓋測試執行方式
- **WHEN** 開發者閱讀測試文件
- **THEN** 文件提供 `cargo test` 的完整指令、指定 crate 測試的方式、以及常用篩選標籤

#### Scenario: 涵蓋測試分類
- **WHEN** 開發者閱讀測試文件
- **THEN** 文件列出各 crate 的測試類型（單元測試、整合測試）與數量

#### Scenario: 涵蓋新增測試準則
- **WHEN** 開發者想要新增測試
- **THEN** 文件說明何時該寫單元測試 vs 整合測試，以及 mock HTTP 的建議方式

### Requirement: 資料來源參考文件

`development/data-sources.md` 必須記錄 TWSE 與 TPEX 的 API 端點、請求方式、與回應格式。

#### Scenario: 涵蓋 TWSE API
- **WHEN** 開發者查閱資料來源文件
- **THEN** 文件中說明 TWSE JSON API 與 CSV API 的端點 URL、參數、與回傳格式範例

#### Scenario: 涵蓋 TPEX API
- **WHEN** 開發者查閱資料來源文件
- **THEN** 文件中說明 TPEX JSON API 的端點 URL、參數、與回傳格式範例

#### Scenario: 涵蓋編碼處理
- **WHEN** 開發者查閱資料來源文件
- **THEN** 文件中說明 TWSE CSV API 使用 Big5 編碼，以及專案中如何處理
