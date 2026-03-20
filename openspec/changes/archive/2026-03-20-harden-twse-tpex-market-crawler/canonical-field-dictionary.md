# Canonical 欄位字典與來源對映

## Core 欄位

| 欄位 | 型別 | 說明 | TWSE 對應 | TPEX 對應 |
|---|---|---|---|---|
| trade_date | TEXT(YYYY-MM-DD) | 交易日期（西元） | 外部參數/檔名日期 | 民國年日期轉西元 |
| market_code | TEXT | 市場代碼 | TWSE | TPEX |
| symbol_raw | TEXT | 原始代號（保真） | 證券代號（可為 `="0050"`） | 代號 |
| symbol_clean | TEXT | 清洗後代號（主鍵鍵值） | 去除 Excel 包裝與引號 | 去除空白 |
| security_name | TEXT | 商品名稱 | 證券名稱 | 名稱 |
| instrument_type | TEXT | 商品大類 | 分類器產出 | 分類器產出 |
| instrument_subtype | TEXT? | 商品子類 | 分類器產出 | 分類器產出 |
| isin_code | TEXT? | ISIN 代碼 | security_master | security_master |
| cfi_code | TEXT? | CFI 代碼 | security_master | security_master |
| section_label | TEXT? | 市場分類標籤 | security_master | security_master |
| open_price | REAL? | 開盤價 | 開盤價 | 開盤 |
| high_price | REAL? | 最高價 | 最高價 | 最高 |
| low_price | REAL? | 最低價 | 最低價 | 最低 |
| close_price | REAL? | 收盤價 | 收盤價 | 收盤 |
| volume_shares | INTEGER? | 成交股數 | 成交股數 | 成交股數 |
| turnover_value | REAL? | 成交金額 | 成交金額 | 成交金額(元) |
| trade_count | INTEGER? | 成交筆數 | 成交筆數 | 成交筆數 |
| change_sign | TEXT? | 漲跌符號 | 漲跌(+/-) | 漲跌拆解 |
| change_value | REAL? | 漲跌值 | 漲跌價差 | 漲跌拆解 |
| last_bid_price | REAL? | 最後買價 | 最後揭示買價 | 最後買價 |
| last_bid_qty_raw | TEXT? | 最後買量（原值） | 最後揭示買量 | 最後買量(張數) |
| last_ask_price | REAL? | 最後賣價 | 最後揭示賣價 | 最後賣價 |
| last_ask_qty_raw | TEXT? | 最後賣量（原值） | 最後揭示賣量 | 最後賣量(張數) |
| pe_ratio | REAL? | 本益比 | 本益比 | 無 |
| issued_shares | INTEGER? | 發行股數 | 無 | 發行股數 |
| next_limit_up | REAL? | 次日漲停價 | 無 | 次日漲停價 |
| next_limit_down | REAL? | 次日跌停價 | 無 | 次日跌停價 |
| source_file | TEXT | 來源檔名 | 檔案名 | 檔案名 |
| source_encoding | TEXT | 來源編碼 | cp950/big5hkscs | cp950/big5hkscs |
| source_line_no | INTEGER | 來源列號 | parser 產出 | parser 產出 |
| ingest_run_id | TEXT | 批次 ID | orchestrator 注入 | orchestrator 注入 |
| ingested_at | TEXT | 入庫時間 | orchestrator 注入 | orchestrator 注入 |

## Extension 欄位

| 欄位 | 型別 | 說明 |
|---|---|---|
| raw_change | TEXT? | 原始漲跌字串（保真） |
| raw_payload | TEXT? | parser 原始結構序列化 |

## 正規化規則

- 數值欄位先去逗號，`--`/空字串轉 `NULL`。
- 代號同時保留 `symbol_raw` 與 `symbol_clean`，避免 `="0050"` 遺失。
- 日期一律轉為西元 `YYYY-MM-DD`，DB 不存民國年。
