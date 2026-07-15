## Why

`stock-core/src/fetcher/tpex.rs` 中 TPEX JSON API 的欄位索引（field indices）直接複製自 TWSE CSV 格式，但 TPEX JSON API 的回應欄位順序完全不同。這導致 close_price 實際上存入了成交股數，所有價格資料（開高低收）全部錯位。TPEX 市場的分析功能完全不可用。

## What Changes

- **修正 `tpex.rs` 中的 field indices**：重新對應 TPEX JSON API 的實際欄位順序
- **調整 `fields.len()` 邊界檢查**：從 `12` 改為 `3`（至少需要代號、名稱、收盤）
- **新增 TPEX parser 的單元測試**：用實際 API 回應樣本驗證欄位對映正確性
- **需刪除既有 TPEX 資料**：因已存入錯誤資料，修正後需清除後重新抓取

## Capabilities

### New Capabilities

- `data-fetching`: 資料爬蟲功能需確保 TWSE 和 TPEX 的欄位解析各自正確

### Modified Capabilities

（無 — 此專案先前無 OpenSpec spec 定義）

## Impact

- `stock-core/src/fetcher/tpex.rs`：field indices 全面修正、邊界檢查放寬
- `stock-core/src/fetcher/tpex.rs`（測試區塊）：新增單元測試
- 既有 SQLite 資料庫中的 TPEX 資料需要清除後重新匯入
