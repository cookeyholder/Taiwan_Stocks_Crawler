# OpenSpec Review-Fix Plan

## Change
- Name: `review-pr-38-42-fixes`
- Branch: `review-fix/pr-38-42`
- Scope: PR #38–#42 的十項審查發現，集中於一個修正變更。
- Tracking issue: [#43](https://github.com/cookeyholder/Taiwan_Stocks_Crawler/issues/43)

## Commands
```bash
openspec new change "review-pr-38-42-fixes"
openspec instructions proposal --change "review-pr-38-42-fixes"
openspec instructions design --change "review-pr-38-42-fixes"
openspec instructions tasks --change "review-pr-38-42-fixes"
```

## Completion Gate
- [x] `docs/CODE_REVIEW_REPORT.md` 已列出 P1-P3 發現、影響情境與驗收條件。
- [x] OpenSpec delta 規格與實作任務對應全部修正項目。
- [x] 已執行 `openspec validate review-pr-38-42-fixes --strict`。
- [ ] 工作區測試、建置、Clippy 與格式檢查全數通過。
- [x] 已建立 [PR #44](https://github.com/cookeyholder/Taiwan_Stocks_Crawler/pull/44)，並關聯 Issue #43。

## PR #44、#45 審查修正

- Change: `review-pr-44-45-fixes`
- Branch: `review-fix/pr-44-45`
- Scope: BSR 日期/來源驗證、CSV 完整性、TWSE 名單、零明細續傳與 CLI 批次退出狀態。
- Tracking issue: [#46](https://github.com/cookeyholder/Taiwan_Stocks_Crawler/issues/46)
- 狀態：實作、工作區測試、建置、Clippy、格式檢查、OpenSpec 驗證與第二輪審查已完成；PR 建立中。
