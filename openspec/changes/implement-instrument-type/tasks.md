## 1. TPEX parser：放寬表格限制

- [ ] 1.1 修改 `tpex.rs` 的 `parse_tpex_json`，不再跳過非「上櫃股票」表格
- [ ] 1.2 傳遞 table title 資訊到 `normalize_records`（或回傳分類資訊）
- [ ] 1.3 更新 TPEX 測試資料以涵蓋多個 table title 情境

## 2. normalize_records：實作分類規則

- [ ] 2.1 在 `normalize_records` 新增 `instrument_type` 推斷函數
- [ ] 2.2 實作 TPEX 分類：table title 包含「上櫃股票」→ `stock`，包含「上櫃ETF」→ `etf`
- [ ] 2.3 實作 TWSE 分類：依 symbol prefix 規則判斷
- [ ] 2.4 更新 `normalize_records` 的 `instrument_type` 由硬編碼改為分類函數

## 3. 前端：類型篩選 UI

- [ ] 3.1 `ScreenerView.vue` 新增類型多選控制項（checkbox 群組）
- [ ] 3.2 `screener.ts` store 的 `instrumentTypes` state 整合 UI 控制
- [ ] 3.3 invoke `screen_bollinger` 時傳入選取的類型

## 4. 驗證

- [ ] 4.1 `cargo build` 通過
- [ ] 4.2 `cargo test --workspace` 通過（含 TPEX 多表格測試）
- [ ] 4.3 確認新擷取資料的 `instrument_type` 值正確寫入 DB
