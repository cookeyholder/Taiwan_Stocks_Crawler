## 1. 後端：Pipeline::screen() 支援日期參數

- [x] 1.1 修改 `Pipeline::screen()` 簽名：新增 `date: Option<&str>` 參數
- [x] 1.2 實作邏輯：`date` 為 `Some` 時直接使用，`None` 時維持 `get_latest_trade_date()`
- [x] 1.3 更新 `screen_bollinger` Tauri command：新增 `date: Option<String>` 參數並傳遞

## 2. 前端：日期選擇器 UI

- [x] 2.1 `screener.ts` store 新增 `date` reactive state，類型 `string`；新增 `dateLoading: boolean` 標記初始化狀態
- [x] 2.2 `ScreenerView.vue` 新增日期輸入欄位（`<input type="date">`），預設填入最新交易日
- [x] 2.3 元件掛載時從 `get_latest_trade_date` 取得日期並設為預設值；期間 `dateLoading = true`
- [x] 2.4 `ScreenerView.vue` 在 `dateLoading` 時禁用篩選按鈕，顯示「載入中...」
- [x] 2.5 invoke `screen_bollinger` 時傳入 `date` 參數
- [x] 2.6 篩選結果為空且 `date` 有值時，顯示提示訊息「該日期無交易資料」

## 3. CLI：--date 選項

- [x] 3.1 `Screen` subcommand 新增 `--date` 選項
- [x] 3.2 `Pipeline::screen()` 呼叫時傳入日期參數

## 4. 驗證

- [x] 4.1 `cargo build` 通過
- [x] 4.2 `cargo test --workspace` 通過（37 passed）
- [x] 4.3 手動測試：日期選擇器預設顯示最新交易日
- [x] 4.4 手動測試：切換日期後篩選結果正確
- [x] 4.5 手動測試：選取無交易資料日期（如週末），顯示「該日期無交易資料」
- [x] 4.6 手動測試：快速點擊篩選（日期初始化前），按鈕應為禁用狀態
