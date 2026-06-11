## Context

現有 `gui.py` 提供 Tkinter 雙分頁介面（單股分析 + 布林通道選股器），但啟動官方資料管線（TWSE/TPEX 收盤行情下載）只能透過 CLI `twse-crawler` 執行。GUI 使用者無法一鍵觸發完整市場資料下載，必須切換到終端機手動操作。

`pipeline.py` 與 `orchestrator.py` 已提供完整的 `fetch_and_ingest_day()` 與 `run_date_range_ingestion()` 函式，可複用。

## Goals / Non-Goals

**Goals:**
- 在 GUI 中加入「一鍵下載所有上市、上櫃股票」功能
- 支援三種下載模式：下載今日、下載指定日期、下載日期區間
- 背景執行緒執行下載，不阻塞 UI
- 透過佇列回傳進度與結果，更新狀態列/進度條/日誌區

**Non-Goals:**
- 不修改 CLI 或 Pipeline 核心邏輯
- 不修改資料庫 Schema
- 不引入新的外部依賴

## Decisions

- **複用現有 Pipeline**：直接呼叫 `run_date_range_ingestion()` + `make_fetcher()`，不重複實作爬蟲邏輯。避免維護兩套下載路徑。
- **新增第三個分頁（下載管理）**：在 Notebook 中新增 Tab 3，放置日期選擇器（起始/結束日期輸入框）、下載按鈕、進度日誌區。不塞入既有分頁以保持介面清晰。
- **背景執行緒 + Queue 進度回饋**：沿用現有的 `task_queue` + `after()` 輪詢模式。背景執行緒將下載進度（處理到哪一天、成功/失敗筆數）透過佇列傳回主執行緒。
- **進度粒度**：以「天」為單位回報（如「正在處理 2026-06-09...」），而非逐筆股票。因 Pipeline 設計上單一天次即下載所有股票，逐筆回報會增加不必要的複雜度。

## Risks / Trade-offs

- [執行緒安全] 背景執行緒使用獨立的 `requests.Session` 與 `sqlite3.Connection`，避免與主執行緒共用連線造成競爭。
- [長時間操作] 回溯大量日期（如 1 年）可能耗時數分鐘。需顯示明確進度，並維持按鈕禁用防止重複點擊。
- [錯誤處理] 單一日期失敗不中斷整體流程，繼續處理剩餘日期。
