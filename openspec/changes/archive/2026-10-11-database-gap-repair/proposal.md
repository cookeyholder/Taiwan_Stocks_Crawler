## Why

在台股市場歷史資料的蒐集與管理過程中，網路斷線、排程中斷或部分爬取失敗常會造成資料庫內出現「缺漏交易日（Gap Dates）」。目前系統僅能全區間重新下載，無法自動精確偵測資料庫在哪些工作日缺少資料，既浪費網路頻寬與時間，也難以快速維護歷史行情的連續性與完整性。

## What Changes

- **核心倉儲與診斷介面 (`stock-core`)**：
  - 在 `MarketRepository` 與 `Pipeline` 提供缺漏交易日診斷方法（`find_missing_dates` / `diagnose_date_gaps`），自動排除週末（六日），比對現存交易日，精確找出缺少資料的平日工作日。
  - 定義 `GapReport` 診斷報告結構體（涵蓋查詢區間、現存日數、平日總數及缺漏日期清單）。
- **桌面應用狀態與修補排程 (`stock-gpui::state`)**：
  - 在 `CoreState` 實作非同步診斷 `diagnose_gaps` 與針對性修補方法 `repair_gaps`，僅對缺漏日期進行定向下載入庫，並在修補完成後自動同步維度表。
- **下載管理畫面整合 (`stock-gpui::views::download`)**：
  - 在下載管理視圖新增「診斷缺漏交易日」與「一鍵修補缺漏」操作按鈕。
  - 診斷後呈現視覺化缺漏日數與日期清單。
  - 修補執行中提供即時進度比例、當前修補日期與取消機制。
- **CLI 命令列支援 (`stock-cli`)**：
  - 在 CLI 增加 `repair-gaps` 與 `diagnose-gaps` 子命令，支援批次維護與自動化排程。

## Capabilities

### New Capabilities

- `database-gap-repair`: 提供資料庫交易日缺漏診斷、缺漏日清單檢視、針對性批次修補與維度表自動同步能力。

## Impact

- **核心模組**：
  - `stock-core/src/db/mod.rs`：新增 `find_missing_dates` 方法。
  - `stock-core/src/pipeline/mod.rs`：新增診斷與修補相關型別與方法。
- **桌面應用**：
  - `stock-gpui/src/state.rs`：新增非同步診斷與修補方法。
  - `stock-gpui/src/views/download.rs`：整合診斷結果展示與一鍵修補互動按鈕。
- **命令列**：
  - `stock-cli/src/main.rs`：擴充子命令支援。
