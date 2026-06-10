## ADDED Requirements

### Requirement: 基於版本號的 Migration
系統需以 SQLite `PRAGMA user_version` 記錄當前 schema 版本，在啟動時依序執行未套用的 migration。

#### Scenario: 全新資料庫
- **WHEN** 應用程式啟動時資料庫為空
- **THEN** 需從版本 0 執行所有 migration 腳本至最新版本

#### Scenario: 已 migration 的資料庫
- **WHEN** 應用程式啟動且 `PRAGMA user_version` 等於最新 migration 版本
- **THEN** 不執行任何 migration 腳本

#### Scenario: 部分 migration
- **WHEN** 應用程式啟動且 `PRAGMA user_version` 處於中間版本
- **THEN** 僅執行比目前版本新的 migration
