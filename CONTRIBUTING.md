# 貢獻指南

感謝您對本專案的興趣！本指南將幫助您設定開發環境並進行貢獻。

## 開發環境設置

### Rust 開發環境

```bash
# 安裝 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安裝 Tauri Linux 依賴（Ubuntu/Debian）
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev

# 安裝 Node.js 相依
cd stock-tauri && npm install

# 驗證
cargo --version
node --version
npm --version
```

### 使用 Dev Container（歷史 Python 開發）

Dev Container 仍保留於 `.devcontainer/`，僅供參考 Python 版原始碼。
Rust/Tauri 開發建議在本機環境進行。

---

## 開發工作流程

### 1. 建立分支

```bash
git checkout -b feature/功能名稱
# 或
git checkout -b fix/bug名稱
```

### 2. 進行變更

修改 Rust 程式碼（`stock-core/`、`stock-cli/`、`stock-tauri/src-tauri/`）
或 Vue 前端（`stock-tauri/src/`）。

### 3. 執行測試

```bash
# Rust 單元測試（29 項）
cargo test -p stock-core

# Lint
cargo clippy -p stock-core
cargo clippy -p stock-cli
cargo clippy -p stock-tauri

# 格式化
cargo fmt -p stock-core
cargo fmt -p stock-cli
cargo fmt -p stock-tauri

# Python 測試（已停止維護，僅供歷史參考）
make test
```

### 4. 執行前端建置檢查

```bash
cd stock-tauri
npm run build
npm run typecheck   # 若已設定
```

### 5. 提交變更

```bash
git add .
git commit -m "說明性的提交訊息"
```

提交訊息應遵循以下格式:
- `feat:` 新功能說明
- `fix:` bug 修復說明
- `docs:` 文件更新
- `test:` 測試相關變更
- `refactor:` 程式碼重構

---

## Tauri 開發模式

```bash
cd stock-tauri
npm run tauri dev
```

這會啟動：
1. Vite dev server（Vue 前端 hot-reload）
2. Tauri Rust 後端（含自動重新編譯）

### 常用開發指令

```bash
# 建置正式版本
npm run tauri build

# 預覽 Rust 後端日誌
# 可在 src-tauri/src 中加入 println! 或 tauri::log 輸出
```

---

## 常用命令速查

### Rust / Tauri

| 命令 | 說明 |
|------|------|
| `cargo test -p stock-core` | 執行 Rust 單元測試 |
| `cargo clippy -p stock-core` | Rust lint |
| `cargo fmt -p stock-core` | Rust 格式化 |
| `cd stock-tauri && npm run tauri dev` | 啟動桌面應用開發模式 |
| `cd stock-tauri && npm run tauri build` | 建置正式桌面應用 |
| `cargo build -p stock-cli --release` | 建置 CLI |

### Makefile

| 命令 | 說明 |
|------|------|
| `make tauri-dev` | 啟動 Tauri 開發模式 |
| `make tauri-build` | 建置正式版 |
| `make stock-cli ARGS="--help"` | 執行 CLI |
| `make cargo-test` | Rust 單元測試 |
| `make help` | 顯示所有命令 |

---

## 歷史參考（Python 開發流程）

Python 原始碼已於 2026-06 停止主動維護並移除，歷史版本可透過 Git 檢視。
以下內容保留為歷史參考：

### 原始開發環境

- Python 3.13+、uv 套件管理
- `uv sync --all-extras` 安裝依賴
- `make test` 執行 pytest（181 項測試）
- `make check` ruff lint
- `make format` ruff format

---

## 提交 Pull Request

1. 推送您的分支到遠端
2. 開啟 Pull Request
3. 填寫 PR 描述:
   - 說明變更內容
   - 列舉相關的 issue
   - 描述測試方法

4. 等待程式碼審查
5. 根據反饋進行調整並推送新提交

## 資源連結

### Rust / Tauri

- [Rust 官方](https://www.rust-lang.org/)
- [Tauri v2 文檔](https://v2.tauri.app/)
- [clap CLI 框架](https://docs.rs/clap/)

### 前端

- [Vue 3](https://vuejs.org/)
- [ECharts](https://echarts.apache.org/)

---

感謝您的貢獻！如有任何問題，請開啟 issue 或聯繫維護者。
