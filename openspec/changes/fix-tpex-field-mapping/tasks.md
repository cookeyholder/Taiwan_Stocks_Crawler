## 1. 修正 TPEX 欄位對映

- [ ] 1.1 修正 `tpex.rs` 的 field indices：open=[4], high=[5], low=[6], close=[2], change=[3], volume=[8], turnover=[9], trade_count=[10], issued_shares=[15]
- [ ] 1.2 放寬 `fields.len()` 邊界檢查：從 `12` 改為 `3`
- [ ] 1.3 新增 TPEX JSON 解析的單元測試，用實際 API 回應樣本驗證欄位正確性

## 2. 驗證與清理

- [ ] 2.1 執行 `cargo test -p stock-core` 確認舊測試與新測試皆通過
- [ ] 2.2 執行 `cargo clippy -p stock-core` 確認無 lint 警告
- [ ] 2.3 提醒使用者清除既有 TPEX 資料庫後重新抓取
