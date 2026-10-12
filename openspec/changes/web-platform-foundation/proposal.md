## Why

目前系統只有 CLI 與 GPUI 桌面入口，無法從瀏覽器使用，也無法直接部署成 VPS 網站。需要先建立可部署的 Web 殼層與穩定 API 邊界，讓後續行情、選股及資料維護功能都能共用既有 Rust 核心與 SQLite。

## What Changes

- 新增 Axum Web 服務，提供版本化 JSON API 與健康檢查。
- 以 Caddy 提供原生 HTML、CSS、JavaScript 靜態檔案，並將同源 `/api/v1/*` 轉送至 Axum。
- 以 Docker Compose 管理 Caddy 與 Axum；SQLite 使用持久化資料卷。
- VPS 對外開放前加入單一管理者登入保護，HTTPS 與 API 採同一網域。
- 建立共用 API 錯誤格式、設定來源與基本啟動/關閉流程。

## Capabilities

### New Capabilities
- `web-platform`: 提供有登入保護、同源 API、靜態頁面與持久化資料庫的個人網站基礎平台。

### Modified Capabilities

## Impact

- Rust workspace 新增 `stock-web` 服務並依賴 `stock-core`；新增 Axum、序列化與設定所需套件。
- 新增原生前端檔案、Caddy 設定、Dockerfile 與 Compose 部署檔。
- SQLite schema 沿用既有 migrations；網站須以設定值指定資料庫路徑，不能依賴容器工作目錄。
- 不變更既有 `stock-cli` 與 `stock-gpui` 的使用方式。
