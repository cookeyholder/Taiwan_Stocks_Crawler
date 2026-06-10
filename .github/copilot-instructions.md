# Copilot 開發指引

## 建置、測試與 lint 指令

本專案使用 Rust 2024 edition + Tauri v2 + Vue 3 + TypeScript。

### Rust

```bash
cargo test -p stock-core
cargo clippy -p stock-core
cargo fmt -p stock-core
```

### Tauri 桌面應用

```bash
cd stock-tauri && npm run tauri dev
cd stock-tauri && npm run tauri build
```

## OpenSpec 注意事項
- 規劃任務時，最後一項要是「更新文件」，並且在每項任務後加上「驗收標準」，明確定義在什麼情況下任務可以視為完成。請盡量採用可量化的標準；如果是質性的標準，也要條列清楚。
- 採用 TDD 流程，先寫測試再實作，確保每個子任務都有對應的 failing test，並且在實作後驗證所有測試都通過。
- apply 提案時，先建立一個新的開發分支，然後在該分支上進行開發與測試，先提交提案文件再開始實作，確保所有測試都通過後再由開發者決定何時發起 PR 合併回 main 分支。
- 除了單元測試之外，也要有整合測試、流程測試與防呆測試，並且盡量涵蓋已知的錯誤案例。
- 在撰寫計畫時，請確保涵蓋所有必要步驟、相關檔案、驗證方式、決策與後續考量，讓使用者能完整理解並有效執行。
- 在使用澄清工具時，請確保問題具體且提供足夠的背景資訊，以便使用者能夠給出明確的回答。
- 在撰寫驗證步驟時，請盡量具體，包含明確的測試案例、指令或 MCP 工具，讓驗證流程有效且可執行。
- 採取高頻提交的策略，每完成一個子任務就提交一次，確保每次提交都能通過所有測試，並且在 commit message 清楚說明變更內容與目的。

## 提交 commit 時的注意事項
- 使用繁體中文撰寫 commit message，必須使用臺灣慣用語，避免出現中國常用語。
- 請遵循專案既有的 conventional commit 風格，使用 `feat:`、`fix:`、`docs:`、`test:`、`refactor:`、`chore:` 等類型標籤。
- 在 commit message 中清楚說明變更內容與目的，可以的話也提到相關測試或文件更新。
- 如果 commit 涉及特定任務或問題，請在提交訊息中引用相關 issue 或任務編號，方便追蹤管理。
- 提交前請確認所有相關測試都已通過，且程式碼也已通過 lint 檢查，確保提交品質符合專案標準。

## 整體架構

```
Cargo workspace
├── stock-core/     # Rust 核心函式庫
│   ├── analysis/   # 技術指標（SMA/EMA/Bollinger/RSI/MACD）
│   ├── db/         # SQLite CRUD + migration
│   ├── fetcher/    # HTTP 爬蟲（TWSE/TPEX，含 Big5 解碼）
│   └── pipeline/   # 端到端管線
├── stock-cli/      # Rust CLI（clap derive）
└── stock-tauri/    # Tauri v2 桌面殼層
    ├── src-tauri/  # Rust 後端（7 個 Tauri commands）
    └── src/        # Vue 3 + TypeScript 前端
        ├── views/  # 三頁簽（分析/選股/下載）
        └── components/  # ECharts 圖表、參數表單、篩選表格
```

## 重要慣例

- `stock-core/src/db/` 使用 rusqlite + 手動 migration（CREATE TABLE IF NOT EXISTS）
- TWSE/TPEX 爬蟲支援 Big5 解碼，使用 `encoding_rs` crate
- 測試使用 `cargo test -p stock-core`，無外部依賴（mock 為主）
- SQLite schema 具備冪等性，upsert 使用 `ON CONFLICT ... DO UPDATE`
- 請遵循 `CONTRIBUTING.md` 的 Rust/Tauri 開發流程

## 文件更新
- 在 `docs/` 中撰寫與更新文件，並且在 commit message 中清楚說明文件更新的內容與目的。
- 文件內容應該清晰、具體且易於理解，並且盡量包含實際的範例與參考，讓使用者能夠有效地使用這些文件來理解專案的規格、流程與最佳實踐。
