# twse-crawler CLI 使用說明

`twse-crawler` 是臺股每日收盤行情爬蟲的命令列入口，
安裝後可直接使用 `twse-crawler` 指令抓取 TWSE 與 TPEX 資料並寫入 SQLite。

## 安裝

```bash
uv sync --all-extras
# 之後直接使用：
twse-crawler --help
```

## 執行模式

四種模式互斥，擇一使用。

### --today（建議排程使用）

抓取今日資料。**建議排程在 18:00 後執行**，確保收盤資料已更新。

```bash
twse-crawler --today
twse-crawler --today --db /data/market.db
```

### --date YYYY-MM-DD

抓取指定單日。

```bash
twse-crawler --date 2026-03-17
twse-crawler --date 2026-03-17 --db /data/market.db
```

### --backfill N

從今日往前 N 天（含今日），適合首次建立歷史資料。

```bash
twse-crawler --backfill 30
twse-crawler --backfill 90 --db /data/market.db
```

### --start / --end

指定任意日期區間。

```bash
twse-crawler --start 2026-01-01 --end 2026-03-31
```

## 共用引數

| 引數 | 說明 | 預設值 |
|------|------|--------|
| `--db PATH` | SQLite 資料庫路徑 | `market_data.db` |
| `--holidays DATE ...` | 手動指定假日（ISO 格式，可多個） | 無 |
| `--no-auto-holidays` | 停用自動抓取 TWSE holidaySchedule | 否 |
| `--timeout SECS` | HTTP 逾時秒數 | 30 |
| `--retries N` | HTTP 重試次數 | 3 |

## 自動假日抓取

預設情況下，每次執行都會嘗試從 TWSE `holidaySchedule` API 抓取當年國定假日，
並與 `--holidays` 指定的手動清單合併。

若 API 失敗，系統會發出 warning 但繼續執行（不中斷）。

如需停用自動抓取（例如離線環境或 CI 環境）：

```bash
twse-crawler --today --no-auto-holidays
twse-crawler --today --no-auto-holidays --holidays 2026-04-04 2026-04-05
```

## 資料來源路由

| 情況 | TWSE 資料來源 | TPEX 資料來源 |
|------|--------------|--------------|
| `date == today` | TWSE JSON API (`STOCK_DAY_ALL`) | TPEX CSV API（不變） |
| `date < today` | TWSE 歷史 CSV API (`MI_INDEX?date=YYYYMMDD`) | TPEX CSV API（不變） |

## 排程設定建議（cron）

```cron
# 每個工作日 18:30 抓取當日資料
30 18 * * 1-5 /path/to/.venv/bin/twse-crawler --today --db /data/market.db
```

若要在 18:00 前執行（收盤資料尚未刷新），TWSE JSON API 可能回傳前一交易日資料，
此時該日會被標為 `market_closed`，不計入 `source_error`。

## 輸出格式

```
  ✓ trading        ok
  ✓ trading        ok
  ✗ weekend        skip

完成：成功 2 天，休市 0 天，失敗 0 天
```

## python -m 執行方式

不安裝套件時也可直接執行：

```bash
python -m market_crawler.pipeline --today
```
