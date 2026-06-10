## 1. Classification Improvements

- [x] 1.1 Add new regex patterns to `_from_ticker_pattern()`: 6-digit ETFs, 5-digit+letter ETFs, 5-digit convertible bonds
- [x] 1.2 Add new instrument_type string `"convertible_bond"` and update return type hints
- [x] 1.3 Add `reclassify_database()` function to classification.py
- [x] 1.4 Update existing tests and add new test cases for all new patterns

## 2. Screener Parameter Expansion

- [x] 2.1 Add `instrument_types` and `min_volume` parameters to `screen_bollinger_squeeze()`
- [x] 2.2 Apply type filter in SQL query (`WHERE instrument_type IN (...)`)
- [x] 2.3 Apply volume filter in results (on latest_volume)

## 3. GUI Type Filter & Volume Input

- [x] 3.1 Add instrument type checkboxes (stock fixed, ETF/warrant/convertible_bond/preferred/other toggleable)
- [x] 3.2 Add minimum volume input field
- [x] 3.3 Pass filter values from GUI to screener via task queue

## 4. GUI Column Sorting

- [x] 4.1 Add sort state tracking (`t2_sort_col`, `t2_sort_asc`)
- [x] 4.2 Bind column header click events to toggle sort direction
- [x] 4.3 Implement Treeview re-sorting without re-running screener

## 5. K-line Chart

- [x] 5.1 Add mplfinance dependency to pyproject.toml and install it
- [x] 5.2 Replace line chart with candlestick chart + Bollinger bands overlay
- [x] 5.3 Color volume bars by price direction (red/green)

## 6. DB Migration & Integration

- [x] 6.1 Wire `reclassify_database()` into GUI startup (background thread)
- [x] 6.2 Run full test suite and fix any failures
- [ ] 6.3 Commit and create PR
