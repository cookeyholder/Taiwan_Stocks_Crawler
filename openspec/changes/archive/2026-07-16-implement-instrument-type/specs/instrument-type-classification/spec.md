## ADDED Requirements

### Requirement: 資料擷取時自動分類 instrument_type

系統 SHALL 在 `normalize_records` 中根據資料來源與 symbol 規則設定正確的 `instrument_type`。

#### Scenario: TPEX 上櫃股票分類為 stock
- **WHEN** 擷取 TPEX 資料且 table title 包含「上櫃股票」
- **THEN** 該筆記錄的 `instrument_type` 設為 `stock`

#### Scenario: TPEX ETF 分類為 etf
- **WHEN** 擷取 TPEX 資料且 table title 包含「上櫃ETF」
- **THEN** 該筆記錄的 `instrument_type` 設為 `etf`

#### Scenario: TWSE 一般股票分類為 stock
- **WHEN** 擷取 TWSE 資料且 symbol 為 4 位數、首位為 1-9（如 2330）
- **THEN** 該筆記錄的 `instrument_type` 設為 `stock`

#### Scenario: TWSE ETF 分類為 etf
- **WHEN** 擷取 TWSE 資料且 symbol 以 `0` 開頭（如 0050、00878）
- **THEN** 該筆記錄的 `instrument_type` 設為 `etf`

#### Scenario: TWSE 權證分類為 warrant
- **WHEN** 擷取 TWSE 資料且 symbol 為 5 位數、以 03 開頭
- **THEN** 該筆記錄的 `instrument_type` 設為 `warrant`

#### Scenario: TWSE ETN 分類為 etn
- **WHEN** 擷取 TWSE 資料且 symbol 為 5 位數、以 02 開頭
- **THEN** 該筆記錄的 `instrument_type` 設為 `etn`

#### Scenario: 無法分類時回退 other
- **WHEN** 資料不符以上任何規則
- **THEN** `instrument_type` 設為 `other`

### Requirement: TPEX parser 處理所有表格

TPEX parser SHALL 不再跳過非「上櫃股票」的表格，改為依 table title 設定分類。

#### Scenario: 處理 TPEX 多個表格
- **WHEN** TPEX API 回傳包含「上櫃股票」和「上櫃ETF」等多個表格
- **THEN** 每個表格的資料都被處理，各自依 title 分類

### Requirement: 前端布林選股頁面新增類型篩選

系統 SHALL 在 ScreenerView 提供 `instrument_type` 篩選 UI，讓使用者可過濾股票類型。

#### Scenario: 顯示類型篩選控制項
- **WHEN** 使用者開啟布林選股頁面
- **THEN** 頁面上顯示類型多選控制項（stock/etf/warrant/etn）
- **THEN** 預設勾選 `stock` 類型

#### Scenario: 篩選特定類型
- **WHEN** 使用者勾選 `etf` 類型並執行篩選
- **THEN** 系統只回傳 `instrument_type` 為 `etf` 的結果
