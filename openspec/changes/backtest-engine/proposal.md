## Why

現有 `screen_bollinger_squeeze` 將資料載入、指標計算、策略判斷全部綁死在 `db/mod.rs`，無法抽換其他選股策略，也無法對歷史日期進行回測。需要一個可擴展的回測引擎抽象層。

## What Changes

- **新增 `backtest/` 模組**：包含 `Strategy` trait、`BacktestEngine`（`run_date` / `run_range`）、`SignalRecord`
- **`db/mod.rs` 移除 `screen_bollinger_squeeze`**：策略邏輯移至 `backtest/bollinger.rs`
- **`pipeline.rs` 的 `screen()` 改為內部呼叫 `BacktestEngine`**：行為不變，實作換掉
- **`models.rs` 新增 `SignalRecord` / `BacktestOutput`**：含 `trade_date` 欄位支援回測
- **`stock-cli` 新增 `backtest` 子命令**：支援 `--start`、`--end` 日期範圍參數
- Tauri/Vue 端完全不動，向後相容

## Capabilities

### New Capabilities

- `backtest-engine`: 回測引擎抽象層，含可插拔策略 trait、單日篩選（run_date）與日期範圍回測（run_range）

### Modified Capabilities

（無 — 此為新功能，不修改既有 spec 的行為）

## Impact

- `stock-core/src/backtest/`：新增模組（engine, trait, bollinger strategy）
- `stock-core/src/db/mod.rs`：刪除 `screen_bollinger_squeeze`
- `stock-core/src/models.rs`：新增結構
- `stock-core/src/pipeline/mod.rs`：`screen()` 改用 Engine
- `stock-cli/src/main.rs`：新增 `backtest` 子命令
- Tauri backend、Vue frontend：無影響
