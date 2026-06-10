## ADDED Requirements

### Requirement: Treeview columns are sortable by click
Each column header in the results table SHALL be clickable to sort results by that column.

#### Scenario: Click column header sorts ascending
- **WHEN** user clicks "最新收盤價" column header
- **THEN** results SHALL sort by close_price ascending

#### Scenario: Click same column again reverses sort
- **WHEN** user clicks "最新收盤價" again
- **THEN** results SHALL sort by close_price descending

#### Scenario: Click different column sorts by new column
- **WHEN** user clicks "成交量" after sorting by "最新收盤價"
- **THEN** results SHALL sort by volume ascending
