## 0. ParsedRecord：新增 instrument_type 欄位

- [ ] 0.1 在 `models.rs` 的 `ParsedRecord` 結構新增 `instrument_type: Option<String>` 欄位
- [ ] 0.2 更新 `ParsedRecord` 的 serde Serialize/Deserialize 處理（若需自訂）

## 1. TPEX parser：放寬表格限制 + 直接分類

- [ ] 1.1 修改 `tpex.rs` 的 `parse_tpex_json`，不再跳過非「上櫃股票」表格
- [ ] 1.2 解析 table title，根據映射規則設定每筆資料的 `ParsedRecord.instrument_type`
- [ ] 1.3 更新 TPEX 測試資料以涵蓋多個 table title 情境

## 2. normalize_records：symbol prefix 分類（TWSE）

- [ ] 2.1 新增 `classify_by_symbol(symbol: &str) -> &str` 函數，實作 TWSE symbol prefix 規則（0 開頭不限長度 → etf；03 開頭 5 位數 → warrant；02 開頭 5 位數 → etn；其餘 4 位數 → stock；以上皆非 → other）
- [ ] 2.2 `normalize_records` 中：若 `record.instrument_type` 為 `Some`，直接使用（TPEX）；否則呼叫 `classify_by_symbol`（TWSE）
- [ ] 2.3 移除原本硬編碼的 `"other"`

## 3. 前端：類型篩選 UI

- [ ] 3.1 `screener.ts` store 將 `instrumentTypes` 預設值由 `[]` 改為 `["stock"]`
- [ ] 3.2 `ScreenerView.vue` 新增類型多選控制項（checkbox 群組）
- [ ] 3.3 invoke `screen_bollinger` 時傳入選取的類型

## 4. 驗證

- [ ] 4.1 `cargo build` 通過
- [ ] 4.2 `cargo test --workspace` 通過（含 TPEX 多表格測試）
- [ ] 4.3 確認新擷取資料的 `instrument_type` 值正確寫入 DB
