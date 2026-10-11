## Context

目前系統架構以 [`stock-core`](file:///home/cookey/Projects/Taiwan_Stocks_Crawler/stock-core/src/lib.rs) 作為獨立核心函式庫，向上提供給 CLI（`stock-cli`）與桌面端（`stock-tauri`）。現有桌面端採用 Tauri v2 + Vue 3 架構，所有資料傳遞皆經由 `tauri::command` IPC 透過 JSON 序列化傳送。

本設計旨在將桌面端重構為純 Rust 應用程式 `stock-gpui`，基於 Longbridge 開源之 **GPUI-Kit** 與 **GPUI**，徹底消除前端 Webview 進程與 IPC 傳輸開銷。（背景動機詳見 `proposal.md` - Why）

## Goals / Non-Goals

**Goals:**
- 在 Cargo Workspace 中新增獨立 crate `stock-gpui`，完全使用 Rust 實作桌面 GUI。
- 引入 `gpui-kit` 作為統一 UI 與元件基礎庫（包含 `Button`、`DataTable`、`Chart`、`Form`、`Progress`、`Tabs`）。
- 實作高效能日 K 線與布林通道主圖，並以時間軸聯動架構整合成交量、RSI 與 MACD 副圖。
- 達成與現有 Tauri 桌面端 100% 的功能對等（個股分析、選股、回測、下載管理）。
- 保留 `stock-core` 原始結構與測試，不產生非必要的破壞性更動。

**Non-Goals:**
- 不修改 `stock-core` 既有之資料庫 Schema、指標演算法與回測邏輯。
- 不支援行動端平臺（專注於桌面 macOS、Windows 與 Linux）。
- 不在本次遷移中引入非既有功能或未經請求的全新技術指標。

## Decisions

### 1. 採用 `gpui-kit` 作為 Umbrella Crate，而非裸接裸 GPUI
- **決策內容**：相依於 `gpui-kit`（v0.7+），直接使用其封裝的 75+ 生產級元件與圖表庫。
- **背後考量**：GPUI 核心為 Pre-1.0 且緊隨 Zed 頻繁調整 API。`gpui-kit` 由長橋證券維護，已針對證券終端鎖定相容版本並實作了 `Candlestick` K 線圖與虛擬滾動 `DataTable`，可節省 70% 以上之手刻 UI 與底層 Canvas 頂點計算時間。
- **評估過之替代方案**：
  - *裸接 GPUI*：無現成表單、無表格、無 K 線圖，需自行處理所有座標轉換與佈局，工程代價極高。
  - *Iced / Slint*：雖為純 Rust GUI，但缺乏專為金融證券場景最佳化之高效能 K 線與深度圖表生態。

### 2. 雙軌漸進式遷移策略（Staged Migration Pipeline）
- **決策內容**：在 `stock-gpui` 達成全功能覆蓋並通過平臺驗證前，保留 `stock-tauri`；待驗證完備後才進行 Breaking 移除。
- **背後考量**：確保開發過程中既有桌面功能持續可用，並降低平臺圖形相容性風險。
- **評估過之替代方案**：
  - *一次性砍除重寫（Big Bang Rewrite）*：風險過高，若在圖表客製化或輸入法遇到邊界問題將無後退備案。

### 3. 多窗格副圖同步聯動架構（Multi-Pane Synchronization）
- **決策內容**：個股分析畫面採用主副圖分離但共用狀態之設計：
  - 上層 View 維護共用之 `TimeWindow`（起訖索引與縮放比例）與 `HoverState`（游標索引與 OHLCV/指標數值）。
  - 主圖（K線 + 布林 + 均線）、副圖（成交量）、副圖（RSI）、副圖（MACD）各自以垂直排列的 `Chart` 實例渲染。
  - 透過 GPUI 的事件監聽器（Listener），主圖之拖曳平移與滾輪縮放事件會同步廣播更新共用狀態，驅動所有副圖重新繪製對應視窗區間。
- **背後考量**：避免在單一巨大圖表中耦合過多不同量綱的 Y 軸，同時確保毫秒級的 120 FPS 聯動體驗。

### 4. 狀態管理與非同步任務協調
- **決策內容**：使用 GPUI 的 `Model<T>` 與 `cx.spawn()` 整合 Tokio 非同步任務。
- **背後考量**：下載管理與回測為非同步長時間任務，透過 `Model` 與取消旗標（`AtomicBool`），可維持 UI 介面流暢並即時接收下載進度推播。

## Risks / Trade-offs

- **[Risk] Linux / Windows 上繁體中文輸入法（IME）與字型支援**
  - *Mitigation*：在第一階段 PoC 立即在 Ubuntu (Wayland/X11) 與 Windows 進行注音/倉頡輸入法測試，並在 GPUI 主題設定中配置泛用 CJK 回退字型（Noto Sans CJK TC / Microsoft JhengHei）。
- **[Risk] `gpui-kit` 之 Chart 指標線自訂深度限制**
  - *Mitigation*：`gpui-kit` 支援自訂繪圖擴充，若預設 API 無法完美繪製布林通道半透明陰影填色，可透過其 Canvas API 疊加自訂視覺層。
- **[Risk] Rust 完整編譯時間（Build Time）較長**
  - *Mitigation*：保持 `stock-gpui` 模組劃分明確，開發階段推薦啟用平行連結器（`mold` 或 `lld`），避免影響日常除錯效率。

## Migration Plan

1. **第 1 階段：PoC 與環境驗證**
   - 新增 `stock-gpui` crate 並導入 `gpui-kit`。
   - 驗證基礎視窗建立、SQLite 連線與 `DataTable` 渲染。
   - 驗證 `Chart` 繪製 K 線與布林通道。
2. **第 2 階段：核心功能完整實作**
   - 實作四大檢視畫面：個股分析、布林選股、回測系統、下載管理。
   - 補齊深色/淺色主題與繁體中文介面文字。
3. **第 3 階段：平臺打包與整合測試**
   - 撰寫自動化整合測試（利用 `gpui-kit` headless window 測試能力）。
   - 設定 Linux 與 Windows 平臺打包腳本。
4. **第 4 階段：淘汰舊殼層**
   - 移除 `stock-tauri` 與根目錄 Node.js 檔案。
   - 更新專案說明文件與使用指南。
