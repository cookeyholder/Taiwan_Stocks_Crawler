## Why

為了提升專案代碼品質與開發效率，我們需要建構一個適合本專案（結合 Rust 工作區與 Vue/TypeScript Tauri 前端）的持續整合（CI）日常檢查流程。
透過在每次提交與 Pull Request 時自動執行格式化檢查、Clippy 靜態分析以及單元測試，並針對 Rust Clippy 警告提供自動化修正，能有效防止有錯誤或不合規範的程式碼合併至主分支，提升團隊協作效率。

## What Changes

- **新增日常 CI 工作流**：在 `.github/workflows/ci.yml` 建立日常 PR / Push 的持續整合流程。
- **自動化 Rust 檢查與修正**：
  - 執行 `cargo fmt` 檢查格式。
  - 執行 `cargo clippy --fix` 自動修正可自動修復的 Clippy 警告（如 `collapsible_if`、`len_zero` 等）。
  - 將修正後的程式碼自動 commit 並 push 回 PR 分支，並在 commit 訊息中加上 `[skip ci]` 避免無窮迴圈觸發 CI。
  - 執行 `cargo test` 確保 Rust 全工作區的單元測試與整合測試通過。
- **前端 Vue/TypeScript 檢查與測試**：
  - 執行 `npm ci` 還原依賴。
  - 執行 `npm run build`（包含 `vue-tsc --noEmit`）進行前端型別檢查與編譯檢查。
  - 執行 `npm run test`（vitest）跑前端的單元測試。

## Capabilities

### New Capabilities
- `ci-pipeline`: 提供本專案在 GitHub 上的日常自動化檢查、測試與自動修正功能。

### Modified Capabilities
<!-- 無 -->

## Impact

- **新增檔案**：`.github/workflows/ci.yml`。
- **Git 變更**：更新了根目錄的 `.gitignore` 忽略 `.worktrees/` 目錄。
- **代碼異動**：CI 的自動修正功能將會影響 `stock-core` 及其它工作區成員中未符合 clippy 規範的 Rust 代碼。
