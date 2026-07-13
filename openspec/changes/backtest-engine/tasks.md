## 1. 模型與資料結構

- [x] 1.1 在 `models.rs` 中新增 `SignalRecord`、`BacktestOutput`、`Signal` 結構與 `EngineError` 列舉
- [x] 1.2 `SignalRecord` 實作 `to_screen_result()` 轉換方法，依 spec 定義的 metadata key 映射對應既有 `ScreenResult` 欄位

## 2. Backtest 模組基底

- [x] 2.1 新增 `stock-core/src/backtest/mod.rs`，定義 `Strategy` trait（含 `id()`、`min_history()`、`evaluate()`）
- [x] 2.2 新增 `stock-core/src/backtest/engine.rs`，實作 `BacktestEngine`（含 `new()`、`run_date()`、`run_range()` 以及內部資料載入方法）
- [x] 2.3 在 `lib.rs` 中加入 `pub mod backtest`

## 3. 布林通道策略遷移

- [x] 3.1 新增 `stock-core/src/backtest/bollinger.rs`，將 `screen_bollinger_squeeze` 的邏輯改寫為 `BollingerSqueezeBreakout` 策略
- [x] 3.2 從 `db/mod.rs` 中移除 `screen_bollinger_squeeze` 方法與其專屬資料庫查詢常數

## 4. Pipeline 與後向相容

- [x] 4.1 修改 `pipeline/mod.rs` 的 `screen()`，內部改用 `BacktestEngine::run_date()` + `BollingerSqueezeBreakout`
- [x] 4.2 驗證 `cargo build` 通過（Tauri 端 `screen_bollinger` command 不需修改）

## 5. CLI backtest 子命令

- [x] 5.1 在 `stock-cli/src/main.rs` 新增 `Backtest` 子命令列舉（含 `--start`、`--end`、`--max-bandwidth`、`--min-days`、`--types`、`--min-volume` 參數）
- [x] 5.2 實作 `backtest` 命令的執行邏輯，呼叫 `BacktestEngine::run_range()` 並輸出含 `trade_date` 的表格

## 6. 測試

- [x] 6.1 將既有 `test_screen_bollinger_squeeze_empty_on_no_data` 與 `test_screen_bollinger_squeeze_finds_tight_symbol`（`db/mod.rs`）遷移至 `backtest/bollinger.rs` 的 `#[cfg(test)]` 區塊，改用 Engine + Strategy 測試
- [x] 6.2 在 `backtest/engine.rs` 的 `#[cfg(test)]` 區塊新增 `run_date` 指定日期的測試（含無交易日自動回退 scenario）
- [x] 6.3 在 `backtest/engine.rs` 的 `#[cfg(test)]` 區塊新增 `run_range` 回測的測試（驗證多個 `trade_date` 結果）
- [x] 6.4 執行 `cargo test -p stock-core` 確認全部通過
- [x] 6.5 執行 `cargo clippy -p stock-core` 確認無新增 lint 警告

## 7. 文件更新

- [x] 7.1 更新 `docs/guide/命令列工具.md`：新增 `backtest` 子命令說明區段（含 `--start`、`--end` 參數與用法範例）
- [x] 7.2 更新 `docs/development/系統架構.md`：workspace 圖加入 `backtest/` 模組、CLI 命令清單加入 `backtest`、`pipeline::screen()` 標注改用 BacktestEngine
- [x] 7.3 更新 `docs/development/系統架構.md`：模組職責表移除 `db::MarketRepository` 的 `screen_bollinger_squeeze()`、新增 `backtest` 模組列
- [x] 7.4 更新 `docs/development/系統架構.md`：關鍵資料型別表新增 `SignalRecord`（回測輸出，含 trade_date）
