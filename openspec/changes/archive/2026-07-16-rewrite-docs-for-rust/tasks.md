## 1. 清理舊文件與建立新目錄結構

- [x] 1.1 刪除舊文件：`docs/CODE_REVIEW_REPORT.md`、`docs/OPENSPEC_REVIEW_FIX_PLAN.md`
- [x] 1.2 建立新目錄：`docs/guide/`、`docs/development/`、`docs/operations/`

## 2. 寫作：入口文件

- [x] 2.1 撰寫 `docs/README.md` — 文件導覽頁（含專案簡介、文件地圖、快速連結）

## 3. 寫作：使用者指南（guide/）

- [x] 3.1 撰寫 `guide/installation.md` — 系統需求、編譯方式、預編譯二進位檔下載、初次執行指引
- [x] 3.2 撰寫 `guide/cli.md` — 從舊 `cli_usage.md` 改寫，涵蓋 `fetch`/`screen`/`sync` 三個子指令的所有參數與範例

## 4. 寫作：開發者指南（development/）

- [x] 4.1 撰寫 `development/architecture.md` — workspace 架構圖（ASCII）、資料流圖、模組職責說明
- [x] 4.2 撰寫 `development/database-schema.md` — 三個表格欄位定義、索引、migration 歷史
- [x] 4.3 撰寫 `development/testing.md` — 從舊 `testing_matrix.md` 改寫，涵蓋測試執行方式、分類、新增測試準則
- [x] 4.4 撰寫 `development/data-sources.md` — TWSE/TPEX API 端點、請求參數、回應格式範例、編碼處理說明

## 5. 寫作：維運者指南（operations/）

- [x] 5.1 撰寫 `operations/troubleshooting.md` — 從舊 `debugging_guide.md` 改寫，涵蓋休市判斷、API 格式變動、資料庫問題、連線問題
- [x] 5.2 撰寫 `operations/automation.md` — cron 排程、批次補檔、Windows 排程、資料庫維護

## 6. 更新跨文件參照

- [x] 6.1 檢查並更新 `README.md` 中指向舊 `docs/` 檔案的連結
- [x] 6.2 檢查並更新 `CONTRIBUTING.md` 中指引開發者閱讀舊文件的段落

## 7. 最終驗證

- [x] 7.1 確認舊文件已全部清理完畢
- [x] 7.2 確認所有新文件存在且無斷鏈
- [x] 7.3 運行 `cargo build` 確認無意外影響（失敗為 Windows dlltool 環境問題，與文件變更無關）
