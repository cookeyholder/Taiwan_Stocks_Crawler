## ADDED Requirements

### Requirement: Screener filters by instrument_type
The Bollinger Squeeze screener SHALL allow filtering results by instrument_type.

#### Scenario: Filter by ETF only
- **WHEN** user checks only "ETF" in type filter and clicks "開始本地篩選"
- **THEN** results SHALL only include securities classified as "etf"

#### Scenario: Stock always included
- **WHEN** user unchecks "股票" type (if possible)
- **THEN** system SHALL keep "股票" checked and enabled (stock always shows)

#### Scenario: No types selected defaults to stock
- **WHEN** user unchecks all non-stock types
- **THEN** results SHALL only include "stock" type securities

#### Scenario: Multiple types selected
- **WHEN** user checks "ETF" and "權證"
- **THEN** results SHALL include both "etf" and "warrant" types

### Requirement: Classification rules cover ETF patterns
The classification function SHALL correctly identify ETF symbols from ticker patterns.

#### Scenario: 6-digit ETF identified
- **WHEN** classifying symbol "006208" (6 pure digits)
- **THEN** instrument_type SHALL be "etf"

#### Scenario: Leveraged ETF identified
- **WHEN** classifying symbol "00631L" (5 digits + L)
- **THEN** instrument_type SHALL be "etf"

#### Scenario: Inverse ETF identified
- **WHEN** classifying symbol "00632R" (5 digits + R)
- **THEN** instrument_type SHALL be "etf"

#### Scenario: Bond ETF still identified
- **WHEN** classifying symbol "00679B" (5 digits + B)
- **THEN** instrument_type SHALL be "etf"

### Requirement: Classification identifies convertible bonds
The classification function SHALL identify convertible bond symbols.

#### Scenario: 5-digit convertible bond
- **WHEN** classifying symbol "23301" (5 pure digits)
- **THEN** instrument_type SHALL be "convertible_bond"

### Requirement: Existing data is reclassified
The system SHALL re-classify all existing records in market_quote_core when reclassification is triggered.

#### Scenario: Reclassify updates instrument_type
- **WHEN** reclassify_database() runs
- **THEN** ALL records SHALL have instrument_type updated per classification rules
