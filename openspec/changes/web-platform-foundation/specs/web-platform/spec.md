## Purpose

提供個人於 VPS 自架網站所需的受保護 Web 入口、同源 API 路由與 SQLite 持久化基礎，讓後續功能可以逐階段接入而不改變桌面與 CLI 使用方式。

## ADDED Requirements

### Requirement: 同源網站與版本化 API
網站 SHALL 由一個公開網域提供靜態前端與 `/api/v1/` JSON API，並以一致的 JSON 格式回報 API 錯誤。

#### Scenario: 前端呼叫 API
- **WHEN** 使用者從網站載入頁面並呼叫 `/api/v1/` 路徑
- **THEN** 請求在同一來源抵達 Web API，瀏覽器不需要跨來源設定

#### Scenario: API 發生錯誤
- **WHEN** API 請求因驗證、找不到資料或伺服器錯誤而失敗
- **THEN** 回應包含適當 HTTP 狀態碼及一致結構的 JSON 錯誤內容

### Requirement: VPS 網站存取保護
網站 SHALL 在對外公開前要求管理者通過登入驗證，且未驗證請求不得取得網站功能或呼叫 API。

#### Scenario: 未登入存取網站
- **WHEN** 使用者未通過驗證而開啟網站或呼叫 API
- **THEN** 系統拒絕請求並要求完成登入

#### Scenario: 登入後呼叫同源 API
- **WHEN** 管理者成功登入並於同一來源呼叫 API
- **THEN** API 接受請求，且不要求額外的瀏覽器跨來源授權

### Requirement: 防止跨來源狀態變更請求
所有會改變資料或啟動、取消作業的 API SHALL 驗證請求來源符合設定的網站來源；缺少 Origin 或來源不符的瀏覽器狀態變更請求 SHALL 被拒絕。

#### Scenario: 同源狀態變更請求
- **WHEN** 已驗證使用者從設定的網站來源送出狀態變更 API 請求
- **THEN** API 依請求內容執行操作

#### Scenario: 跨來源或缺少來源標頭
- **WHEN** 狀態變更 API 請求的 Origin 不符設定值或缺少 Origin
- **THEN** API 回傳拒絕狀態且不執行任何狀態變更

### Requirement: SQLite 資料持久化
Web 服務 SHALL 使用既有 `stock-core` 資料模型與 migration，並將 SQLite 資料儲存於容器生命週期之外的持久化位置。

#### Scenario: 容器重新建立後保留資料
- **WHEN** Web 容器停止並重新建立
- **THEN** 已提交的行情與應用資料仍可由 SQLite 讀取

#### Scenario: 指定資料庫位置
- **WHEN** Web 服務啟動時收到資料庫路徑設定
- **THEN** 服務使用該路徑開啟資料庫並執行既有 migrations

### Requirement: Axum 僅供 Caddy 內部連線
Docker Compose SHALL 僅將 Caddy 的 HTTP/HTTPS 連接埠發布至主機；Axum SHALL 僅連接 Compose 內部網路，不得發布主機連接埠。容器內健康檢查 SHALL 直接探測 Axum，不得為健康檢查開放外部連線。

#### Scenario: 從外部網路直接連線 Axum
- **WHEN** 部署完成後從 VPS 外部網路連線 Axum 服務連接埠
- **THEN** 連線無法建立，對外 HTTP/HTTPS 請求只能經過 Caddy 驗證與路由

#### Scenario: 容器健康檢查 Axum
- **WHEN** Docker 執行 Axum 容器健康檢查
- **THEN** 探測在容器內或 Compose 內部網路完成，不發布 Axum 主機連接埠

### Requirement: 靜態網站與 API 健康檢查
部署 SHALL 提供基本健康檢查，並讓反向代理將 API 請求交給 Web 服務、其他網站路徑交給靜態檔案服務。

#### Scenario: 健康檢查成功
- **WHEN** 容器或維運者檢查 Web API 健康狀態
- **THEN** API 回報服務可連線且資料庫可用

#### Scenario: 載入前端路徑
- **WHEN** 瀏覽器請求網站根路徑或前端頁面
- **THEN** 反向代理回傳靜態網站資源，不會將頁面路徑誤送至 API
