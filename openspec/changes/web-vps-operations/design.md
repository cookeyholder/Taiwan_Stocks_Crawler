## Context

此階段最後執行，前置功能與部署服務均已建立。服務只跑在一台 VPS、一個 Axum 寫入程序及一份 SQLite 資料檔，Caddy 負責 HTTPS/反向代理。

## Goals / Non-Goals

**Goals:** 讓單人維運者能備份、驗證還原、更新與診斷網站，且遇到失敗時保護既有資料。

**Non-Goals:** 不建立高可用叢集、多區備援、雲端物件鎖定或企業級稽核系統；不把 SQLite 遷移到 PostgreSQL。

## Decisions

- 使用 SQLite 官方一致性備份能力（例如 rusqlite backup API）或資料庫連線備份，不直接複製正在寫入的主資料檔；建立後執行完整性檢查。
- 備份檔放在專用持久化目錄，可由管理者透過受驗證介面下載；設定保留數量/天數並在寫入前檢查空間。應用程式負責產生可下載備份，維運者依手冊定期複製到 VPS 外並演練還原，第一版不綁定特定雲端儲存商。
- 還原只透過離線 `stock-web-maintenance restore` 命令：先停止 Axum，將候選檔複製到資料卷同一檔案系統的暫存路徑，分別對候選與現有資料庫跑 integrity check/migration 與 `wal_checkpoint(TRUNCATE)`，關閉連線並確認 checkpoint 完成，再移除已關閉資料庫的 WAL/SHM sidecar，將現有 DB 移至保護副本並原子 rename 候選檔。任何切換前失敗均不得改動現有 DB；啟動後健康檢查失敗時手冊須提供停止服務並還原保護副本的步驟。不得透過正在持有 SQLite 連線的 HTTP handler 替換資料檔。
- Axum 內建一個每日排程器，依可設定的臺北時間（預設 03:00）觸發與手動操作相同的備份工作；程序啟動時只有在已超過當日排程時刻且當日沒有成功記錄時才補做，排程時刻前啟動則等到設定時刻。若工作管理器正忙，排程器待機直到空閒，不可丟棄該日備份。
- 使用映像 tag 固定版本而非 `latest`；更新先備份，再拉取新版本、啟動及檢查；schema migration 僅採向前相容策略，回退前評估是否需恢復資料備份。
- 日誌輸出至標準輸出供 Docker 收集，採結構化欄位且排除機敏值；`/health/live`、`/health/ready` 分別反映程序與資料庫狀態，Compose probe 使用 readiness；既有 `/api/v1/health` 保留為 readiness 別名，Caddy 不公開健康路由。

## Risks / Trade-offs

- [SQLite schema migration 無法保證逆向] → 更新前強制/提示備份，回退文件明確區分映像回退與資料回復。
- [主機磁碟不足使備份或抓取失敗] → 備份前檢查空間、報告失敗，維運手冊列出容量檢查。
- [還原操作覆蓋最新資料] → 使用二次確認和還原前保護副本，只有檢查通過才切換資料檔。

## Migration Plan

分批部署維運功能但不改行情資料格式。先人工備份並演練還原，再啟用排程備份；更新映像前先保存目前 tag 與 SQLite 備份。失敗時回復舊 tag，必要時以保護副本還原資料。
