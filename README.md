# 臺股選股工具

臺灣股市每日收盤行情自動抓取、入庫與技術分析桌面應用。
支援 **TWSE**（臺灣證券交易所）與 **TPEX**（證券櫃買中心）資料，
提供圖形化介面與命令列工具進行資料查詢與布林擠壓選股。

## 功能特色

- **雙來源支援**：自動抓取 TWSE 與 TPEX 每日收盤行情
- **智慧路由**：今日資料走 JSON API，歷史資料走 CSV API（日期前後自動切換）
- **桌面應用**：純 Rust + GPU 加速桌面端（基於 GPUI-Kit，支援 K 線、布林通道、均線、成交量、RSI 與三大法人圖表，並提供自選股清單）
- **布林擠壓選股**：以 bandwidth 門檻篩選 squeezed 標的，支援成交量與商品類型過濾
- **技術指標**：SMA / EMA / Bollinger Bands / RSI / MACD（Rust 純函式計算，含單元測試）
- **CLI 工具**：`stock-cli` 支援 fetch、screen、sync 子指令
- **跨平台打包**：Linux 與 Windows 原生發行二進位檔

---

## 技術架構

```
Cargo workspace
├── stock-core/          # Rust 核心函式庫
│   ├── analysis/        # 技術指標（SMA/EMA/Bollinger/RSI/MACD）
│   ├── db/              # SQLite CRUD + migration
│   ├── fetcher/         # HTTP 爬蟲（TWSE/TPEX，含 Big5 解碼）
│   └── pipeline/        # 端到端管線
├── stock-cli/           # Rust CLI（clap derive）
└── stock-gpui/          # 純 Rust 桌面應用（GPUI-Kit v0.7+，GPU 加速）
    ├── src/
    │   ├── app.rs       # 主外框 AppView 與主題切換
    │   ├── state.rs     # 狀態模型 CoreState 與非同步協調
    │   └── views/       # 五大業務檢視畫面（個股分析/選股/回測/下載/證券清單）
    └── tests/           # 無頭 UI 整合測試
```

---

## 建置指南

### 前置需求

| 平台 | 需求 |
|------|------|
| **Linux** | Rust、`libxkbcommon-x11-0`、`libvulkan1` 等系統套件 |
| **Windows** | Rust（MSVC ABI）、Direct3D 12 支援 |

#### Linux

```bash
# 系統依賴（Ubuntu/Debian）
sudo apt install build-essential pkg-config libxkbcommon-x11-0 libvulkan1 curl

# 安裝 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

#### Windows

1. 安裝 [rustup](https://rustup.rs)
2. 安裝 Visual Studio Build Tools，勾選「Desktop development with C++」

### 建置桌面應用

```bash
# 透過平臺腳本進行正式發行建置（產出至 dist/）
./scripts/build-linux.sh       # Linux
powershell ./scripts/build-windows.ps1 # Windows

# 或直接使用 Cargo 建置
cargo build --release -p stock-gpui
```

輸出路徑：
- Linux：`dist/stock-gpui-linux` 或 `target/release/stock-gpui`
- Windows：`dist/stock-gpui-windows.exe` 或 `target/release/stock-gpui.exe`

### 建置 CLI

```bash
cargo build -p stock-cli --release
# 輸出：target/release/stock-cli（Linux）或 stock-cli.exe（Windows）
```

---

## 快速開始

### 桌面應用模式

```bash
cargo run -p stock-gpui
```

### CLI 模式

```bash
# 今日資料
cargo run -p stock-cli -- fetch --today

# 布林擠壓選股
cargo run -p stock-cli -- screen --bandwidth-limit 0.1 --consecutive-days 3

# 批次補檔
cargo run -p stock-cli -- sync --start 2026-01-01 --end 2026-06-10
```

### Rust 測試

```bash
cargo test -p stock-core
```

---

## 命令列工具（stock-cli）

`stock-cli` 為 Rust 實作的 CLI，提供以下子指令：

### fetch

```bash
stock-cli fetch --today
stock-cli fetch --date 2026-06-10
```

### screen

```bash
stock-cli screen --bandwidth-limit 0.1 --consecutive-days 3
stock-cli screen --bandwidth-limit 0.05 --consecutive-days 5 --min-volume 1000000 --instrument-types stock,etf
```

### sync

```bash
stock-cli sync --start 2026-01-01 --end 2026-06-10
```

### 共用選項

| 選項 | 說明 | 預設值 |
|------|------|--------|
| `--db-path PATH` | SQLite 資料庫路徑 | `~/.local/share/stock-screener/market_data.db` |

### 排程範例（cron）

```cron
# 每日 18:10 自動抓取今日收盤資料
10 18 * * 1-5 /path/to/stock-cli fetch --today
```

---

## 資料來源路由

| 請求日期 | TWSE 來源 | TPEX 來源 |
|----------|----------|----------|
| 今日 | JSON API（`openapi.twse.com.tw`） | JSON API（`dailyQuotes?response=json`） |
| 歷史日期 | CSV API（`MI_INDEX?response=csv`） | JSON API（`dailyQuotes?response=json`） |

---

## 資料庫 Schema

資料分兩張表儲存：

**`market_quote_core`** — 標準化報價欄位（主鍵：`trade_date + market_code + symbol_clean`）

| 欄位 | 說明 |
|------|------|
| `trade_date` | 西元格式交易日（`YYYY-MM-DD`） |
| `market_code` | `TWSE` 或 `TPEX` |
| `symbol_clean` | 標準股票代號 |
| `symbol_raw` | 原始代號 |
| `instrument_type` | 商品類型（`stock`、`etf`、`warrant` 等） |
| `close_price` | 收盤價 |
| `volume_shares` | 成交股數 |

**`market_quote_extension`** — 原始 payload 與附加中繼資料

---

## 開發

### Rust 開發

```bash
# 完整測試（核心庫與桌面端）
cargo test --workspace

# Lint
cargo clippy -p stock-core -p stock-cli -p stock-gpui

# 格式化
cargo fmt --all
```

### GPUI 桌面應用

```bash
cargo run -p stock-gpui
```

### Makefile 指令

| 指令 | 說明 |
|------|------|
| `make gpui-run` | 啟動 GPUI 桌面應用 |
| `make gpui-build` | 建置 Linux 正式發行版 |
| `make stock-cli ARGS="--help"` | 執行 CLI |
| `make cargo-test` | Rust 整合測試 |

---

## 歷史參考（Python 版）

本專案原為 Python Tkinter 實作，已於 2026-06 遷移至 Tauri + Rust。
Python 原始碼已於 2026-06 移除，歷史版本可透過 Git 檢視。

### 原始架構

```
market_crawler/
├── http_client.py
├── parsers.py
├── encoding.py
├── normalize.py
├── classification.py
├── repository.py
├── orchestrator.py
├── pipeline.py
├── quality.py
├── models.py
└── fetch_security_master.py
```

### 原始 CLI（twse-crawler）

```bash
twse-crawler --today
twse-crawler --date 2026-03-17
twse-crawler --backfill 30
twse-crawler --start 2026-01-01 --end 2026-03-31
```

### 原始測試（181 項 Python pytest）

詳見 [docs/development/測試.md](docs/development/測試.md)。

---

## 貢獻

詳見 [CONTRIBUTING.md](CONTRIBUTING.md)。

本專案採用：
- **TDD 流程**：先寫 failing test，再實作
- **Conventional Commits**：`feat:`、`fix:`、`test:`、`docs:`、`refactor:`
- **高頻提交**：每完成一個子任務即提交，確保每次提交都通過所有測試

## 授權

本專案為個人開發學習用途。
