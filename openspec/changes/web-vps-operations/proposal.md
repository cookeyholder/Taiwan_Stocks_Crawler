## Why

網站功能完成後，個人 VPS 必須能安全保存唯一一份 SQLite 資料、升級映像並在故障時還原。若沒有可驗證的備份、升級與復原流程，容器化雖能啟動服務，仍不足以可靠長期使用。

## What Changes

- 建立 SQLite 線上安全備份、下載與驗證還原流程，還原前自動保護目前資料。
- 建立版本標記、資料庫 migration 升級檢查與可回退的容器更新步驟。
- 補強健康/就緒狀態、結構化日誌、磁碟空間與作業失敗等維運可觀測性。
- 將手動與排程備份納入共用背景工作管理器，明定忙碌、排程等待及重啟復原行為。
- 要求備份定期保存至 VPS 以外並完成還原驗證。
- 提供 VPS 初次部署、更新、備份、還原與復原演練文件。

## Capabilities

### New Capabilities
- `web-vps-operations`: 提供個人 VPS 網站的資料保護、升級復原與健康維運能力。

### Modified Capabilities
- `web-platform`: 新增分離的 liveness/readiness 路徑與容器健康檢查契約。
- `web-background-jobs`: 新增備份工作的持久化、查詢、忙碌處理及重啟復原契約。

## Impact

- 擴充 `stock-web` 維運 API/狀態與部署文件、Compose/Caddy 設定。
- 可能新增維運備份記錄資料表或 metadata；行情表 schema 維持不變。
- 前置變更：`web-platform-foundation`、`web-market-analysis`、`web-screener`、`web-data-operations`。
