## Why

使用者調整篩選條件（日期、證券類型、帶寬、天數、成交量）後按「開始篩選」，每次都會完整執行 BacktestEngine 重新計算。當想要比較不同條件組合的結果時，相同條件的結果沒有被保留，必須重複等待同一組運算。加入快取機制後，相同參數的篩選結果可以直接回傳，不需重算。

## What Changes

- 新增 SQLite 表 `screener_cache`，存放篩選條件與結果的對應
- 新增 Tauri command `screen_bollinger_with_cache`，取代原有的 `screen_bollinger`
- 新增 Tauri command `clear_screener_cache`
- 前端「布林通道選股」頁面新增「清除快取」按鈕
- 移除舊的 `screen_bollinger` command

## Capabilities

### New Capabilities
- `screener-cache-filter`: 篩選結果快取查詢與寫入、手動清除

### Modified Capabilities
（無）

## Impact

- **stock-core**: 新增 migration V5 `screener_cache` 表；`MarketRepository` 新增三個方法（get/set/clear）
- **stock-tauri (Rust)**: 修改 `screen_bollinger` 為 `screen_bollinger_with_cache`，新增 `clear_screener_cache` command
- **stock-tauri (Vue)**: `screener.ts` 改用新的 command；`ScreenerView.vue` 新增清除按鈕
