## 1. 前端實作

- [x] 1.1 在 `DownloadView.vue` 將 `fetchToday` 改為 `fillToToday`：
      - `today = getTodayStr()` 取得臺灣今日日期
      - 以 try/catch 包圍 `invoke("get_latest_trade_date")` 取得最新日期
      - 若回傳 `null`，設定 `store.result = "資料庫尚無資料，請先手動指定日期下載"`
      - 若等於 `today`，設定 `store.result = "資料已是最新"`
      - 若 RPC 拋出例外 e，設定 `store.result = "查詢日期失敗：${e}"`
      - 否則計算 `nextDay = calcNextDay(latestDate)`，呼叫 `store.startDownload(nextDay, today)`
      - `await store.startDownload` 完成後呼叫 `await refreshDbInfo()` 更新資料庫概況卡
- [x] 1.2 新增 `getTodayStr()` 輔助函式（使用 `Intl.DateTimeFormat` 搭配 `Asia/Taipei` 時區，避免 UTC 跨日誤差）
- [x] 1.3 新增 `calcNextDay()` 輔助函式（手動解析年月日避免時區問題）
- [x] 1.4 將模板中的按鈕文字從「下載今日」改為「補至今日資料」
- [x] 1.5 按鈕加入 `:disabled="store.running"` 屬性

## 2. 格式化與驗證

- [ ] 2.1 確認無編譯/型別錯誤（`cargo check` + Vite 前端編譯）
- [ ] 2.2 手動驗證補齊功能正常
