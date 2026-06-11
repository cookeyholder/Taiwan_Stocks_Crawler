## Context

`tpex.rs` 的 TPEX JSON API 欄位索引直接複製自 TWSE CSV 的 `parse_twse_csv()`。但 TPEX JSON API 的回應使用 `fields` 陣列宣告欄位名稱，其順序與 TWSE CSV 完全不同。

### 當前錯誤對照

TPEX JSON `fields` 陣列：`["代號","名稱","收盤","漲跌","開盤","最高","最低","均價","成交股數","成交金額(元)","成交筆數","最後買價","最後買量(張數)","最後賣價","最後賣量(張數)","發行股數",...]`

| 欄位 | 目前 index | 實際欄位 | 正確 index |
|------|-----------|---------|-----------|
| open | `[5]` | 最高 | `[4]` |
| high | `[6]` | 最低 | `[5]` |
| low | `[7]` | 均價 | `[6]` |
| close | `[8]` | 成交股數 | `[2]` |
| change | `[4]` | 開盤 | `[3]` |
| volume | `[10]` | 成交筆數 | `[8]` |
| turnover | `[11]` | 最後買價 | `[9]` |
| trade_count | `[12]` | 最後買量 | `[10]` |
| issued_shares | `[13]` | 最後賣價 | `[15]` |

## Goals / Non-Goals

**Goals:**
- TPEX 資料的開高低收、成交量、成交金額、成交筆數欄位正確對映
- 新增單元測試以實際 API 樣本驗證解析正確性
- 邊界檢查放寬為最小必要欄位數

**Non-Goals:**
- 不變更 TWSE parser
- 不變更資料庫 schema
- 不變更 pipeline 流程

## Decisions

### 1. 直接修正 index 而非重構為 named fields

**決策**：保留現有 array-index 存取方式，只修正 index 值。理由：
- TPEX API 使用 array-of-arrays 格式，沒有 named fields 可用
- 改用 named fields 需要改動 `ParsedRecord` 結構，超出本 change 範圍
- 修正 index 成本最低、風險最小

### 2. 邊界檢查從 `12` 改為 `11`

**決策**：`fields.len() < 11`。因為程式碼直接索引到 index 10（成交筆數），而 index 15（發行股數）已用 `.get(15)` 安全存取。11 個欄位即可涵蓋所有直接索引欄位（代號、名稱、收盤、漲跌、開盤、最高、最低、均價、成交股數、成交金額、成交筆數）。

### 3. 新增測試而非原地修改既有測試

**決策**：在 `tpex.rs` 的 `#[cfg(test)]` 區塊新增測試，使用實際 API 回應樣本。不修改既有 TWSE 測試。

## Risks / Trade-offs

- **[風險] 既有 TPEX 資料已錯誤**：修正後需手動清除並重新抓取。企業使用者需安排 migration 時間。
- **[風險] TPEX API 欄位順序異動**：台灣證交所或櫃買中心可能變更 API 格式。→ 已新增測試，API 格式變動時測試會失敗，可及早發現。
- **[取捨] 不處理 `last_bid`/`last_ask` 欄位**：TPEX API 有這些資料（index 11-14），但 `ParsedRecord` 的對應欄位用於 TWSE 的 CSV 格式。TPEX 的買賣價格式不同（含張數單位），需求不明確前保留為 `None`。
