## Context

本專案是一個整合了 Rust（核心邏輯與命令列工具）以及 Vue/TypeScript Tauri（桌面應用）的雙端專案。
目前僅有針對發布 Tag（`v*`）的 `build.yml` 工作流，缺乏日常開發 Commit 或 Pull Request 的自動化防呆檢查，導致格式錯誤、未通過測試、或 Clippy 警告（在 strict 模式下會阻礙編譯）等程式碼容易被合併。

## Goals / Non-Goals

**Goals:**
- 在 GitHub Actions 中建立日常 CI 工作流（`.github/workflows/ci.yml`）。
- 當 Rust 程式碼有 Clippy 警告且可自動修復時，由 CI 自動跑 `cargo clippy --fix` 並提交（Auto-commit）回原分支，減少人工修正瑣碎風格警告的時間。
- 使用 `[skip ci]` 防止 auto-commit 觸發無窮迴圈的 CI 執行。
- 前端 Tauri 部分自動執行 TypeScript 靜態分析與單元測試。
- 採用合適的快取機制優化 CI 執行時間。

**Non-Goals:**
- 不打包產出 Release 檔（維持原有的 `build.yml` 處理）。
- 不對 legacy 的 Python 代碼進行 CI 檢查。

## Decisions

### 1. 雙 Job 平行架構
我們將 CI 分為兩個獨立的 Job：`rust-ci` 與 `frontend-ci`。
- **原因**：Rust 與 Node.js 依賴互相獨立。平行執行能加快反饋速度。

### 2. Auto-commit 策略防範無窮迴圈
當 Rust Clippy 發出警告時，我們利用 `cargo clippy --fix` 進行自動修正，並透過 `stefanzweifel/git-auto-commit-action@v5` 推送回分支。
- **預防無窮迴圈**：Commit 訊息固定格式為 `style: [skip ci] auto fix clippy warnings`。GitHub 預設不為帶有 `[skip ci]` 的 commit 觸發 workflow。
- **備用選擇**：原本考慮 reviewdog suggestion 模式，但 auto-commit 能最大化減少手動工作，因此採用 auto-commit。

### 3. Rust & Node 依賴快取
- **Rust**：使用 `swatinem/rust-cache@v2`，它會自動將 Cargo 工作區之 `target/` 與 registry 快取保存。
- **Node.js**：使用 `actions/setup-node@v4` 的 `cache: npm` 快取 npm 下載快取（`~/.npm`），而非直接快取 `node_modules`。

## Risks / Trade-offs

- **[Risk] Git Conflict 衝突**
  - **描述**：當 CI 正在自動修正 Rust 程式碼並推送時，若開發者在本地也做了修改並同時 push，會引發衝突或需要手動 git pull。
  - **Mitigation**：Clippy auto-commit 僅會在開發者推上了「未格式化或有簡單風格警告」的 Rust 程式碼時發生。我們可以在專案開發守則中說明：建議開發者在 push 後，先等待 CI 執行完成（通常 1-2 分鐘內），或是本地先跑 `cargo clippy --fix` 與 `cargo fmt` 確保代碼乾淨再 Push。
- **[Risk] Fork PR 權限不足**
  - **描述**：來自 fork 儲存庫的 PR 會因為 `GITHUB_TOKEN` 沒有寫入權限而無法 auto-commit。
  - **Mitigation**：如果為 fork PR，`git-auto-commit-action` 會安全地跳過而不出錯，但後續的嚴格 clippy 步驟會因為仍有 warning 而標記失敗，這能保證程式碼品質。
