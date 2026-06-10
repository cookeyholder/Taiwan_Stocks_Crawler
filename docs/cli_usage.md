# stock-cli CLI 使用說明

`stock-cli` 是臺股選股工具的 Rust 命令列入口，
支援資料抓取、布林擠壓選股與批次補檔。

## 安裝

```bash
cargo build -p stock-cli --release
# 輸出：target/release/stock-cli
```

## 子指令

### fetch — 抓取收盤資料

```bash
# 今日資料
stock-cli fetch --today

# 指定日期
stock-cli fetch --date 2026-06-10

# 客製資料庫路徑
stock-cli fetch --today --db-path /tmp/market.db
```

### screen — 布林擠壓選股

```bash
# 預設條件（bandwidth <= 0.1、連續 3 天）
stock-cli screen

# 自訂條件
stock-cli screen --bandwidth-limit 0.05 --consecutive-days 5 --min-volume 1000000 --instrument-types stock,etf
```

### sync — 批次補檔

```bash
stock-cli sync --start 2026-01-01 --end 2026-06-10
```

## 共用選項

| 選項 | 說明 | 預設值 |
|------|------|--------|
| `--db-path PATH` | SQLite 資料庫路徑 | `~/.local/share/stock-screener/market_data.db` |

## 資料來源路由

| 情況 | TWSE 資料來源 | TPEX 資料來源 |
|------|--------------|--------------|
| `date == today` | TWSE JSON API (`STOCK_DAY_ALL`) | TPEX JSON API (`dailyQuotes?response=json`) |
| `date < today` | TWSE 歷史 CSV API (`MI_INDEX?date=YYYYMMDD`) | TPEX JSON API (`dailyQuotes?response=json`) |

## 排程設定建議（cron）

```cron
# 每個工作日 18:30 抓取當日資料
30 18 * * 1-5 /path/to/stock-cli fetch --today
```

---

## 歷史參考：twse-crawler（Python 版）

Python 版 `twse-crawler` 已於 2026-06 停止維護，以下內容僅供參考。

### 安裝方式

```bash
uv sync --all-extras
twse-crawler --help
```

### 執行模式

```bash
# 今日資料
twse-crawler --today

# 指定日期
twse-crawler --date 2026-03-17

# 批次補檔
twse-crawler --backfill 30

# 日期區間
twse-crawler --start 2026-01-01 --end 2026-03-31
```

### Python 執行方式

```bash
python -m market_crawler.pipeline --today
```
