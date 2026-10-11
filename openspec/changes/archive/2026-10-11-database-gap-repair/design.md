# Design: 資料庫缺漏交易日自動診斷與修補

## 1. 概述

本設計為 `Taiwan_Stocks_Crawler` 提供資料庫歷史行情缺漏交易日之自動診斷與針對性定向修補功能。系統藉由比對指定期間之平日工作日與資料庫現存交易日，找出中斷或缺漏之日期，並提供定向爬取入庫流程與圖形介面/CLI 互動。

## 2. 架構設計與演算法

### 2.1 缺漏日比對演算法

1. 給定起始日期 `start_date` 與結束日期 `end_date`（格式 `YYYY-MM-DD`）。
2. 使用 `chrono::NaiveDate` 循序枚舉介於兩者之間的所有日期：
   - 若該日為週六（`Weekday::Sat`）或週日（`Weekday::Sun`），則直接略過。
   - 若為週一至週五，計入平日工作日清單 `all_workdays`。
3. 呼叫 `MarketRepository::get_trade_dates_in_range(start, end)` 取得資料庫現存交易日集合 `HashSet<String>`。
4. 比對 `all_workdays` 中未出現於現存集合者，即為「疑似缺漏工作日」`missing_dates`。
5. 產出 `GapReport`：
   ```rust
   #[derive(Clone, Debug, PartialEq, Eq)]
   pub struct GapReport {
       pub start_date: String,
       pub end_date: String,
       pub total_workdays: usize,
       pub recorded_dates: usize,
       pub missing_dates: Vec<String>,
   }
   ```

### 2.2 定向修補流程

1. 接收 `missing_dates: &[String]` 與進度回報回呼函數 `on_progress`。
2. 重設 `cancel_flag = false`。
3. 依序對每一筆缺漏日期呼叫 `Pipeline::fetch_date(&date)`：
   - 檢查 `cancel_flag`，若使用者中途取消則優雅中止。
   - 若交易所回傳 0 筆（例如國定假日休市），紀錄為已處理，不中斷整體批次。
   - 若成功入庫則累加寫入筆數。
   - 每次處理完畢觸發 `on_progress(done_count, total_count, current_date)`。
4. 所有日期處理完成後，呼叫 `sync_securities()` 更新證券維度表。

## 3. GPUI 桌面應用與視圖整合

### 3.1 `DownloadView` 狀態與視覺呈現

- **狀態管理**：
  - 新增 `pub gap_report: Option<GapReport>` 保存診斷結果。
- **操作介面**：
  - 新增 `Button::new("btn-diagnose-gaps")`：「🔍 診斷缺漏交易日」。
  - 新增 `Button::new("btn-repair-gaps")`：「🔧 一鍵修補缺漏 (N)」，當有缺漏時以警告色或主要色呈現，點擊觸發定向修補。
  - 當有診斷結果時，展示診斷摘要面板：
    - 平日總數、現存交易日數、缺漏天數。
    - 缺漏日期列表預覽（可捲動或逗號分隔呈現前數個缺漏日）。

## 4. 測試規劃

1. **核心倉儲與診斷單元測試 (`stock-core/tests/`)**：
   - 測試指定區間無缺漏之情況。
   - 測試包含週末與工作日缺漏之準確比對。
   - 測試空資料庫之全量平日診斷。
2. **狀態與修補測試 (`stock-gpui/tests/gap_repair_test.rs`)**：
   - 測試 `CoreState::diagnose_gaps` 產出正確 `GapReport`。
   - 測試 `CoreState::repair_gaps` 之排程、進度回呼與取消功能。
   - 測試無頭視窗下診斷按鈕之渲染與互動。
