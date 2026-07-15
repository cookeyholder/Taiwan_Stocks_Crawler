## Context

`instrument_type` 欄位目前硬編碼為 `"other"`，使 `WHERE instrument_type IN (...)` 篩選永遠不命中。TPEX 端只有「上櫃股票」表格被處理，ETF 等資料被整張跳過。

## Goals / Non-Goals

**Goals:**
- 在 `normalize_records` 中根據規則設定正確的 `instrument_type`
- TPEX parser 放寬表格限制，依 table title 對應到正確類型
- TWSE 資料依 symbol prefix 規則分類
- 前端 ScreenerView 新增類型篩選 UI

**Non-Goals:**
- 不重新擷取歷史資料（新分類僅影響新寫入的資料；但重新 fetch 舊日期時 Upsert 會自動覆蓋）

## Decisions

| 決策 | 選擇 | 理由 |
|------|------|------|
| 分類方式 | TPEX: table title；TWSE: symbol prefix | TPEX 的 table title 直接宣告類型；TWSE 無分類欄位，以 symbol prefix 推斷 |
| TPEX 表格映射 | `"上櫃股票"` → `stock`；`"上櫃ETF"` → `etf`；`"興櫃"` → `stock` | 符合台灣市場慣例；其餘表格分類為 `other` |
| TWSE symbol 規則 | 以 `0` 開頭（如 0050、00878）→ `etf`；以 `03` 開頭的 5 位數 → `warrant`；以 `02` 開頭的 5 位數 → `etn`；4 位數首位 1-9 → `stock` | 台灣證交所常見分類慣例；修正為不限位數的 0 開頭以涵蓋 00878、00940 等 5 位數 ETF |
| 歷史資料 | 不回溯修改 | 改動不影響舊有分析邏輯，新擷取資料自動依新規則分類 |
| TPEX title 傳遞 | `ParsedRecord` 新增 `instrument_type: Option<String>` 欄位 | `normalize_records` 無法取得 table title，TPEX parser 解析 JSON 時直接填入；TWSE 路徑為 `None`，由 symbol 規則補上 |
| 前端預設值 | `instrumentTypes` 預設為 `["stock"]` | 符合 spec「預設勾選 stock 類型」；使用者可自行增減 |

## Classification Rules

### TPEX: table title → `ParsedRecord.instrument_type`（在 parser 中設定）

| Table title | instrument_type |
|-------------|----------------|
| `"上櫃股票"` | `stock` |
| `"上櫃ETF"`  | `etf` |
| `"興櫃"`     | `stock` |
| 其他         | `other` |

### TWSE: symbol prefix → `normalize_records` 推斷

| Symbol pattern | Length | Example | instrument_type |
|----------------|--------|---------|----------------|
| Starts with `0` | 4+ digits | `0050`, `00878` | `etf` |
| Starts with `03` | 5 digits | `03001` | `warrant` |
| Starts with `02` | 5 digits | `02001` | `etn` |
| Starts with `1-9` | 4 digits | `2330` | `stock` |
| 以上皆非 | — | — | `other` |

## Data Flow

```
TPEX API
  ↓
parse_tpex_json()
  → 不再 skip 非「上櫃股票」表格
  → 每個 table title 決定此批資料的 instrument_type
  → ParsedRecord { ..., instrument_type: Some("etf") }
  ↓
TWSE API
  ↓
parse_twse_csv() / parse_twse_html()
  → ParsedRecord { ..., instrument_type: None }  ← 無分類資訊
  ↓
normalize_records(market_code, records)
  → for each record:
      if record.instrument_type is Some → 直接使用（TPEX 已分類）
      else → 依 market_code + symbol prefix 規則推斷（TWSE）
      → CanonicalQuote { instrument_type, ... }
  ↓
DB upsert_core_batch()
  → instrument_type 寫入 market_quote_core
```

### 前端路徑

```
ScreenerView.vue
  → 預設勾選 stock 類型
  → screenerStore.instrumentTypes = ["stock"]
    → invoke("screen_bollinger", { instrumentTypes: ["stock"], ... })
      → Pipeline::screen(... instrument_types=["stock"])
        → get_active_symbols(start_date, Some(["stock"]))
          → WHERE instrument_type IN ('stock')
```

## Risks / Trade-offs

- [Symbol 誤判] TWSE 的 symbol prefix 規則是 heuristic，少數標的可能誤分類 → 回退 `other` 是最保守的處理
- [歷史資料不一致] 已存在的資料仍為 `"other"`，篩選時會遺漏部分舊資料 → 僅影響新擷取，可接受；重新 fetch 舊日期會自動覆蓋
- [TPEX 表格名稱變動] TPEX 若修改 table title 字串 → 新增對應的 `contains` 匹配
- [ParsedRecord 改動] 新增欄位影響序列化/反序列化，需確認 serde 相容
