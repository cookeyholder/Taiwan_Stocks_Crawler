## ADDED Requirements

### Requirement: 分離程序存活與服務就緒探測
部署 SHALL 提供 `/health/live` 與 `/health/ready` 端點供容器內部探測。`/health/live` SHALL 只表示程序仍存活，不檢查 SQLite；`/health/ready` SHALL 在 API 可服務且資料庫可用時回報成功。既有 `/api/v1/health` SHALL 保留為 `/health/ready` 的相容別名。Caddy SHALL 不將未驗證的外部請求轉送至健康端點；Compose 的 Web 健康檢查 SHALL 使用 readiness，程序存活探測 SHALL 使用 liveness。

#### Scenario: 健康檢查成功
- **WHEN** 容器或維運者於容器網路內檢查 liveness 與 readiness
- **THEN** 程序存活時 liveness 成功，且 API 與 SQLite 可用時 readiness 成功

#### Scenario: SQLite 不可用
- **WHEN** Web 程序仍存活但 SQLite 無法開啟或查詢
- **THEN** liveness 仍成功、readiness 回報失敗，`/api/v1/health` 亦回報 readiness 失敗

#### Scenario: 載入前端路徑
- **WHEN** 瀏覽器請求網站根路徑或前端頁面
- **THEN** 反向代理回傳靜態網站資源，不會將頁面路徑誤送至 API
