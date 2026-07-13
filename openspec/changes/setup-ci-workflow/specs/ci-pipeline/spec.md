## ADDED Requirements

### Requirement: 日常提交與 PR 觸發 CI 檢查
系統必須在每一次推送（Push）到 `main`、`master`、`dev` 分支，或發起針對這些分支的 Pull Request 時，自動觸發持續整合（CI）日常檢查流程。

#### Scenario: 觸發日常 CI
- **WHEN** 開發者推送程式碼到 `main` 分支，或發起一個 PR 至 `dev` 分支。
- **THEN** GitHub Actions 必須自動啟動日常檢查工作流。

### Requirement: Rust 程式碼風格與 Clippy 自動修正
系統必須在 CI 中自動執行 Rust 的 `cargo fmt --check` 格式檢查，並自動執行 `cargo clippy --fix` 修正所有能自動修復的 Clippy 警告。

#### Scenario: 自動修正 Clippy 警告並 Push 回分支
- **WHEN** Rust 程式碼包含可自動修正的 Clippy 警告（如 `collapsible_if`、`len_zero` 等）。
- **THEN** CI 必須自動套用修正，並將改動以包含 `[skip ci] style: auto fix clippy warnings` 訊息的 commit 推送回原 PR 分支，且此 commit 絕不能再次觸發 CI 流程。

#### Scenario: 語法錯誤或無法自動修復之 clippy 錯誤
- **WHEN** Rust 程式碼包含無法自動修正的 clippy 錯誤或編譯語法錯誤。
- **THEN** CI 必須標記 Clippy 檢查步驟為失敗，且不得提交任何自動修正的 commit。

### Requirement: 雙端測試與前端型別檢查
系統必須執行 Rust 工作區的所有單元與整合測試，並在前端（`stock-tauri`）執行 TypeScript 型別檢查與 vitest 測試，任何一個步驟失敗時均須回報 CI 失敗。

#### Scenario: 雙端測試與前端型別檢查順利通過
- **WHEN** Rust 測試、前端 vitest 測試以及前端 TypeScript 編譯（`vue-tsc --noEmit`）皆無錯誤時。
- **THEN** 整個 CI 檢查必須順利通過並回報綠燈。
