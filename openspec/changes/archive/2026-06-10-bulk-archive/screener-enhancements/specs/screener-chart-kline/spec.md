## ADDED Requirements

### Requirement: Chart shows candlesticks with Bollinger bands
The right-side detail chart SHALL display Japanese candlesticks (OHLC) instead of a line chart, with Bollinger bands overlaid.

#### Scenario: Candlestick chart renders
- **WHEN** user clicks a stock row in the results table
- **THEN** the chart SHALL show candlesticks for each trading day with Bollinger upper/middle/lower bands

#### Scenario: Volume bars colored by price direction
- **WHEN** the chart renders
- **THEN** volume bars SHALL be green when close >= open and red when close < open
