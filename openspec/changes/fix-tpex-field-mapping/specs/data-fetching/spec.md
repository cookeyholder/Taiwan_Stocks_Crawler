## ADDED Requirements

### Requirement: TPEX 收盤行情欄位正確對映

系統從 TPEX JSON API 擷取資料時，所有價格與數量欄位必須根據 TPEX 實際回應的欄位順序正確對映。

#### Scenario: 收盤價取自 index 2
- **WHEN** 解析 TPEX JSON API 回應的一筆資料列
- **THEN** `close` 欄位值取自陣列 index 2（`fields` 中的「收盤」）

#### Scenario: 開盤價取自 index 4
- **WHEN** 解析 TPEX JSON API 回應的一筆資料列
- **THEN** `open` 欄位值取自陣列 index 4（`fields` 中的「開盤」）

#### Scenario: 最高價取自 index 5
- **WHEN** 解析 TPEX JSON API 回應的一筆資料列
- **THEN** `high` 欄位值取自陣列 index 5（`fields` 中的「最高」）

#### Scenario: 最低價取自 index 6
- **WHEN** 解析 TPEX JSON API 回應的一筆資料列
- **THEN** `low` 欄位值取自陣列 index 6（`fields` 中的「最低」）

#### Scenario: 漲跌取自 index 3
- **WHEN** 解析 TPEX JSON API 回應的一筆資料列
- **THEN** `change` 欄位值取自陣列 index 3（`fields` 中的「漲跌」）

#### Scenario: 成交股數取自 index 8
- **WHEN** 解析 TPEX JSON API 回應的一筆資料列
- **THEN** `volume` 欄位值取自陣列 index 8（`fields` 中的「成交股數」）

#### Scenario: 成交金額取自 index 9
- **WHEN** 解析 TPEX JSON API 回應的一筆資料列
- **THEN** `turnover` 欄位值取自陣列 index 9（`fields` 中的「成交金額」）

#### Scenario: 成交筆數取自 index 10
- **WHEN** 解析 TPEX JSON API 回應的一筆資料列
- **THEN** `trade_count` 欄位值取自陣列 index 10（`fields` 中的「成交筆數」）

### Requirement: TPEX 資料列最小欄位數驗證

系統應以合理的下限檢查資料列完整性。

#### Scenario: 最少需要 11 個欄位
- **WHEN** 資料列長度小於 11（無法涵蓋至成交筆數 index 10）
- **THEN** 該列被跳過，不產生 `ParsedRecord`

### Requirement: TPEX parser 有對應單元測試

系統應包含針對 TPEX JSON 回應格式的單元測試，確保欄位對映正確。

#### Scenario: 至少一個測試案例
- **WHEN** 執行 `cargo test -p stock-core`
- **THEN** 至少有一項測試涵蓋 TPEX JSON 解析，且驗證開高低收與成交量欄位的正確性
