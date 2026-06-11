## Why

專案已在 2026-06 從 Python/Tkinter 全面遷移至 Rust/Tauri 架構，但 `docs/` 目錄下的 5 份文件仍全部基於 Python 時代的程式碼撰寫。過期文件會誤導開發者與使用者，降低專案的可維護性與入門體驗。現在需要全面更新文件，讓它準確反映 Rust 專案的現狀。

## What Changes

- **重組 `docs/` 目錄結構**：從單層 5 個檔案，改為 `guide/`、`development/`、`operations/` 三子目錄架構
- **刪除 2 份過期文件**：`CODE_REVIEW_REPORT.md`（針對已刪除的 Python 程式碼）、`OPENSPEC_REVIEW_FIX_PLAN.md`（歸屬 `openspec/`）
- **重寫 3 份既有文件**：`cli_usage.md` → `guide/命令列工具.md`、`debugging_guide.md` → `operations/問題排查.md`、`testing_matrix.md` → `development/測試.md`
- **新增 5 份文件**：`guide/安裝指南.md`、`development/系統架構.md`、`development/資料庫結構.md`、`development/資料來源.md`、`operations/自動化排程.md`
- **`docs/README.md` 導覽頁**：串連所有文件的入口

## Capabilities

### New Capabilities

- `user-guide`: 終端使用者面向的操作說明，包含安裝方式與 CLI 指令參考
- `dev-guide`: 開發者面向的技術文件，包含系統架構、資料庫 Schema、資料來源 API 格式、測試策略
- `ops-guide`: 維運者面向的操作指引，包含常見問題排查與自動化排程設定

### Modified Capabilities

（無 — 此專案先前無 OpenSpec spec 定義）

## Impact

- `docs/` 目錄結構全面變更：舊檔案刪除，新目錄與新檔案建立
- `README.md` 中若有連結指向舊 `docs/` 檔案，需要一併更新
- `CONTRIBUTING.md` 若有指引開發者閱讀舊文件，需要更新連結
- 不影響任何 Rust 原始碼、測試、或建置流程
