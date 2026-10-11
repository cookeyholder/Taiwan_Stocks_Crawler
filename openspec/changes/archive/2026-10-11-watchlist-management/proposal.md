## Why

目前系統缺少自選股（Watchlist）管理機制。使用者在瀏覽分析個股或透過選股篩選出潛力標的後，無法將感興趣的標的加入個人自選名單，每次重新檢視都需要重新搜尋代號，嚴重影響看盤與追蹤效率。

## What Changes

- **資料庫 Schema 擴充（Migration V6）**：在 SQLite 新增 `watchlists` 資料表，支援儲存標的代號（`symbol_clean`）、自訂分組名稱（`group_name`，預設為「預設自選」）以及建立時間。
- **核心資料倉儲方法**：在 `stock-core::MarketRepository` 實作自選股的加入（`add_to_watchlist`）、移除（`remove_from_watchlist`）、查詢名單（`get_watchlist`）、群組清單（`get_watchlist_groups`）以及狀態判斷（`is_in_watchlist`）。
- **應用狀態協調**：在 `stock-gpui::CoreState` 實作非同步自選股狀態快取與一鍵切換（`toggle_watchlist`）。
- **圖形介面整合**：
  - 在個股分析畫面（`AnalysisView`）行情概況列提供一鍵收藏按鈕（`★ 已收藏` / `☆ 加入自選`）。
  - 在左側證券清單工具列新增清單切換籤（`全部` / `★ 自選股`），方便一鍵切換追蹤標的。

## Capabilities

### New Capabilities

- `watchlist-management`: 提供標的收藏、分組管理、狀態切換與自選股清單過濾能力。

## Impact

- **核心資料庫**：
  - `stock-core/src/db/migrations.rs`：新增 V6 資料庫遷移。
  - `stock-core/src/db/mod.rs`：新增 `MarketRepository` 自選股相關查詢與修改方法。
- **桌面應用**：
  - `stock-gpui/src/state.rs`：新增自選股狀態管理與協調方法。
  - `stock-gpui/src/views/analysis.rs`：在證券清單面板與行情列整合自選股切換與過濾。
