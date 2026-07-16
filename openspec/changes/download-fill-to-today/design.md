## Context

下載管理頁面目前有兩個下載入口：
- **下載今日**（`fetchToday`）：直接呼叫 `invoke("fetch_today")`，僅下載今天一天的 TWSE + TPEX 資料
- **批次下載**（`startDownload`）：使用者手動輸入起始/結束日期，呼叫 `batch_download` 逐日下載

當資料庫已有數天前的資料時（例如最新日期為 07-10），要下載 07-11 到今天的資料，使用者必須：
1. 查詢資料庫概況得知最新日期
2. 手動輸入 `07-11` 到今天的日期到批次下載欄位
3. 按下「開始下載」

這流程繁瑣且容易打錯日期。

## Goals / Non-Goals

**Goals:**
- 單一按鈕自動補齊缺失的交易日資料
- 按鈕從「下載今日」改為「補至今日資料」
- 全前端實作，不新增後端 command

**Non-Goals:**
- 不修改 `batch_download` 或 `fetch_date` 的既有行為
- 不下載今日之後的未來資料
- 不處理 DB 完全空白的情況（由使用者先手動指定起始日期）

## Decisions

### 1. 全前端串接，不新增後端 command

利用既有的三個後端 command 即可完成：

```
fillToToday()
    │
    ├─ today = getTodayStr()
    │
    ├─ invoke("get_latest_trade_date")
    │    若回傳 None → 顯示「資料庫尚無資料，請先手動下載」
    │    若 RPC 拋異常 → 顯示「查詢日期失敗：${e}」
    │
    ├─ 若 latestDate == today → 顯示「資料已是最新」
    │
    └─ 否則：
          nextDay = calcNextDay(latestDate)
          呼叫 store.startDownload(nextDay, today)
          await refreshDbInfo()
          batch_download + progress 事件已完整支援
```

不新增後端 command 的理由：
- 既有 `batch_download` 已完整實作進度回報（emit `download-progress`）
- 前端 store 的 `startDownload` 已封裝 listen/unlisten 邏輯
- 兩次 RPC（查日期 + 下載）的成本極低，且第一次查日期是瞬間回傳

### 2. 日期格式與時區

使用兩個前端輔助函式處理日期，全在用戶端執行（不新增後端 command）。

#### 2a. 取得「今日」日期（臺灣時區）

```typescript
function getTodayStr(): string {
  const formatter = new Intl.DateTimeFormat('zh-TW', {
    timeZone: 'Asia/Taipei',
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  });
  return formatter.format(new Date()).replace(/\//g, '-');
}
```

使用 `Intl.DateTimeFormat` 搭配 `Asia/Taipei` 時區，避免 `new Date().toISOString()` 回傳 UTC 日期導致跨日誤差（臺灣 UTC+8，UTC 日期可能比臺灣慢一天）。

#### 2b. 日期加一日

與前端其他日期處理一致，手動解析年月日以避免 `new Date(isoDate)` 的 UTC/時區偏移問題：

```typescript
function calcNextDay(isoDate: string): string {
  const [y, m, d] = isoDate.split('-').map(Number);
  const date = new Date(y, m - 1, d);
  date.setDate(date.getDate() + 1);
  const yy = date.getFullYear();
  const mm = String(date.getMonth() + 1).padStart(2, '0');
  const dd = String(date.getDate()).padStart(2, '0');
  return `${yy}-${mm}-${dd}`;
}
```

此作法避免 `new Date(isoDate)` 的 UTC/時區偏移問題（與 screener-date-range 的 navigateDate 採用相同模式）。

### 3. progress 與 result 共用既有 UI

`fillToToday` 啟用後直接呼叫 `store.startDownload()`，其內部的 `progress` 和 `result` 已由 download store 管理，不需新增 UI 元件。

## Risks / Trade-offs

- **[連續點擊] → 依賴 store.running 防護**：startDownload 已使用 `running` flag，`fillToToday` 在呼叫前再次檢查
- **[日期區間過大] → 可接受**：若使用者很久沒更新，批次下載可能需下載上百天，但 batch_download 已有 progress 回報
- **[RPC 異常] → 需 try/catch**：`invoke("get_latest_trade_date")` 可能因網路或後端錯誤拋出例外，`fillToToday` 需包在 try/catch 中並將錯誤訊息設入 `store.result`
