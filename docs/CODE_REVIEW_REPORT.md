# PR #38–#42 程式碼審查報告

## 審查範圍

審查已合併的 PR #38、#39、#40、#41、#42 之差異，以及目前 `main` 分支整合後的程式與相關 OpenSpec 規格。審查面向包含邏輯正確性、資料完整性、效能、介面互動與測試涵蓋。

## 發現事項

### P1_HIGH

1. **缺漏日抓取失敗仍回報修補完成** — `stock-gpui/src/state.rs:794-850` 的 `repair_gaps` 忽略 `fetch_date` 回傳的 `outcome.ok` 與錯誤訊息；網路或交易所錯誤會被當成休市略過，最後仍回報成功，介面也會清除診斷結果。應區分休市、抓取失敗與成功入庫，並保留仍未修補的日期。
2. **三大法人資料抓取未接入產品流程** — `stock-core/src/pipeline/mod.rs:96-101` 新增接線前，`fetch_institutional_twse` 與資料庫寫入方法只有定義及測試呼叫，沒有日常行情抓取流程呼叫。個股分析只讀取資料庫，因此實際使用者無法透過應用程式取得法人資料。應將法人日報抓取及入庫接入日期抓取流程，並維持行情抓取在法人來源失敗時可用。

### P2_MEDIUM

3. **同一根 K 棒內移動游標不會更新水平線** — `stock-gpui/src/views/analysis.rs:1869` 更新 `hover_pos`，但原先只在 K 棒索引改變時通知重繪；游標在同一根 K 棒內上下移動時水平線停留在舊位置。
4. **離開主圖後 HUD 可能保留舊日期** — `stock-gpui/src/views/analysis.rs:1884` 原先沒有離開事件清除懸停狀態；游標離開容器後 HUD 可能繼續顯示舊 K 棒資料。
5. **快速連續切換自選股可能只執行加入** — `stock-gpui/src/state.rs:176-185` 原先在取得資料庫鎖前讀取快取；兩次併發切換可同時讀到未收藏，之後兩次都執行冪等加入，無法達成切換兩次後回到原狀。
6. **依群組切換自選股會誤判收藏狀態** — `stock-gpui/src/state.rs:176-185` 原先以全域快取判斷是否收藏；股票已在群組 A 時，對群組 B 執行切換會嘗試移除 B 的紀錄而非加入 B，導致群組操作失效。
7. **與既有高點相同也會觸發新高訊號** — `stock-core/src/backtest/new_high.rs:90` 原先使用 `curr_val >= prior_max`，使當日價格等於回溯區間最高價時仍被列為突破，與規格「高於／超越」不符。
8. **三大法人張數顯示截斷零股** — `stock-gpui/src/views/analysis.rs:953,1004,1731-1735` 原先以整數除以 1000 轉換股數，未滿千股的買賣超會顯示為 0 張；例如 999 股的正負方向都可能失去可見差異。應保留小數張精度。

### P3_LOW

9. **缺漏診斷的現存日數會計入週末** — `stock-core/src/db/mod.rs:351-394` 原先讓 `recorded_dates` 計入週末資料，但 `total_workdays` 排除週末，會出現現存日數大於平日總數的矛盾。
10. **空自選清單查不到預設群組** — `stock-core/src/db/mod.rs:589-601` 原先在尚無收藏紀錄時回傳空清單，與現有規格要求的「預設自選」基礎群組不符。

## 修正範圍與驗收

- 修補流程回報各日期的抓取失敗，成功訊息不得掩蓋錯誤，且失敗日期仍留在診斷結果中。
- 行情日期抓取完成後擷取並保存 TWSE T86 資料；法人 API 失敗不得使行情匯入失敗。
- 游標位置變更與離開主圖皆更新十字線/HUD 狀態。
- 自選股切換具序列化語意；空清單群組查詢回傳「預設自選」。
- 突破策略只在嚴格高於既有高點時發出訊號。
- 法人買賣超以至少兩位小數顯示張數，圖表尺度保留股數精度。
- 缺漏診斷現存日數只計入平日。
- 對以上行為新增或調整回歸測試，並通過工作區測試、建置、Clippy 與格式檢查。

## 待確認與限制

- 缺漏診斷目前依平日推估可能缺漏日，未使用交易所休市日曆；休市日仍可能列入待檢查清單，但修補流程會將交易所無資料與來源錯誤分開處理。
- 三大法人來源為 TWSE T86；TPEX 標的是否有對應法人資料來源不在本輪修正範圍。

## 相關 PR

- [PR #38](https://github.com/cookeyholder/Taiwan_Stocks_Crawler/pull/38)
- [PR #39](https://github.com/cookeyholder/Taiwan_Stocks_Crawler/pull/39)
- [PR #40](https://github.com/cookeyholder/Taiwan_Stocks_Crawler/pull/40)
- [PR #41](https://github.com/cookeyholder/Taiwan_Stocks_Crawler/pull/41)
- [PR #42](https://github.com/cookeyholder/Taiwan_Stocks_Crawler/pull/42)

## 修正追蹤

- [Issue #43](https://github.com/cookeyholder/Taiwan_Stocks_Crawler/issues/43)
