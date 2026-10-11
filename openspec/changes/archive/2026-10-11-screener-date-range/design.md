## Context

布林通道選股目前只能篩選單一日期。使用者在 params bar 選擇日期後按下「開始篩選」，觸發 `screen_bollinger_with_cache` → 查 cache → miss 則執行 `BacktestEngine::run_date()`。快取機制（V5 `screener_cache`）已在上一期實作，每個日期+參數的結果獨立快取。

若要比較不同日期的篩選結果，使用者必須手動來回切換日期重新執行。對於已快取的日期，雖然不需重算，但仍需手動操作 UI。

本變更將單一日期選擇器改為起始/結束日期雙選擇器，後端自動迭代區間內所有交易日，並利用既有的 `screener_cache` 跳過已計算的天數。結果呈現方式為「顯示最後一天結果 + 掃描範圍統計」，不改變 `ScreenResult` 結構。

## Goals / Non-Goals

**Goals:**
- 使用者可指定起始日與結束日，一次掃描多個交易日
- 利用現有的 `screener_cache` 跳過已計算天數，不重複運算
- 即時進度回報（emit `range-progress` event）
- 最終顯示最後一天的篩選結果 + 掃描統計（交易日數、快取命中數、實際計算數）
- 移除單日的 `screen_bollinger_with_cache` command，統一由 `screen_bollinger_range` 處理

**Non-Goals:**
- 不改變 `ScreenResult` 結構（不加入 `trade_date` 欄位）
- 不支援多日期結果合併比較（只看最後一天）
- 不修改 `BacktestEngine` 本身
- 不修改快取邏輯

## Decisions

### 1. 後端迭代日期，前端一次呼叫

後端新增 `screen_bollinger_range` command，負責：
- 查詢 `get_trade_dates_in_range()` 取得區間內所有交易日（跳過休市日）
- 迭代每個交易日，逐日呼叫 `screen_bollinger_with_cache` 的內部邏輯（查 cache → miss 則計算 → 存入 cache）
- 每次迭代完 emit `range-progress` event
- 保留最後一天的 `Vec<ScreenResult>` 作為最終結果
- 回傳 `RangeScreenResult { results: Vec<ScreenResult>, meta: RangeMeta }`

前端只需一次 `invoke`，不需自己跑迴圈。

### 2. 從交易日清單迭代，而非逐日 try

使用 `get_trade_dates_in_range()` 直接從 DB 取得實際有交易的日期清單，避免對週末/假日執行無意義的 `get_effective_trade_date()` 查詢。

### 3. 進度回報沿用 `app.emit()` 模式

與 `batch_download` 相同的 Tauri event 模式：

```rust
#[derive(Clone, Serialize)]
struct RangeProgress {
    current: usize,
    total: usize,
    date: String,
    cached: bool,
}
```

前端在 `runRangeScreen()` 中使用 `listen("range-progress", ...)` 接收即時進度，結束後 `unlisten`。

### 4. 新結構體

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeMeta {
    pub start_date: String,
    pub end_date: String,
    pub trading_days: usize,
    pub cached_days: usize,
    pub calculated_days: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeScreenResult {
    pub results: Vec<ScreenResult>,
    pub meta: RangeMeta,
}
```

### 5. 前端參數列改動

```
現在                          改為
────────────────────────      ────────────────────────
日期: [picker]                 ◀  起始: [picker]  →  結束: [picker]  ▶
                               [開始篩選]  [清除快取] ③
                               已掃描 2026-07-01→07-14
                               共 8 交易日（快取 3 天）
```

Store 的 `date: ref("")` 改為 `startDate: ref("")`、`endDate: ref("")`。

### 6. 向前向後按鈕（方向 A）

在起始日期左側加入 ◀ 按鈕、結束日期右側加入 ▶ 按鈕。點擊時將整個區間向前/向後推移 1 天（區間長度維持不變），並**自動觸發篩選**。

```
◀  [2026-07-01]  →  [2026-07-14]  ▶
                     點 ▶ ↓
◀  [2026-07-02]  →  [2026-07-15]  ▶  ← 自動開始篩選
```

實作要點：
- 點 ◀ 時 `startDate -= 1 day`、`endDate -= 1 day`
- 點 ▶ 時 `startDate += 1 day`、`endDate += 1 day`
- 推移後立即呼叫 `runRangeScreen()`
- 推移不要求使用者先點「開始篩選」
- 推移時按鈕 disabled（loading 中不可連續按）

### 7. 日曆彈出視窗寬度

原生 `<input type="date">` 的彈出日曆由 WebKitGTK 控制，無法用 CSS 改變寬度。輸入框本身已設為 `width: 200px`，能完整顯示日期字串。若需要寬裕的日期選取體驗，可考慮之後改用自訂日期元件，但現階段以保留原生 + 箭頭操作為主。

### 8. 移除 `screen_bollinger_with_cache`

不再保留單日 command，前端統一使用 `screen_bollinger_range`（傳入 `start = end` 即等同單日查詢）。

### 9. 日期區間上限

為避免使用者選取過大區間（如一年 = ~250 交易日 × 1-2 秒 = 250-500 秒等待），後端需加入硬性上限 **60 個交易日**（約三個月）。若 `start_date` 與 `end_date` 之間的交易日超過上限，回傳錯誤「日期區間過大，請縮小至 60 個交易日以內」。

### 10. 日期驗證

後端 command 需執行以下驗證，參考 `delete_date_range` 的既有做法：

```rust
if chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").is_err()
    || chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").is_err()
{
    return Err("日期格式錯誤，請使用 YYYY-MM-DD 格式".into());
}
if start_date > end_date {
    return Err("起始日期不得晚於結束日期".into());
}
```

### 11. 錯誤處理策略

區間迭代期間，若某一天的篩選失敗（DB 查詢異常或 `pipeline.screen()` 拋錯），**跳過該天並繼續處理後續日期**，而非讓整個區間失敗。失敗資訊記錄於 `RangeMeta` 中：

```rust
pub struct RangeMeta {
    // ...原有欄位...
    pub failed_days: Vec<String>,  // 新增：篩選失敗的日期列表
}
```

前端在掃描統計中顯示「失敗 N 天」。

### 12. 取消機制

後端：

- 在 `AppState` 或 `Pipeline` 中新增 `cancel_flag: Arc<AtomicBool>`，`screen_bollinger_range` 每次迭代前檢查該 flag，若為 `true` 則提早回傳當前結果。
- 新增 `cancel_screening` command，將 flag 設為 `true`。

前端：

- 篩選進行中時，顯示「取消」按鈕。
- 點擊取消時呼叫 `cancel_screening`，後端在下一次迭代前停止。
- 取消後前端顯示「篩選已取消」，保留已完成的進度統計。

### 13. ◀ ▶ 按鈕防連點

即使按鈕在 `loading` 時 disabled，Vue 的非同步響應可能讓短時間內多次點擊通過檢查。需在 `navigateDate` 中加入 lock flag：

```typescript
const navigating = ref(false);
async function navigateDate(direction: -1 | 1) {
  if (navigating.value || loading.value) return;
  navigating.value = true;
  startDate.value = shiftDate(startDate.value, direction);
  endDate.value = shiftDate(endDate.value, direction);
  await runRangeScreen();
  navigating.value = false;
}
```

另可於按鈕事件加入 leading-edge debounce（`useDebounceFn` 或 `setTimeout` guard）。

## Risks / Trade-offs

- **[較長等待時間] → 進度條 + 區間上限 + 取消按鈕**：掃描一個月可能需執行 20+ 次篩選，每次約 1-2 秒。進度條讓使用者明確知道處理進度；60 日上限避免極端等待；取消按鈕讓使用者可隨時中止。
- **[快取加速效果依賴使用習慣] → 可接受**：若使用者每次都換不同日期區間，快取無法累積。但這是使用模式問題，非實作錯誤。
- **[移除舊 command 可能影響外部參考] → 無**：`screen_bollinger_with_cache` 僅在前端 `screener.ts` 中被呼叫，無其他消費者。
- **[逐日錯誤跳過可能遺漏] → 日誌補償**：若某天資料不完整導致篩選失敗，該天會被跳過且記錄於 `failed_days`。使用者可從掃描統計中得知並手動補查。此設計優於整批失敗。
- **[Vue 響應式延遲導致連點] → navigating flag 保護**：即使 `loading` 狀態的非同步更新可能有視窗期，`navigateDate` 內部的 `navigating` flag 確保同一時間只有一次推移操作。
