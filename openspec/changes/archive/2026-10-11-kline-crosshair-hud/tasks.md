## 1. 座標命中計算與控制器單元測試

- [x] 1.1 在 `stock-gpui/src/views/chart_viewport.rs` 實作滑鼠 X 座標換算 K 棒切片索引函式 `candle_index_at_x`，並於 `chart_test.rs` 撰寫單元測試驗證
- [x] 1.2 執行 `cargo test -p stock-gpui --test chart_test` 驗證邊界、平移與縮放情境之命中索引計算正確

## 2. 滑鼠事件監聽與十字游標畫布繪製

- [x] 2.1 於 `stock-gpui/src/views/analysis.rs` 之 `AnalysisView` 加入 `hovered_candle_idx` 與 `hover_pos` 狀態，並於主圖容器綁定 `on_mouse_move` 與 `on_mouse_leave` 事件
- [x] 2.2 在主圖及副圖畫布層實作十字游標時間軸垂直虛線與價格軸水平虛線之繪製，並確保在滑鼠移出時隱藏

## 3. 頂部 HUD 即時數值資訊列

- [x] 3.1 在 `AnalysisView` 圖表頂部實作 HUD 資訊列，動態呈現懸停 K 棒之日期、開盤、最高、最低、收盤、漲跌金額與漲跌幅、成交量（張）及技術指標數值（MA/布林/RSI/MACD）
- [x] 3.2 確保在滑鼠移出畫布時平滑還原顯示最新一筆收盤交易日之行情與指標數據

## 4. 品質檢查與無頭測試驗證

- [x] 4.1 擴充無頭測試驗證十字游標與 HUD 狀態更新邏輯，執行 `cargo test --workspace` 驗證所有測試全數通過
- [x] 4.2 執行 `cargo clippy --workspace` 與 `cargo fmt --all -- --check` 驗證程式碼品質無誤
