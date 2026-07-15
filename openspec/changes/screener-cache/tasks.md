## 0. `CacheParams` 結構體（前置作業）

- [x] 0.1 在 `stock-core/src/models.rs` 新增 `CacheParams` struct（含 `query_date`, `instrument_types`, `max_bandwidth`, `min_days`, `min_volume` 五個欄位），derive `Serialize, Deserialize`
- [x] 0.2 確認 `CacheParams` 在 `stock-core/src/lib.rs` 的 `pub use` 中有匯出（已由 `pub mod models` 自動涵蓋）

## 1. 先寫測試（TDD Red）

- [x] 1.1 在 `db::tests` 新增 `test_screener_cache_write_and_read`：寫入一組參數與結果，用 `get_cached_screening` 讀回，assert 內容一致
- [x] 1.2 新增 `test_screener_cache_miss`：查詢不存在的 key，assert 回傳 `None`
- [x] 1.3 新增 `test_screener_cache_overwrite`：同一 key 寫入兩次，assert 第二次覆蓋成功
- [x] 1.4 新增 `test_screener_cache_clear`：寫入多筆後清除，assert 回傳正確刪除筆數且查詢全 miss
- [x] 1.5 新增 `test_screener_cache_corrupted_json`：手動 INSERT 壞 JSON，assert `get_cached_screening` 回傳 `None`
- [x] 1.6 在 `pipeline::tests` 新增 `test_build_cache_key_deterministic`：相同參數產生相同 key，不同參數產生不同 key
- [x] 1.7 新增 `test_build_cache_key_types_order`：`["etf","stock"]` 與 `["stock","etf"]` 產生相同 key
- [x] 1.8 新增 `test_build_cache_key_version`：不同版本號產生不同 key
- [x] 1.9 `cargo test` 確認上述測試編譯但因尚未實作而**預期失敗**（TDD Red 完成）

## 2. 資料庫 Migration（TDD Green）

- [x] 2.1 在 `migrations.rs` 新增 V5：`CREATE TABLE IF NOT EXISTS screener_cache`
- [x] 2.2 在 `test_fresh_migration` 中新增 assert 確認 `screener_cache` 表存在
- [x] 2.3 `cargo test` 確認 1.1~1.5 的測試開始通過（DB 層齊全）

## 3. MarketRepository 快取方法（TDD Green）

- [x] 3.1 實作 `MarketRepository::get_cached_screening(&self, key: &str) -> SqlResult<Option<Vec<ScreenResult>>>`
- [x] 3.2 實作 `MarketRepository::set_cached_screening(&self, key: &str, params: &CacheParams, results: &[ScreenResult]) -> SqlResult<()>`
- [x] 3.3 實作 `MarketRepository::clear_screener_cache(&self) -> SqlResult<usize>`（回傳刪除筆數）
- [x] 3.4 `cargo test` 確認 1.1~1.5 全部通過（DB 層 TDD Green）

## 4. `build_cache_key()` 與 Tauri Command

- [x] 4.1 在 `lib.rs` 定義 `const SCREEN_CACHE_VERSION: &str = "v1"`
- [x] 4.2 新增 `build_cache_key()` 輔助函式（在 `pipeline/mod.rs`，供 lib.rs 呼叫）
- [x] 4.3 `cargo test` 確認 1.6~1.8 通過（key 相關測試齊全）
- [x] 4.4 實作 `screen_bollinger_with_cache` command：先查快取，miss 則篩選後存入再回傳
      - 若 `get_cached_screening` 因 corrupted JSON 回傳 `None`，視同 miss，重新計算後寫入覆蓋
      - 若 `set_cached_screening` 寫入失敗，log error 但仍回傳計算結果
- [x] 4.5 用 `screen_bollinger_with_cache` 取代原本的 `screen_bollinger` command
      - 從 `lib.rs` 移除舊 `screen_bollinger` 函式
      - 從 `invoke_handler` 的 `generate_handler!` 巨集中移除 `screen_bollinger`
      - 更名前先確認 `screen_bollinger` 未被其他 openspec change / 外部參考引用
- [x] 4.6 實作 `clear_screener_cache` command 呼叫 `repo.clear_screener_cache()`，回傳刪除筆數
- [x] 4.7 在 `invoke_handler` 的 `generate_handler!` 巨集中註冊新 command

## 5. 前端 Store + UI

- [ ] 5.1 修改 `screener.ts` 的 `runScreen()`，將 `invoke("screen_bollinger")` 改為 `invoke("screen_bollinger_with_cache")`
- [ ] 5.2 新增 `clearing` loading state（`ref(false)`）
- [ ] 5.3 新增 `clearCache()` 方法，呼叫 `invoke("clear_screener_cache")`，期間 `clearing.value = true`，錯誤時寫入 `error.value`
- [ ] 5.4 在 store 的 return 中匯出 `clearCache` 與 `clearing`
- [ ] 5.5 在 `ScreenerView.vue` 的 params bar 中，於「開始篩選」按鈕旁新增「清除快取」按鈕
- [ ] 5.6 按鈕 disabled 條件：`store.loading || store.dateLoading || store.clearing`
- [ ] 5.7 按鈕文字：`store.clearing ? "清除中..." : "清除快取"`
- [ ] 5.8 清除成功後顯示「已清除 X 筆快取」

## 6. 格式化與最終驗證

- [ ] 6.1 `cargo fmt` 格式化所有 Rust 程式碼
- [ ] 6.2 `cargo fmt --all -- --check` 確認格式化無誤
- [ ] 6.3 `cargo check` 確認編譯無誤
- [ ] 6.4 `cargo test` 確認全部測試通過
- [ ] 6.5 手動驗證快取命中與清除功能正常
