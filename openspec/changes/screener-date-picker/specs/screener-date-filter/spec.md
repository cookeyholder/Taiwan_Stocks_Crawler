## ADDED Requirements

### Requirement: 使用者可以從布林選股頁面選擇特定日期篩選

系統 SHALL 在布林通道選股頁面提供日期選擇功能，讓使用者指定交易日篩選突破訊號。

#### Scenario: 預設顯示最新交易日
- **WHEN** 使用者開啟布林選股頁面
- **THEN** 日期欄位自動填入資料庫中最新的交易日
- **THEN** 系統自動以該日期執行篩選

#### Scenario: 使用者選擇日期後篩選
- **WHEN** 使用者選取特定日期（如 2026-06-01）
- **THEN** 系統以該日期執行篩選
- **THEN** 篩選結果顯示在該交易日有突破訊號的股票

#### Scenario: 日期無對應資料
- **WHEN** 使用者選取無交易資料的日期（如週末或休市日）
- **THEN** 系統回傳空結果
- **THEN** 顯示提示訊息「該日期無交易資料」

### Requirement: 後端支援指定日期篩選

`Pipeline::screen()` SHALL 接受 `Option<&str>` 日期參數，傳入時直接以該日期篩選。

#### Scenario: 傳入日期參數
- **WHEN** `screen()` 被呼叫且傳入 `Some("2026-06-01")`
- **THEN** 系統使用 `BacktestEngine::run_date("2026-06-01", ...)` 而非最新交易日

#### Scenario: 不傳日期參數（向後相容）
- **WHEN** `screen()` 被呼叫且傳入 `None`
- **THEN** 系統使用 `repo.get_latest_trade_date()` 取得最新交易日

### Requirement: CLI 支援 --date 選項

CLI `screen` 子命令 SHALL 新增 `--date` 選項。

#### Scenario: CLI 傳入 --date
- **WHEN** 執行 `stock-cli screen --date 2026-06-01`
- **THEN** 系統以該日期篩選並輸出結果

#### Scenario: CLI 不傳 --date
- **WHEN** 執行 `stock-cli screen`（不帶 `--date`）
- **THEN** 系統以最新交易日篩選（向後相容）
