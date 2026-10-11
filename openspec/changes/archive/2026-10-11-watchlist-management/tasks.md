# Tasks: 自選股清單（收藏、分組、快速切換）

## 1. 核心資料庫層 (Core Database)

- [x] 1.1 實作 SQLite 資料庫遷移 V6（`watchlists` 資料表與索引） <!-- id: 1.1 -->
- [x] 1.2 在 `MarketRepository` 實作自選股 CRUD 介面（`add_to_watchlist`, `remove_from_watchlist`, `get_watchlist`, `get_watchlist_groups`, `is_in_watchlist`） <!-- id: 1.2 -->
- [x] 1.3 撰寫 `stock-core` 自選股倉儲單元測試並確保全數通過 <!-- id: 1.3 -->

## 2. 桌面應用狀態層 (GPUI State)

- [x] 2.1 在 `CoreState` 實作自選代號快取集合與初始化載入 <!-- id: 2.1 -->
- [x] 2.2 在 `CoreState` 實作 `is_favorited` 查詢與 `toggle_watchlist` 一鍵切換方法 <!-- id: 2.2 -->
- [x] 2.3 撰寫狀態層自選股變更相關測試 <!-- id: 2.3 -->

## 3. 圖形使用者介面層 (GPUI UI)

- [x] 3.1 在 `AnalysisView` 左側股票清單頂部新增「全部」/「★ 自選股」篩選籤 <!-- id: 3.1 -->
- [x] 3.2 在 `AnalysisView` 股票代號行情列新增收藏狀態切換按鈕（`★ 已收藏` / `☆ 加入自選`） <!-- id: 3.2 -->
- [x] 3.3 在左側股票清單列表項目中顯示收藏星號識別 <!-- id: 3.3 -->
- [x] 3.4 驗證介面切換、即時篩選與響應流暢度 <!-- id: 3.4 -->

## 4. 驗證、封存與 PR (Verification & Archive)

- [x] 4.1 執行完整工作區測試與靜態檢查（`cargo test`, `cargo clippy`, `cargo fmt`） <!-- id: 4.1 -->
- [x] 4.2 封存 OpenSpec 變更至 `openspec/specs/` <!-- id: 4.2 -->
- [x] 4.3 建立 PR 並依規範合併至 `main` 分支 <!-- id: 4.3 -->
