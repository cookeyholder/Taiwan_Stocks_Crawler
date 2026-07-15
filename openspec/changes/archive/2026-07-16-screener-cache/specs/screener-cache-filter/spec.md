## ADDED Requirements

### Requirement: 篩選結果快取查詢
系統 SHALL 在收到篩選請求時，先以目前參數組合出 cache key 查詢 `screener_cache` 表。若命中則直接回傳快取的結果，不執行 BacktestEngine。

#### Scenario: 快取命中
- **WHEN** 使用者按下「開始篩選」，且該組參數已存在 `screener_cache` 表中
- **THEN** 系統直接回傳快取中的結果，不執行 `BacktestEngine::run_date()`

#### Scenario: 快取未命中
- **WHEN** 使用者按下「開始篩選」，且該組參數不存在於 `screener_cache` 表中
- **THEN** 系統執行完整的 `BacktestEngine::run_date()` 篩選計算，將結果序列化為 JSON 存入 `screener_cache` 表，再回傳結果

### Requirement: Cache key 確定性
系統 SHALL 使用一致且可重現的 cache key 來識別篩選條件組合。key 格式為 `{版本}|{日期}|{類型}|{帶寬}|{天數}|{成交量}`。類型陣列 SHALL 先排序再合併。

#### Scenario: 相同參數產生相同 key
- **WHEN** 兩次篩選請求使用完全相同的五個參數
- **THEN** 兩次請求的 cache key 完全一致

#### Scenario: 參數順序不影響 key
- **WHEN** 前端傳入 `instrumentTypes: ["stock", "etf"]` 與 `["etf", "stock"]`
- **THEN** 兩者產生相同的 cache key（因類型陣列排序後合併）

### Requirement: 演算法版本號
系統 SHALL 在 cache key 中包含一個版本字串。版本變更時所有舊快取自動失效。

#### Scenario: 版本遞增使快取失效
- **WHEN** 版本從 `v1` 改為 `v2`
- **THEN** 即使參數完全相同，產生的 cache key 也不同，所有舊快取不被使用

### Requirement: 空結果快取
當篩選結果為空陣列時，系統 SHALL 同樣存入快取。

#### Scenario: 空結果快取
- **WHEN** 使用者選擇某日期，篩選結果為 0 筆
- **THEN** 系統將空陣列 `[]` 存入 `screener_cache` 表
- **AND** 下次同一參數再次查詢時，直接回傳空陣列而不重算

### Requirement: 手動清除快取
系統 SHALL 提供清除全部快取的 Tauri command，前端頁面 SHALL 顯示對應的按鈕。

#### Scenario: 清除快取
- **WHEN** 使用者點擊「清除快取」按鈕
- **THEN** 系統執行 `DELETE FROM screener_cache`
- **AND** 清除後再次篩選同一組參數時，重新計算結果

#### Scenario: 清除快取 loading 狀態
- **WHEN** 使用者點擊「清除快取」按鈕
- **THEN** 按鈕立即 disabled，文字變為「清除中...」
- **AND** 清除完成後恢復 enabled
- **AND** 若清除失敗，顯示錯誤訊息

#### Scenario: 快取資料損毀
- **WHEN** 某筆 `screener_cache` 記錄的 `results_json` 內容無法被 `serde_json` 解析
- **THEN** 系統 SHALL 視同 cache miss，記錄 warning log 後重新執行篩選計算
- **AND** 將正確結果寫入快取（覆蓋損毀的記錄）
- **AND** 不 crash、不回傳錯誤給使用者
