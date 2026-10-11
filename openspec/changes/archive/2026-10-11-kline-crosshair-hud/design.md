## Context

目前 `stock-gpui` 之 `AnalysisView` 採用 `ChartViewport` 進行可見 K 棒範圍切片，並結合 `gpui_kit::component::chart` 與自訂 `canvas` 繪製 K 線、布林通道與均線。然而使用者在圖表上移動滑鼠時，缺乏互動回饋與歷史交易日數值檢索機制。

詳細背景與需求請參閱 `proposal.md` 與 `specs/gpui-stock-charts/spec.md`。

## Goals / Non-Goals

**Goals:**
- 在個股分析主圖支援流暢之滑鼠懸浮追蹤，十字游標虛線跟隨游標精確顯示。
- 支援 X 座標至 K 棒資料切片索引（`ohlcv[index]`）之精準換算，支援平移與縮放視窗。
- 於圖表頂部提供響應式 HUD 即時數值資訊列，懸停時即時顯示當日開高低收、漲跌、成交量與均線/布林/RSI/MACD 數值。
- 滑鼠移出（`on_mouse_leave`）時平滑復原顯示最新一筆交易日之概況。
- 遵循臺灣股市色彩慣例（上漲為紅、下跌為綠）。

**Non-Goals:**
- 自由手繪趨勢線、支撐壓力線或黃金分割率標註工具（留待後續專業畫線功能擴充）。
- 回測畫面（`BacktestView`）的十字游標（本次變更聚焦於個股分析 `AnalysisView`）。

## Decisions

### 1. 懸停狀態集中於 `AnalysisView` 管理
- **決策**：在 `AnalysisView` 新增 `hovered_candle_idx: Option<usize>`（指向 `data.ohlcv` 之絕對索引）與 `hover_pos: Option<Point<Pixels>>`。
- **理由**：使用絕對索引而非可見切片之相對索引，能保證在圖表重繪或視窗推移時索引始終精準對應特定的交易日。
- **替代方案**：僅存相對索引。但相對索引在縮放時容易發生偏移造成資料越界或錯位。

### 2. X 軸座標至 K 棒索引的命中計算
- **決策**：利用 `ScaleBand` 之 `band_width` 與 `step`，或以可見 K 棒數量與主圖可用寬度（`bounds.size.width`）計算命中欄位：
  `col_idx = ((mouse_x - origin_x) / band_step).clamp(0, visible_count - 1)`
  再對映至 `slice_range.start + col_idx`。
- **理由**：計算開銷極小（$O(1)$），能確保 60fps 平滑更新而不產生事件延遲。

### 3. 十字輔助線與 HUD 資訊列呈現
- **決策**：十字游標以半透明灰色/主題色虛線於 `canvas` 上層繪製；數值資訊則置於圖表上方專屬 HUD 容器（而非僅用隨鼠浮動 Tooltip）。
- **理由**：金融軟體資訊密集，若僅以懸浮 Tooltip 呈現，常會遮擋附近的 K 線與指標走勢；頂部固定 HUD 既清晰易讀，又不會干擾視覺焦點。

## Risks / Trade-offs

- **[Risk]** 滑鼠快速移動時引發頻繁重繪造成 UI 卡頓。
  - **Mitigation**：僅在命中之 K 棒索引變更時更新狀態；利用 GPUI 內建排版與 GPU 渲染快取，確保每秒 60 幀平滑反應。
- **[Risk]** 可見 K 棒為 0 或視窗寬度為 0 時的邊界計算。
  - **Mitigation**：加入防禦性邊界檢查，若無資料或索引超出範圍直接返回 `None`。
