# Tasks: 三大法人買賣超爬取、入庫與副圖繪製

## 1. 資料庫遷移與核心資料模型 (stock-core)

- [x] 1.1 在 `stock-core/src/db/migrations.rs` 新增 Migration V7（`institutional_investors` 資料表與索引）
- [x] 1.2 在 `stock-core/src/models.rs` 新增 `InstitutionalRecord` 模型
- [x] 1.3 在 `stock-core/src/db/mod.rs` 實作 `upsert_institutional_batch` 與 `get_institutional_records`
- [x] 1.4 在 `stock-core/src/fetcher/twse.rs` 實作 TWSE T86 法人日報表解析器
- [x] 1.5 撰寫核心模組單元測試（遷移、資料庫讀寫、T86 報表解析）

## 2. 應用狀態層與走勢資料對齊 (stock-gpui::state)

- [x] 2.1 在 `stock-gpui/src/state.rs` 之 `AnalysisData` 擴充 `pub institutional: Vec<Option<InstitutionalRecord>>`
- [x] 2.2 在 `CoreState::get_analysis` 查詢法人買賣超並與 OHLCV 日期精確對齊
- [x] 2.3 撰寫狀態層整合測試，驗證法人資料對齊正確性

## 3. 圖表介面與副圖繪製 (stock-gpui::views::analysis)

- [x] 3.1 在 `AnalysisView` 工具列新增「三大法人」切換控制項 (`show_institutional`)
- [x] 3.2 在副圖區塊實作三大法人買賣超柱狀圖（紅漲綠跌 / 買超紅色、賣超綠色）
- [x] 3.3 在頂部十字游標資訊列（HUD）連動展示外資、投信、自營商與合計買賣超數值
- [x] 3.4 撰寫無頭 UI 測試，確認副圖開關與介面渲染正確

## 4. 驗證與品質檢核

- [x] 4.1 執行 `cargo test --workspace` 確保所有測試案例通過
- [x] 4.2 執行 `cargo clippy --workspace --all-targets -- -D warnings` 與 `cargo fmt --all -- --check` 確保零警告與排版規範
- [x] 4.3 封存 OpenSpec 變更並建立 Pull Request 經測試通過後 Squash Merge 併入 `main`
