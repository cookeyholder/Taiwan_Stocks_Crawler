## Context

`instrument_type` 欄位目前硬編碼為 `"other"`，使 `WHERE instrument_type IN (...)` 篩選永遠不命中。TPEX 端只有「上櫃股票」表格被處理，ETF 等資料被整張跳過。

## Goals / Non-Goals

**Goals:**
- 在 `normalize_records` 中根據規則設定正確的 `instrument_type`
- TPEX parser 放寬表格限制，依 table title 對應到正確類型
- TWSE 資料依 symbol prefix 規則分類
- 前端 ScreenerView 新增類型篩選 UI

**Non-Goals:**
- 不修改 `ParsedRecord` 結構（分類在 `normalize_records` 層處理）
- 不重新擷取歷史資料（新分類僅影響新寫入的資料）

## Decisions

| 決策 | 選擇 | 理由 |
|------|------|------|
| 分類方式 | TPEX: table title；TWSE: symbol prefix | TPEX 的 table title 直接宣告類型；TWSE 無分類欄位，以 symbol prefix 推斷 |
| TPEX 表格映射 | `"上櫃股票"` → `stock`；`"上櫃ETF"` → `etf`；`"興櫃"` → `stock` | 符合台灣市場慣例；其餘表格分類為 `other` |
| TWSE symbol 規則 | `^0\d{3}$`(4 位數 0 開頭) → `etf`；`^03\d{3}$`(5 位數 03 開頭) → `warrant`；`^02\d{3}$`(5 位數 02 開頭) → `etn`；其餘 4 位數 → `stock` | 台灣證交所常見分類慣例 |
| 歷史資料 | 不回溯修改 | 改動不影響舊有分析邏輯，新擷取資料自動依新規則分類 |

## Classification Rules

```
normalize_records():
  ┌─ TPEX data ─────────────────────────┐
  │ table title                          │
  │   "上櫃股票"           → stock       │
  │   "上櫃ETF"            → etf         │
  │   "興櫃"               → stock       │
  │   其他                  → other      │
  └──────────────────────────────────────┘

  ┌─ TWSE data ─────────────────────────┐
  │ symbol pattern                       │
  │   4 digits, starts with 0 (0050)    → etf          │
  │   5 digits, starts with 03 (03001)  → warrant      │
  │   5 digits, starts with 02 (02001)  → etn          │
  │   4 digits, starts with 1-9 (2330) → stock        │
  │   其他                              → other        │
  └──────────────────────────────────────┘

  ┌─ Fallback ──────────────────────────┐
  │   以上皆非                           → other        │
  └──────────────────────────────────────┘
```

## Risks / Trade-offs

- [Symbol 誤判] TWSE 的 symbol prefix 規則是 heuristic，少數標的可能誤分類 → 回退 `other` 是最保守的處理
- [歷史資料不一致] 已存在的資料仍為 `"other"`，篩選時會遺漏部分舊資料 → 僅影響新擷取，可接受
- [TPEX 表格名稱變動] TPEX 若修改 table title 字串 → 新增對應的 `contains` 匹配
