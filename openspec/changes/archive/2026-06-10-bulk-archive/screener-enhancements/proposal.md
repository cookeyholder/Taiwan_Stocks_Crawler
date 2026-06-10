## Why

布林擠壓選股器目前對所有證券一視同仁，但權證（11K 檔）和債券的價格行為與股票截然不同，會淹沒真正值得關注的股票/ETF 訊號。同時，目前分類邏輯有缺口，導致 83% 的證券被歸入「other」黑盒子，使用者無法區分 ETF、可轉債與其他商品。

## What Changes

- **分類補強**：修正 `classification.py` 的編碼規則，涵蓋 6 碼純數字 ETF、5 碼+字母槓桿/反向/主動型 ETF、可轉債等，並對既有資料庫執行一次性重新分類
- **證券類型篩選**：在 Tab 2 加入 checkboxes，可勾選要顯示的證券類型（股票固定顯示），過濾條件傳入 `screen_bollinger_squeeze()`
- **最低成交量篩選**：新增最低成交量門檻輸入欄，低於門檻者不顯示
- **欄位排序**：點擊 Treeview 欄位標題可切換遞增/遞減排序
- **K 線圖取代折線圖**：右側圖表以 mplfinance 繪製 K 線圖疊加布林通道，成交量柱依漲跌著色

## Capabilities

### New Capabilities
- `screener-type-filter`: 依 instrument_type 篩選證券類型，含分類補強
- `screener-volume-filter`: 依最低成交量門檻篩選
- `screener-chart-kline`: 右側圖表改為 K 線圖疊加布林通道

### Modified Capabilities
- `screener-sorting`: 現有 Treeview 結果表格加入欄位點擊排序

## Impact

- `market_crawler/classification.py`: 補強分類規則
- `market_crawler/screener.py`: 新增 instrument_type 與 min_volume 參數
- `market_crawler/gui.py`: Tab 2 新增篩選 UI、K 線圖
- `pyproject.toml`: 加入 `mplfinance` 依賴
- `market_data.db`: 一次性重新分類既有資料
