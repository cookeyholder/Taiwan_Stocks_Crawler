# 文件更新設計

## 受影響文件

| 文件 | 現狀 | 更新內容 |
|------|------|----------|
| `README.md` | 僅 Python | 加入 Tauri 桌面應用、Rust CLI 說明、建置指南 |
| `CONTRIBUTING.md` | Python dev workflow | 加入 Rust workspace、cargo test、Tauri dev 流程 |
| `docs/cli_usage.md` | 僅 `twse-crawler` | 新增 `stock-cli` 章節，Python CLI 歸檔 |
| `docs/testing_matrix.md` | 僅 Python 測試 | 加入 Rust 29 項測試，雙軌並陳 |

## 撰寫原則

- 全部使用 **zh-tw（繁體中文）**
- 保留 Python 歷史內容（移至各文件後段歸檔區）
- Tauri 內容放在前半段（新的主要內容）
- 程式碼片段標註 shell / powershell 語言
