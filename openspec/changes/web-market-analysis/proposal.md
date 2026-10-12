## Why

網站平台提供了安全且穩定的入口，但使用者仍無法從瀏覽器查詢本機 SQLite 中的行情並檢視個股。此階段先移植最常用的唯讀行情與技術分析流程，建立後續選股頁可共用的資料與圖表呈現方式。

## What Changes

- 新增股票代號/名稱查詢、上市櫃證券清單與歷史行情 API。
- 新增個股分析 API，重用 `stock-core` 的 OHLCV 與技術指標計算。
- 建立原生 JavaScript 個股分析頁，支援日期範圍與 K 線/成交量/技術指標圖表。
- 沿用基礎平台的登入保護、`/api/v1`、JSON 錯誤格式與同源路由。

## Capabilities

### New Capabilities
- `web-market-analysis`: 提供網站股票搜尋、歷史行情、個股技術分析與互動圖表。

### Modified Capabilities

## Impact

- 擴充 `stock-web` API 與靜態頁；讀取 `stock-core::MarketRepository`、`pipeline` 與 analysis 型別。
- 僅新增唯讀查詢，不修改 SQLite schema，也不觸發資料抓取。
- 前置變更：`web-platform-foundation` 必須先完成並部署其 API、驗證及靜態服務契約。
