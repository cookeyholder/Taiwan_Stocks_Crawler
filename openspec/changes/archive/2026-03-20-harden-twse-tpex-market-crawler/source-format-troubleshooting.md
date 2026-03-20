# 新來源格式變動排查流程

## 1. 先確認錯誤型態

- Header 找不到：`TWSE daily close section/header not found` 或 `TPEX header not found`
- 欄位不足：`Expected >=16 columns`（TWSE）或 `Expected >=17 columns`（TPEX）
- 編碼問題：解碼後亂碼或 parser 無法命中 header

## 2. 排查步驟

1. 先保留原始檔案，不做人工編輯。
2. 用現行 parser 跑一次，確認錯誤列號與原因。
3. 檢查標題列是否改名、換序或新增前置說明列。
4. 檢查是否由 cp950/big5hkscs 改為其他編碼。
5. 以最小改動更新 parser header matching 規則。
6. 新增 regression 測試覆蓋此次變動（含異常列）。
7. 重跑 `pytest tests`，確認既有樣本不回歸。

## 3. 快速判斷指引

- 只有前置說明增加：優先調整 section/header 搜尋，不動核心欄位對映。
- 欄位名稱微調：在 header normalize 階段做最小容忍（去空白、同義映射）。
- 欄位數改動：先更新對映設計，再調整 normalize 與 repository，補 migration 測試。

## 4. 回報內容模板

- 來源：TWSE/TPEX
- 發生日期：YYYY-MM-DD
- 錯誤摘要：header not found / column mismatch / decode issue
- 影響範圍：受影響筆數與市場
- 修正 PR：連結與新增測試清單
