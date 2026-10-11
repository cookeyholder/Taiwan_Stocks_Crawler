# backtest-ui Specification

## Purpose

提供回測系統之使用者介面，包含策略選擇、動態參數輸入、績效指標摘要、訊號列表與交易明細。

## Requirements

### Requirement: 策略選擇器
系統 SHALL 顯示下拉選單列出所有可用策略，選擇後動態顯示對應的參數輸入欄位。

#### Scenario: 選擇策略
- **WHEN** 使用者從下拉選單選擇「MA 交叉」
- **THEN** 參數表單顯示快速週期（預設 5）、慢速週期（預設 20）欄位

### Requirement: 績效摘要
系統 SHALL 在回測完成後以卡片方式顯示總報酬率、勝率、Sharpe、最大回撤。

#### Scenario: 回測完成後顯示
- **WHEN** 回測執行完畢
- **THEN** 四個卡片分別顯示對應的績效數據

### Requirement: 訊號表格
系統 SHALL 以表格顯示所有訊號，支援點擊欄位排序。

#### Scenario: 排序訊號
- **WHEN** 使用者點擊「日期」欄位標題
- **THEN** 表格依日期排序

### Requirement: 交易明細
系統 SHALL 以表格顯示所有已了結交易，包含買入/賣出日期、價格、損益。

#### Scenario: 交易顯示
- **WHEN** 回測完成且有交易
- **THEN** 每筆交易顯示買入日、賣出日、代號、股數、損益百分比
