## 1. 建立 GitHub Actions 設定

- [x] 1.1 建立 `.github/workflows/ci.yml` 檔案
- [x] 1.2 設定 `ci.yml` 觸發條件（針對 `main`, `master`, `dev` 分支的 push 與 pull_request）

## 2. 實作 Rust 檢查與自動修正 Job

- [x] 2.1 在 `ci.yml` 中設定 `rust-ci` job
- [x] 2.2 設定 Rust stable 鏈以及 `swatinem/rust-cache` 快取
- [x] 2.3 設定 `cargo fmt --all -- --check` 檢查步驟
- [x] 2.4 設定 `cargo clippy --fix` 步驟並搭配 `stefanzweifel/git-auto-commit-action@v5` 自動推回分支，commit 訊息帶有 `[skip ci]`
- [x] 2.5 設定嚴格的 `cargo clippy --all-targets -- -D warnings` 與 `cargo test` 步驟

## 3. 實作前端 Tauri 檢查與測試 Job

- [x] 3.1 在 `ci.yml` 中設定 `frontend-ci` job，以 `stock-tauri` 作為 working directory
- [x] 3.2 設定 Node 22 與 npm 快取
- [x] 3.3 設定 `npm ci` 與 `npm run build`（型別檢查與編譯）步驟
- [x] 3.4 設定 `npm run test`（vitest）單元測試步驟

## 4. 驗證與修復當前代碼警告

- [x] 4.1 在本地端執行 `cargo clippy --fix --allow-dirty` 修正當前已存在的 clippy 警告
- [x] 4.2 本地執行 `cargo fmt` 與 `cargo test` 確保變更皆正常
- [x] 4.3 將變更提交並推送，在 GitHub 上驗證新建立的 CI 流程是否可成功通過且無窮迴圈機制防範有效
