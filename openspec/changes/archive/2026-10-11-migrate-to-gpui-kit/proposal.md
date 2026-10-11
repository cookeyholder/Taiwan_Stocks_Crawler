## Why

目前專案的桌面應用程式（`stock-tauri`）採用 Tauri v2 + Vue 3 + Vite + ECharts 架構。雖然具備豐富的 Web 生態，但存在以下限制：
1. **跨語言架構與 IPC 序列化開銷**：核心資料庫與計算皆由 Rust（`stock-core`）處理，但每次載入大量 K 線與指標資料時，皆須透過 JSON 跨程序序列化至 Webview，造成額外 CPU 與記憶體損耗。
2. **多套工具鏈維護成本**：專案同時依賴 Rust (Cargo) 與 Node.js (npm / Vite / TypeScript)，打包與 CI/CD 流程複雜。
3. **記憶體佔用與啟動延遲**：Webview 行程開銷較大（記憶體約 150MB～300MB）。

藉由評估並轉換至由 Longbridge（長橋證券）開源維護的 **GPUI-Kit**（基於 Zed 的 GPU 加速 UI 框架 GPUI），本專案可實現 100% 純 Rust 單一技術棧、零 IPC 記憶體直傳，並直接利用其專為金融交易終端打造的高效能 K 線圖表（`Chart`）與虛擬滾動表格（`DataTable`），達成極致效能與 120 FPS 的原生桌面體驗。

## What Changes

本變更採「分階段漸進式驗證與遷移（Staged PoC & Migration）」：

1. **第一階段：可行性概念驗證（PoC）**
   - 評估 GPUI / GPUI-Kit 在 Linux 與 Windows 平臺之相容性（渲染後端 Vulkan/Direct3D、中文輸入與字型）。
   - 在 Cargo Workspace 中新增獨立 crate `stock-gpui`。
   - 驗證 `gpui-kit` 之 `Chart` 對 K 線（Candlestick）、布林通道、均線與副圖指標（RSI / MACD）的繪製與聯動支援度。
   - 驗證 `DataTable` 在大批選股結果下的虛擬滾動與排序效能。

2. **第二階段：核心畫面移植**
   - 實作原生應用程式外框（視窗管理、分頁切換、暗色/明亮主題）。
   - 移植「個股分析」畫面：股票代號查詢、K 線與指標圖表、OHLCV 游標資訊窗。
   - 移植「布林通道選股」畫面：篩選參數設定、即時執行、結果排序表與點擊事件。
   - 移植「下載管理」與「回測系統」畫面：整合 Tokio 非同步任務與進度回報。

3. **第三階段：桌面端整合與發行測試**
   - 評估完成並達到功能對等後，將桌面預設入口由 `stock-tauri` 移轉至 `stock-gpui`。
   - 配置跨平台桌面應用程式打包流程（`cargo-bundle` 或平臺原生打包工具）。
   - **BREAKING**：移除 `stock-tauri` 及根目錄下的 Node.js / npm 前端工具鏈與相依套件。

## Capabilities

### New Capabilities
- `gpui-desktop-shell`: 以 GPUI 與 GPUI-Kit 實作的原生 GPU 加速桌面應用程式殼層，提供視窗管理、多分頁導覽、主題配置與 `stock-core` 狀態整合。
- `gpui-stock-charts`: 基於 `gpui-kit` 之圖表元件，提供高效能日 K 線、布林通道軌道、成交量及多指標副圖聯動展示與十字游標互動。

### Modified Capabilities
- `tauri-desktop-shell`: 將原本依賴 Tauri Webview 之原生視窗與生命週期需求，擴充/轉移由原生 GPUI 殼層承接。

## Impact

- **程式碼架構**：
  - 新增 `stock-gpui` crate，相依於 `stock-core`、`gpui`、`gpui-kit`、`tokio`。
  - 既有 `stock-core` 核心函式庫保持不變，向上直接由 Rust GUI 模組呼叫。
  - 最終將淘汰並移除 `stock-tauri/` 目錄與根目錄之前端建置設定檔。
- **建置與依賴**：
  - 專案根目錄全面回歸純 Cargo Workspace，移除 `package.json`、`package-lock.json` 與 Node.js 建置相依。
  - 桌面端需要平臺圖形相依庫支援（Linux 上之 Vulkan、Wayland / X11、Fontconfig；Windows 上之 Direct3D / DirectWrite）。
- **效能指標**：
  - 桌面程式記憶體佔用預期由 150MB~300MB 降至 30MB~60MB。
  - 冷啟動速度達毫秒級，圖表縮放與滾動維持 60/120 FPS。
