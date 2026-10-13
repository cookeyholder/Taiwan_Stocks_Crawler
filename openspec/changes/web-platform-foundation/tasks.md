## 1. Web 服務骨架

- [ ] 1.1 將 `stock-web` 加入 workspace，建立 Axum 啟動入口與設定結構；以 `cargo check -p stock-web` 確認可編譯。
- [ ] 1.2 建立 `/api/v1/health` 與一致 JSON 錯誤回應；以路由測試驗證成功及錯誤狀態碼。
- [ ] 1.3 接上 `stock-core` repository 與設定的 SQLite 路徑；以暫存資料庫測試 migration 與健康檢查。

## 2. 靜態前端與存取保護

- [ ] 2.1 建立無框架的首頁、CSS 與 JavaScript 入口；以瀏覽器確認根路徑可載入靜態資源。
- [ ] 2.2 設定 Caddy 同源靜態檔案與 `/api/v1/*` 反向代理；以本機 HTTP 請求確認兩種路徑分流正確。
- [ ] 2.3 加入 VPS 管理者驗證與 HTTPS 設定範本，將密碼憑證改由環境提供；確認未登入頁面與 API 均受拒絕。
- [ ] 2.4 加入狀態變更 API 的 Origin 驗證 middleware；路由測試確認同源請求可用、異源與缺少 Origin 的請求不會執行 handler。

## 3. Docker 部署

- [ ] 3.1 建立 Web 多階段 Dockerfile 與 Compose 服務；以 `docker compose config` 確認設定有效。
- [ ] 3.2 掛載 SQLite named volume 並設定容器內健康檢查；確認 Compose 只有 Caddy 發布主機連接埠、Axum 無法從主機直接連線，且重建容器後測試資料仍存在。
- [ ] 3.3 撰寫本機啟動與 VPS 環境變數說明；依文件啟動 Compose 並成功呼叫健康 API。

## 4. 整合驗證

- [ ] 4.1 執行 `cargo test --workspace`、`cargo clippy --workspace --all-targets -- -D warnings` 與 `cargo fmt --all -- --check`，確認既有桌面及 CLI 未受影響。
- [ ] 4.2 以 Docker Compose 驗證登入、靜態頁、健康 API、SQLite 持久化及容器重啟流程。
