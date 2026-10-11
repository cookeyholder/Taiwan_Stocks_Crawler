## 1. 資料匯入與修補結果

- [x] 1.1 將 T86 擷取及寫入接入 TWSE 行情日期匯入，並新增來源警告保留測試
- [x] 1.2 修正缺漏日期診斷的現存平日計數，並擴充含週末資料的測試
- [x] 1.3 修正缺漏日期修補錯誤分類與回報，來源失敗回傳失敗摘要且休市仍可略過；UI 錯誤分支保留診斷結果

## 2. 策略與自選股

- [x] 2.1 將新高判斷改為嚴格突破，新增價格等於前期高點的無訊號測試
- [x] 2.2 序列化自選切換、依指定群組切換，並新增連續操作測試
- [x] 2.3 空清單群組查詢回傳預設群組，新增資料庫測試

## 3. 圖表互動與張數顯示

- [x] 3.1 修正游標位置更新與主圖離開時的懸停狀態
- [x] 3.2 修正 HUD 與法人副圖張數精度，新增正負不足一張資料的格式化測試

## 4. 文件與整合驗收

- [x] 4.1 完成 `docs/CODE_REVIEW_REPORT.md` 與 OpenSpec 變更文件，並通過 `openspec validate review-pr-38-42-fixes --strict`
- [x] 4.2 執行 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace` 與 `cargo build --workspace`
