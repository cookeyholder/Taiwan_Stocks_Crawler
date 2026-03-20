# 分類規則決策表與維護準則

## 決策優先序

1. manual override
2. section_label
3. cfi_code
4. ticker pattern（依《中華民國證券市場編碼原則》）
5. other

## 決策表

| 條件 | 輸出 instrument_type | 備註 |
|---|---|---|
| override 命中 | override 值 | 最高優先，人工修正 |
| section_label 含 ETF | etf | 例如 ETF/債券 ETF |
| section_label 含 權證 | warrant | 權證類別 |
| section_label 含 債 | bond | 債券類別 |
| cfi_code 首碼 E | stock | Equity |
| cfi_code 首碼 D | bond | Debt |
| cfi_code 首碼 R | warrant | Rights/Warrants |
| `\d{4}` | stock | 一般股票代號 |
| `\d{4}[A-Y]` | preferred_stock | 特別股 |
| `\d{4}Z[1-9]` | exchangeable_preferred_stock | 可交換特別股 |
| `\d{4}G[1-9]` | warrant_certificate | 認股權憑證 |
| `\d{2}\d{3}[PUTFQCBXY]` | warrant | 認購/認售權證 |
| `[BGFAHPI]\w{5}` | bond | 各類債券編碼前綴 |
| `T\w{5}` | fund | 開放式基金等 |
| 其他 | other | 保守回退 |

## 維護準則

- 新增規則必須附對應測試案例（正向 + 反向）。
- 規則調整時，需先確認不影響 override 與 section/cfi 優先序。
- ticker pattern 僅作 fallback，不可覆蓋已命中的 section/cfi。
- 若法規更新，先更新規則文件，再更新程式與測試。
