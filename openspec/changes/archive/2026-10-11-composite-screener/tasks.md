# Tasks: 均線多頭排列、創波段新高複合選股器

## 1. 核心策略與資料模型 (stock-core)

- [x] 1.1 在 `stock-core/src/models.rs` 擴充 `ScreenResult`，加入 `#[serde(default)] pub tags: Vec<String>`
- [x] 1.2 實作 `stock-core/src/backtest/ma_alignment.rs`：`MaBullishAlignmentStrategy` 及其評估邏輯
- [x] 1.3 實作 `stock-core/src/backtest/new_high.rs`：`NewHighBreakoutStrategy` 及其評估邏輯
- [x] 1.4 在 `stock-core/src/backtest/strategies.rs` 註冊 `MaBullishAlignment` 與 `NewHighBreakout` 至 `StrategyKind` 與 `StrategyInfo` 清單
- [x] 1.5 撰寫核心策略單元測試 (`stock-core/tests/ma_alignment_test.rs`、`stock-core/tests/new_high_test.rs`)，確保所有測試通過

## 2. 桌面應用狀態層 (stock-gpui::state)

- [x] 2.1 定義 `ScreenerMode` 列舉（布林擠壓、多頭排列、創波段新高、複合選股）
- [x] 2.2 在 `CoreState` 實作 `screen_composite_range`，支援多模式與複合條件篩選及標籤附加
- [x] 2.3 撰寫狀態層整合測試，驗證複合選股之交集過濾與標籤指派

## 3. 選股介面視覺整合 (stock-gpui::views::screener)

- [x] 3.1 在 `ScreenerView` 新增策略模式選擇器及對應參數配置項
- [x] 3.2 在選股結果表格新增「策略標籤」欄位並以徽章呈現
- [x] 3.3 撰寫選股介面無頭測試，確認介面操作與結果渲染無誤

## 4. 驗證與程式碼品質檢核

- [x] 4.1 執行 `cargo test --workspace` 確保所有測試案例通過
- [x] 4.2 執行 `cargo clippy --workspace --all-targets -- -D warnings` 與 `cargo fmt --all -- --check` 確保零警告與排版規範
- [x] 4.3 封存 OpenSpec 變更並建立 Pull Request 經測試通過後 Squash Merge 併入 `main`
