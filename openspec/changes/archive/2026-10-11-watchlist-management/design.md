# Design: 自選股清單（收藏、分組、快速切換）

## 1. 概述

本設計為 `Taiwan_Stocks_Crawler` 提供自選股（Watchlist）之持久化管理與桌面端流暢互動。包含 SQLite 資料庫遷移 V6、`stock-core` 資料倉儲 CRUD 介面、`stock-gpui` 狀態協調快取，以及 `AnalysisView` 介面的收藏按鈕與清單分類過濾。

## 2. 架構與資料庫設計

### 2.1 SQLite Schema (Migration V6)

在 `stock-core/src/db/migrations.rs` 引入第 6 版遷移腳本：

```sql
CREATE TABLE IF NOT EXISTS watchlists (
    symbol_clean TEXT NOT NULL,
    group_name TEXT NOT NULL DEFAULT '預設自選',
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    PRIMARY KEY (symbol_clean, group_name)
);

CREATE INDEX IF NOT EXISTS idx_watchlists_group ON watchlists(group_name);
```

- 主鍵為 `(symbol_clean, group_name)`，支援同一個股票歸屬於多個自選群組（預設群組為「預設自選」）。
- 自動記錄收藏時間 `created_at`。

### 2.2 `stock-core` 倉儲方法 (`MarketRepository`)

在 `stock-core/src/db/mod.rs` 擴充以下介面：

```rust
impl MarketRepository {
    /// 加入自選股（冪等寫入）
    pub fn add_to_watchlist(&self, symbol_clean: &str, group_name: &str) -> Result<()>;

    /// 移出自選股（若 group_name 為 None 則移除所有群組的該股票）
    pub fn remove_from_watchlist(&self, symbol_clean: &str, group_name: Option<&str>) -> Result<()>;

    /// 查詢自選股清單（回傳 symbol_clean 列表）
    pub fn get_watchlist(&self, group_name: Option<&str>) -> Result<Vec<String>>;

    /// 取得所有已建立之自選群組名稱
    pub fn get_watchlist_groups(&self) -> Result<Vec<String>>;

    /// 判斷股票是否已加入自選股
    pub fn is_in_watchlist(&self, symbol_clean: &str, group_name: Option<&str>) -> Result<bool>;
}
```

## 3. GPUI 狀態與視圖設計

### 3.1 `stock-gpui/src/state.rs` (`CoreState`)

- **快取管理**：維護記憶體中的自選代號快取集合 `watchlist_symbols: Arc<RwLock<HashSet<String>>>`。
- **生命週期**：在初始化及資料庫連線就緒時非同步載入目前所有自選標的。
- **介面方法**：
  - `pub fn is_favorited(&self, symbol: &str) -> bool`：快速同步讀取快取判斷是否已收藏。
  - `pub fn toggle_watchlist(&self, symbol: &str, cx: &mut App)`：一鍵切換收藏狀態，即時更新快取並於背景非同步寫入或移出資料庫。
  - `pub fn refresh_watchlist(&self, cx: &mut App)`：重新從資料庫載入最新自選股名單。

### 3.2 `stock-gpui/src/views/analysis.rs` (`AnalysisView`)

- **清單分類篩選籤**：
  - 在左側證券搜尋列上方新增頁籤按鈕：`[ 全部 (N) ]` / `[ ★ 自選股 (M) ]`。
  - 當切換為「自選股」時，左側清單僅過濾顯示 `watchlist_symbols` 包含的標的。
- **行情概況收藏按鈕**：
  - 在右側分析主區頂部之個股代號與名稱右側，新增收藏按鈕。
  - 若已收藏顯示 `★ 已收藏`（金色/黃色高亮），點擊觸發移出；若未收藏顯示 `☆ 加入自選`，點擊觸發加入。
- **證券清單列星號指示**：
  - 左側清單每列在代號前呈現小星號，方便辨識是否為自選股。

## 4. 測試規劃

1. **核心資料庫單元測試 (`stock-core/tests/`)**：
   - 驗證 Migration V6 正常執行。
   - 驗證 `add_to_watchlist` 冪等性（重複加入不報錯）。
   - 驗證 `remove_from_watchlist` 正常移除與不存在時之容錯。
   - 驗證 `get_watchlist` 群組過濾與全部查詢。
   - 驗證 `is_in_watchlist` 正確性。
2. **狀態與 UI 測試 (`stock-gpui/tests/`)**：
   - 驗證 `CoreState` 自選股快取切換與更新邏輯。
   - 驗證清單過濾條件切換。
