# CLI 使用說明更新規格（docs/cli_usage.md）

## 檔案結構

SHALL 分為兩節：
1. **Tauri CLI（stock-cli）** — 主要內容，置前
2. **Python CLI（twse-crawler）** — 歷史參考，置後

## stock-cli 說明

SHALL 包含以下內容：

### 安裝／建置
```bash
cargo build -p stock-cli --release
# 或
make stock-cli ARGS="--help"
```

### 子指令
- `stock-cli fetch --today` — 抓取今日資料
- `stock-cli fetch --date YYYY-MM-DD` — 抓取指定日期
- `stock-cli screen --bandwidth-limit 0.1 --consecutive-days 3` — 布林擠壓選股
- `stock-cli sync --start YYYY-MM-DD --end YYYY-MM-DD` — 批次補檔

SHALL 說明 `--db-path` 自訂資料庫位置，預設為 `~/.local/share/stock-screener/market_data.db`。

## Python CLI 保留區

SHALL 將現有 `twse-crawler` 內容移至「歷史參考」章節，標註「已停止維護」。
