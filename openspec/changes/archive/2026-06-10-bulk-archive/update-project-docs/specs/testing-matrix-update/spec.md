# 測試矩陣更新規格（docs/testing_matrix.md）

## 檔案結構

SHALL 新增「Rust 測試」章節在 Python 測試之前：

### Rust 單元測試

| 測試模組 | 案例數 | 說明 |
|----------|--------|------|
| `analysis::tests` | 8 | SMA/EMA/Bollinger/RSI/MACD |
| `db::migrations::tests` | 2 | 資料庫 migration 冪等性 |
| `db::tests` | 4 | CRUD、screen_bollinger_squeeze |
| `fetcher::twse::tests` | 6 | CSV/HTML 解析、metadata 過濾 |
| `pipeline::tests` | 11 | 數值解析、change_field、normalize |

**合計：29 項（截至 2026-06-10）**

### 執行方式
```bash
cargo test -p stock-core
```

SHALL 保留現有 Python 測試內容作為「歷史測試」章節。
