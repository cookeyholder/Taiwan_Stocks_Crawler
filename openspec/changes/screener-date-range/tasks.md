## 1. 資料模型

- [ ] 1.1 在 `stock-core/src/models.rs` 新增 `RangeMeta` struct（start_date, end_date, trading_days, cached_days, calculated_days），derive `Serialize, Deserialize`
- [ ] 1.2 新增 `RangeScreenResult` struct（results: Vec<ScreenResult>, meta: RangeMeta），derive `Serialize, Deserialize`
- [ ] 1.3 在 `RangeMeta` 中加入 `failed_days: Vec<String>` 欄位

## 2. Tauri Command：screen_bollinger_range

- [ ] 2.1 在 `stock-tauri/src-tauri/src/lib.rs` 定義 `RangeProgress` 結構（current, total, date, cached），derive `Clone, Serialize`
- [ ] 2.2 實作 `screen_bollinger_range` command：
      - 接收 `start_date`, `end_date`, `max_bandwidth`, `min_days`, `instrument_types`, `min_volume`
      - 驗證日期格式（`chrono::NaiveDate::parse_from_str`），不合則回傳錯誤
      - 驗證 `start_date <= end_date`，違反則回傳錯誤
      - 呼叫 `repo.get_trade_dates_in_range(&start_date, &end_date)` 取得交易日清單
      - 檢查交易日數量 ≦ 60，超過則回傳錯誤「日期區間過大，請縮小至 60 個交易日以內」
      - 迭代清單，逐日呼叫既有的 cache-aware 篩選邏輯（查 `get_cached_screening` → miss 則呼叫 `pipeline.screen()` → 存入 cache）
      - 每次迭代前檢查 `cancel_flag`，若為 true 則中斷並回傳當前結果
      - 若某日 `pipeline.screen()` 拋錯，跳過該日並記錄至 `failed_days`，繼續下一日
      - 每次迭代 emit `range-progress` event
      - 統計 cached_days、calculated_days、failed_days
      - 保留最後一天的結果（若全部失敗則回傳空結果）
      - 回傳 `RangeScreenResult`
- [ ] 2.3 移除舊的 `screen_bollinger_with_cache` command
- [ ] 2.4 在 `invoke_handler` 中註冊 `screen_bollinger_range`
- [ ] 2.5 新增 `cancel_screening` command，將 `cancel_flag` 設為 true
- [ ] 2.6 在 `AppState` 中新增 `cancel_flag: Arc<AtomicBool>`，每個 `screen_bollinger_range` 共用此 flag

## 3. 前端 Store 修改

- [ ] 3.1 將 `date: ref("")` 改為 `startDate: ref("")` 與 `endDate: ref("")`
- [ ] 3.2 新增 `meta: ref<RangeMeta | null>`、`progress: ref({current: 0, total: 0, date: "", cached: false})`
- [ ] 3.3 新增 `navigateDate(direction: -1 | 1)` 方法：
      - 加入 `navigating` lock flag 防止連點
      - `startDate += direction`、`endDate += direction`
      - 呼叫 `runRangeScreen()`
      - 完成後釋放 `navigating`
- [ ] 3.4 新增 `runRangeScreen()` 方法：
      - 在 invoke 前註冊 `listen("range-progress", ...)` 更新 `progress`
      - 呼叫 `invoke("screen_bollinger_range", {startDate, endDate, ...})`
      - 完成後更新 `results` 與 `meta`
      - 取消註冊 listener
- [ ] 3.5 更新 `return` 匯出新屬性與方法
- [ ] 3.6 新增 `cancelScreening()` 方法，呼叫 `invoke("cancel_screening")`

## 4. 前端 UI

- [ ] 4.1 將單一日期選擇器改為兩個：起始日期與結束日期
- [ ] 4.2 在起始日期左側加入 ◀ 按鈕、結束日期右側加入 ▶ 按鈕
- [ ] 4.3 推移按鈕 disabled 條件：`loading || dateLoading || clearing`
- [ ] 4.4 在篩選按鈕下方顯示進度文字（「處理中 3/8 天」）
- [ ] 4.5 篩選完成後顯示掃描統計（「已掃描 2026-07-01 → 2026-07-14，共 8 個交易日（快取 3 天，失敗 0 天）」）
- [ ] 4.6 篩選進行中時顯示「取消」按鈕，點擊後呼叫 `cancelScreening()`
- [ ] 4.7 取消後顯示「篩選已取消」訊息，保留部分統計

## 5. 格式化與最終驗證

- [ ] 5.1 `cargo fmt --all` 格式化 Rust 程式碼
- [ ] 5.2 `cargo check` 確認編譯
- [ ] 5.3 `cargo test -p stock-core` 確認測試通過
- [ ] 5.4 手動驗證日期區間篩選與推移功能正常
