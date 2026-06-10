## ADDED Requirements

### Requirement: Screener filters by minimum volume
The Bollinger Squeeze screener SHALL allow filtering results by a minimum volume threshold.

#### Scenario: Volume filter applied
- **WHEN** user enters "1000000" in minimum volume field and clicks "開始本地篩選"
- **THEN** results SHALL only include securities with latest volume >= 1,000,000 shares

#### Scenario: Volume filter disabled (zero)
- **WHEN** user enters "0" in minimum volume field
- **THEN** results SHALL NOT filter by volume

#### Scenario: Volume filter combined with type filter
- **WHEN** user checks "股票" and enters "500000" in volume
- **THEN** results SHALL include only stock-type securities with volume >= 500,000
