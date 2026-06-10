# CONTRIBUTING.md 更新規格

## 開發環境設置

SHALL 在 Python 段落前新增 Rust 開發環境：
```bash
# Rust 工具鏈
rustup default stable
cargo install --locked trunk

# 驗證
cargo --version
rustc --version
```

## 開發工作流程

SHALL 新增 Rust 相關指令：

### 執行 Rust 單元測試
```bash
cargo test -p stock-core
# 或全部
cargo test --workspace
```

### 啟動 Tauri 開發模式
```bash
cd stock-tauri && npm run tauri dev
```

### 執行 Rust CLI
```bash
cargo run -p stock-cli -- --help
```

## 常用命令速查

SHALL 在表格中新增：
| `cargo test` | 執行 Rust 單元測試 |
| `make tauri-dev` | 啟動 Tauri 開發模式 |
| `make tauri-build` | 建置 Tauri 正式版 |
| `make stock-cli ARGS=--help` | 執行 Rust CLI |

## 程式碼風格

SHALL 新增 Rust 風格說明：
- Rust edition 2024
- 使用 `cargo fmt` 格式化
- 使用 `cargo clippy` 靜態分析
