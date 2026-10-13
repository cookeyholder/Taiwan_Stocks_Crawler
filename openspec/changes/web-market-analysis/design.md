## Context

此階段以前一階段 `web-platform-foundation` 為前置，沿用 `stock-web`、`/api/v1`、Caddy 同源路由及 SQLite。`stock-core` 已提供 OHLCV 查詢與技術指標純函式；GPUI 的圖表不可直接重用於瀏覽器。

## Goals / Non-Goals

**Goals:** 提供唯讀搜尋、行情與分析 API，以及可用瀏覽器操作的個股頁。

**Non-Goals:** 不新增前端框架或圖表套件；不在查詢請求中抓取來源行情；不修改既有分析算法。

## Decisions

- API 由 `stock-web` 轉換現有核心資料為明確 JSON DTO，不直接序列化內部 DB row，避免前端綁定資料庫欄位。證券搜尋直接彙整 `market_quote_core` 中每個代號最新的名稱與市場別，不依賴僅由 `sync_securities()` 更新的獨立表。
- 查詢日期參數使用 ISO `YYYY-MM-DD`；路由採 `/api/v1/securities`、`/api/v1/securities/{symbol}/quotes` 與 `/api/v1/securities/{symbol}/analysis`，顯示範圍預設兩年、上限五年，錯誤沿用階段一格式。
- 個股分析查詢從該股票最早可用日期讀取至顯示迄日，在完整歷史序列上計算 SMA、布林、RSI、MACD，再裁切回傳顯示起訖日期；回應的 `trade_date` 和 `ingested_at` 分別表示交易日與最後匯入時間。
- 原生 JavaScript 使用 `fetch` 與 DOM/SVG 建立圖表，先支援必要的日 K、成交量、指標疊圖及游標提示；避免新建 npm 工具鏈或引入大型圖表依賴。
- 不查詢全市場完整行情；單股顯示範圍最多五年，但指標計算會使用該股顯示迄日以前的完整已存歷史。

## Risks / Trade-offs

- [純 SVG 圖表在極長區間可能變慢] → 限制單次回傳筆數並測試常用區間，日後確有需要再選圖表套件。
- [前端 DTO 和核心型別不一致] → 以 API 整合測試固定 JSON 欄位與日期/數值格式。
- [歷史資料缺漏造成圖表斷線] → 保留日期缺口，不插入虛構行情。

## Migration Plan

無資料庫 schema 變更。完成階段一後部署新版 Axum API 與前端靜態檔；回退可還原上一版容器映像，資料庫不需反向 migration。
