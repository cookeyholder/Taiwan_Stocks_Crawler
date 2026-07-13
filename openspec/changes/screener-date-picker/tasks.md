## 1. 後端：Pipeline::screen() 支援日期參數

- [ ] 1.1 修改 `Pipeline::screen()` 簽名：新增 `date: Option<&str>` 參數
- [ ] 1.2 實作邏輯：`date` 為 `Some` 時直接使用，`None` 時維持 `get_latest_trade_date()`
- [ ] 1.3 更新 `screen_bollinger` Tauri command：新增 `date: Option<String>` 參數並傳遞

## 2. 前端：日期選擇器 UI

- [ ] 2.1 `screener.ts` store 新增 `date` reactive state，類型 `string`
- [ ] 2.2 `ScreenerView.vue` 新增日期輸入欄位（`<input type="date">`），預設填入最新交易日
- [ ] 2.3 元件掛載時從 `get_latest_trade_date` 取得日期並設為預設值
- [ ] 2.4 invoke `screen_bollinger` 時傳入 `date` 參數

## 3. CLI：--date 選項

- [ ] 3.1 `Screen` subcommand 新增 `--date` 選項
- [ ] 3.2 `Pipeline::screen()` 呼叫時傳入日期參數

## 4. 驗證

- [ ] 4.1 `cargo build` 通過
- [ ] 4.2 `cargo test --workspace` 通過
- [ ] 4.3 手動測試前端日期選擇功能
