## Why

行情與分析頁面需要可更新的本機資料，但目前下載、補檔和券商日報表操作只存在於 CLI 或 GPUI。此階段讓使用者從網站啟動並監看既有抓取流程，同時避免長時間工作佔住 HTTP 請求或重複寫入 SQLite。

## What Changes

- 新增受保護的行情更新、區間補檔及券商日報表作業 API。
- 以作業 ID、狀態查詢、進度輪詢及取消操作管理長時間工作。
- 擴充第三階段共用工作管理器，納入資料寫入工作並明確序列化長時間工作。
- 將單日抓取結果改為可區分完整成功、部分失敗、有效無資料及來源失敗的型別；先驗證上游回應結構，不解析人類可讀訊息分類。
- 為 TWSE/TPEX 等上游主機加入固定速率限制，並讓重試遵守 `Retry-After`。
- 建立網站資料維護頁，呈現工作狀態、日期、成功/失敗筆數及錯誤摘要。
- 限制同時執行會寫入 SQLite 的作業，並使用既有 `stock-core` pipeline/CLI domain logic。

## Capabilities

### New Capabilities
- `web-data-operations`: 提供網站觸發、監看與取消資料更新作業的功能。

### Modified Capabilities
- `web-background-jobs`: 將資料寫入工作加入第三階段的持久化工作管理器，並維持工作互斥與取消狀態契約。

## Impact

- 擴充 `stock-web` 背景作業協調器、作業 API 與維護頁；使用既有 fetcher、pipeline 與 repository。
- 初版不增加 Redis/RQ、外部佇列或常駐 worker 容器；作業由單一 Axum 程序管理。
- 前置變更：`web-platform-foundation`、`web-market-analysis`、`web-screener`。
