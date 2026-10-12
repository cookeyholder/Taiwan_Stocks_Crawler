## Context

目前 Cargo workspace 有 `stock-core`、`stock-cli`、`stock-gpui`；核心已包含 SQLite repository、行情抓取與分析。此變更新增網頁入口，不重寫核心。整體五階段順序為：本基礎平台 → 行情分析 → 選股與回測 → 資料作業 → VPS 維運。

## Goals / Non-Goals

**Goals:** 建立所有後續階段共用的 API、靜態前端、驗證、設定及容器部署契約。

**Non-Goals:** 本階段不實作股票查詢、分析、選股、回測或抓取操作頁面；不新增 PostgreSQL、Redis 或前端框架。

## Decisions

- 新增 workspace crate `stock-web`，只負責 HTTP 路由、API DTO、驗證與協調 `stock-core`；不把業務規則搬入 handler。
- Caddy 提供前端靜態檔並反向代理 `/api/v1/*` 至 Axum，維持瀏覽器同源請求；前端使用原生 HTML、CSS、JavaScript。
- 以單一管理者密碼的 Caddy 驗證作為個人 VPS 的最小入口保護；憑證以部署環境提供，HTTPS 由 Caddy 處理。不得把明文密碼或雜湊寫入版本庫。
- Axum 對所有非安全方法的 API 驗證 `Origin` 與設定的公開來源完全相符；拒絕缺少 Origin 的瀏覽器狀態變更請求。這是 Caddy Basic Auth 之外的 CSRF 防線，後續不得繞過 middleware。
- Docker Compose 只需 Caddy 與 Axum 兩個服務；SQLite 由 Axum 單一程序存取並使用 named volume。避免多個 API replica 共寫同一 SQLite 檔案。
- API 固定使用 `/api/v1` 前綴，錯誤 DTO 至少包含穩定錯誤代碼與面向使用者的訊息；之後各階段沿用。

## Risks / Trade-offs

- [外部網路服務暴露登入入口] → 第一階段即要求驗證、HTTPS 與非版本庫憑證設定。
- [SQLite 檔案路徑錯誤造成資料遺失] → 使用明確設定的資料路徑與持久化 volume，並驗證容器重建後資料仍在。
- [共用資料庫有多寫入者] → 第一版維持單一 Axum 實例；資料抓取與維運由同一服務協調。

## Migration Plan

新增 crate 與 Compose 設定，不修改既有桌面/CLI 入口。空白資料庫啟動時執行既有 migration；既有資料可由使用者掛載既有 SQLite 檔案。回退時停止新容器即可，既有桌面與 CLI 繼續使用原資料庫；部署前先備份既有資料檔。
