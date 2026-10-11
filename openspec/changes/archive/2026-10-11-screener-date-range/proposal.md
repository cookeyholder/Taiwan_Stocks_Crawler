## Why

目前布林通道選股只能指定單一日期，無法一次掃描一段日期區間。如果需要比較多天的篩選結果，必須逐日手動調整日期、重新按下篩選，流程繁瑣且耗時。加入日期區間篩選後，使用者可以指定起始日與結束日，系統自動迭代每個交易日並使用既有的快取機制跳過已算過的天數。

## What Changes

- 布林通道選股頁面的日期選擇器從單一日改為起始日期 + 結束日期兩欄
- 新增 Tauri command `screen_bollinger_range`，接收起始/結束日期與篩選參數，後端迭代交易日並回傳結果與 meta
- 新增 `RangeScreenResult` 與 `RangeMeta` 結構體，封裝結果與掃描統計
- 進度回報沿用 `app.emit()` 模式，前端顯示即時進度
- 單日期篩選改為區間篩選，不再保留單獨的 `screen_bollinger_with_cache` command

## Capabilities

### New Capabilities
- `screener-date-range-filter`: 指定起始與結束日期，對區間內每個交易日執行布林通道篩選，利用快取跳過已計算的天數，回傳最後一天結果與掃描統計

### Modified Capabilities

（無）

## Impact

- **stock-core/src/models.rs**：新增 `RangeMeta`、`RangeScreenResult`
- **stock-tauri/src-tauri/src/lib.rs**：新增 `screen_bollinger_range` command，移除 `screen_bollinger_with_cache`
- **stock-tauri/src/stores/screener.ts**：`date` 改為 `startDate`/`endDate`，新增 `meta`、`progress`、`runRangeScreen()`
- **stock-tauri/src/views/ScreenerView.vue**：雙日期選擇器、進度條、meta 資訊列、取消按鈕

## 安全邊界

- **日期驗證**：後端強制檢查日期格式（YYYY-MM-DD）及 `start_date <= end_date`
- **區間上限**：限制最多 60 個交易日（約三個月），避免過大區間造成過長等待
- **參數化查詢**：所有使用者輸入皆經由 rusqlite `params!` 傳遞，無 SQL Injection 風險
- **無檔案操作**：本功能不涉及任何檔案讀寫，無 path traversal 風險

## 效能邊界

- **區間上限保護**：60 個交易日的硬上限，單次篩選最長等待約 60-120 秒
- **快取跳過加速**：已快取日期直接跳過計算，僅 emit 進度
- **進度事件頻率**：每次迭代都 emit，但前端接收頻率受 Tauri IPC 特性自然限制，不致阻塞
- **批次查詢非必要**：N 次個別 `get_cached_screening` 查詢皆為 primary key lookup，開銷可接受
