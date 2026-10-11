## Why

PR #38–#42 的整合審查發現數個會誤導使用者或使功能無法實際使用的問題，包括修補失敗被當成成功、三大法人資料沒有進入產品抓取流程，以及圖表與策略邊界行為不符規格。這次集中修正並補上回歸測試，確保行情、分析與自選操作能正確反映資料狀態。

## What Changes

- 修正缺漏日期修補流程，明確回報來源錯誤並保留未完成的診斷資料；週末不計入現存平日數。
- 將 TWSE T86 資料擷取與資料庫寫入接入有 TWSE 行情的日期匯入流程，法人來源失敗不阻斷行情匯入。
- 修正十字游標在同一根 K 棒內移動時的重繪，以及離開主圖時的 HUD 狀態清除。
- 序列化自選股狀態切換，並在沒有收藏資料時提供「預設自選」群組。
- 將新高條件改為嚴格突破，並以小數張呈現法人買賣超。
- 更新相關規格與審查報告，補上回歸測試。

## Capabilities

### New Capabilities

無。

### Modified Capabilities

- `database-gap-repair`：明確區分修補失敗與休市，修正平日統計。
- `institutional-investors`：將 T86 擷取納入行情日期匯入，法人來源錯誤不得中斷行情匯入，張數保留小數。
- `gpui-stock-charts`：十字游標隨游標位置重繪，離開主圖後回復最新資料。
- `watchlist-management`：自選切換須序列化，空清單仍提供預設群組。
- `composite-screener`：僅將高於回溯高點的價格視為突破。

## Impact

- `stock-core/src/pipeline/mod.rs`、`stock-core/src/db/mod.rs`、`stock-core/src/backtest/new_high.rs`、`stock-gpui/src/state.rs`、`stock-gpui/src/views/analysis.rs`、`stock-gpui/src/views/download.rs`。
- 相關 Rust 單元測試及 GPUI 無頭測試；無資料庫結構遷移。
