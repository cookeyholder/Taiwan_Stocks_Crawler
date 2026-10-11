# Tasks: 資料庫缺漏交易日自動診斷與修補

## 1. 核心診斷層 (Core Diagnostics)

- [x] 1.1 定義 `GapReport` 結構體並在 `MarketRepository` 實作 `find_missing_dates` 與 `diagnose_gaps` <!-- id: 1.1 -->
- [x] 1.2 撰寫 `stock-core` 缺漏診斷單元測試並確保全數通過 <!-- id: 1.2 -->

## 2. 應用狀態層 (State & Repair Pipeline)

- [x] 2.1 在 `CoreState` 實作非同步 `diagnose_gaps` 介面 <!-- id: 2.1 -->
- [x] 2.2 在 `CoreState` 實作 `repair_gaps` 定向修補工作與進度回報通道 <!-- id: 2.2 -->
- [x] 2.3 撰寫狀態層診斷與修補整合測試 <!-- id: 2.3 -->

## 3. 圖形使用者介面層 (GPUI UI)

- [x] 3.1 在 `DownloadView` 加入 `gap_report` 狀態與「診斷缺漏交易日」按鈕 <!-- id: 3.1 -->
- [x] 3.2 在 `DownloadView` 加入「一鍵修補缺漏」操作按鈕與進度通知連動 <!-- id: 3.2 -->
- [x] 3.3 在 `DownloadView` 呈現診斷摘要與缺漏日期檢視面板 <!-- id: 3.3 -->
- [x] 3.4 撰寫無頭視窗 UI 互動測試 <!-- id: 3.4 -->

## 4. 驗證、封存與 PR (Verification & Archive)

- [x] 4.1 執行完整工作區測試與靜態檢查（`cargo test`, `cargo clippy`, `cargo fmt`） <!-- id: 4.1 -->
- [x] 4.2 封存 OpenSpec 變更至 `openspec/specs/` <!-- id: 4.2 -->
- [x] 4.3 建立 PR 並依規範合併至 `main` 分支 <!-- id: 4.3 -->
