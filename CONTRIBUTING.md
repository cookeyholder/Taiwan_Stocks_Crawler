# 貢獻指南

感謝您對本專案的興趣！本指南將幫助您設定開發環境並進行貢獻。

## 開發環境設置

### Rust 開發環境

```bash
# 安裝 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Windows 系統依賴

此專案使用 `x86_64-pc-windows-gnu` Rust 目標，需要 MSYS2 提供建置工具。

```powershell
# 1. 安裝 MSYS2（若尚未安裝）
#    從 https://www.msys2.org/ 下載並安裝

# 2. 在 MSYS2 終端安裝所需套件
pacman -S mingw-w64-x86_64-gcc mingw-w64-x86_64-binutils --noconfirm

# 3. 將 MSYS2 bin 目錄加入系統 PATH（以系統管理員身分執行 PowerShell）
[Environment]::SetEnvironmentVariable(
    "Path",
    [Environment]::GetEnvironmentVariable("Path", "User") + ";C:\msys64\mingw64\bin",
    "User"
)
```

重新啟動終端後驗證：
```bash
dlltool --version    # 應顯示 GNU dlltool 版本
gcc --version        # 應顯示 MinGW-w64 gcc 版本
```

#### Linux（Ubuntu/Debian）

```bash
# GPUI 系統相依套件 (Ubuntu/Debian)
sudo apt install build-essential pkg-config libxkbcommon-x11-0 libvulkan1 curl
```

#### 驗證

```bash
cargo --version
rustc --version
```

### 使用 Dev Container（歷史 Python 開發）

Dev Container 仍保留於 `.devcontainer/`，僅供參考 Python 版原始碼。
Rust / GPUI 開發建議在本機環境進行。

---

## 開發工作流程

### 1. 建立分支

```bash
git checkout -b feature/功能名稱
# 或
git checkout -b fix/bug名稱
```

### 2. 進行變更

修改 Rust 程式碼（`stock-core/`、`stock-cli/`、`stock-gpui/`）。

### 3. 執行測試

```bash
# 全專案測試（核心庫與桌面端）
cargo test --workspace

# Lint
cargo clippy --workspace

# 格式化檢查
cargo fmt --all -- --check
```

### 4. 執行編譯檢查

```bash
cargo check --workspace
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

## GPUI 開發模式

```bash
cargo run -p stock-gpui
```

### 常用開發指令

```bash
# 本地發行打包建置
./scripts/build-linux.sh       # Linux
powershell ./scripts/build-windows.ps1 # Windows
```

---

## 常用命令速查

### Rust / GPUI

| 命令 | 說明 |
|------|------|
| `cargo test --workspace` | 執行所有 Rust 單元與整合測試 |
| `cargo clippy --workspace` | Rust lint 檢查 |
| `cargo fmt --all` | Rust 程式碼格式化 |
| `cargo run -p stock-gpui` | 啟動 GPUI 桌面應用程式 |
| `cargo build -p stock-cli --release` | 建置 CLI 發行版本 |

### Makefile

| 命令 | 說明 |
|------|------|
| `make gpui-run` | 啟動 GPUI 桌面應用 |
| `make gpui-build` | 建置 Linux 正式發行版 |
| `make stock-cli ARGS="--help"` | 執行 CLI |
| `make cargo-test` | 執行測試 |
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
5. 根據審查意見進行調整並推送新提交

## 資源連結

### Rust / GPUI

- [Rust 官方](https://www.rust-lang.org/)
- [GPUI-Kit](https://github.com/longbridgeapp/gpui-kit)
- [clap CLI 框架](https://docs.rs/clap/)

---

感謝您的貢獻！如有任何問題，請開啟 issue 或聯繫維護者。
