# ag-grid-table Specification

## Purpose
TBD - created by archiving change migrate-to-tauri-rust. Update Purpose after archive.
## Requirements
### Requirement: 篩選結果表格
表格需顯示布林擠壓篩選結果，欄位包含：symbol、name、latest_close、latest_volume、max_bandwidth、latest_bandwidth。

#### Scenario: 顯示結果
- **WHEN** 篩選器回傳結果
- **THEN** 表格需顯示所有資料列，數值需格式化（價格 2 位小數、成交量千分位、頻寬百分比）

### Requirement: 欄位排序
每個欄位標題需支援點擊切換遞增/遞減排序。

#### Scenario: 依欄位排序
- **WHEN** 使用者點擊欄位標題
- **THEN** 表格需依該欄位排序，交替遞增與遞減

### Requirement: 選取列觸發圖表更新
點擊表格列需更新 ECharts 圖表顯示該股票的資料。

#### Scenario: 點擊列更新圖表
- **WHEN** 使用者點擊某一列
- **THEN** 系統需呼叫 Rust 指令取得該股票的圖表資料並更新 ECharts

