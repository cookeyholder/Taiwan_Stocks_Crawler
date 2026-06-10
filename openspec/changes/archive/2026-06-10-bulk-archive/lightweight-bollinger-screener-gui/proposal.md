## Why

使用者需要一個具備輕量化（極簡依賴）且可完全離線執行的台股技術分析選股篩選器與圖形視覺化工具，以透過「布林通道擠壓（Bollinger Band Squeeze）」策略，篩選出低波動盤整且即將變盤的股票，並支援使用 `yfinance` 進行一鍵本地歷史資料回補。

## What Changes

- **新增選用依賴**：在 `pyproject.toml` 中新增 `viz` 選用相依套件組，包含 `matplotlib`、`pandas-ta` 與 `yfinance`，避免肥大化原生 CLI 工具的安裝體積。
- **建立 Tkinter GUI 介面**：新增 `market_crawler/gui.py` 模組，使用 Python 內建的 Tkinter 與嵌入式 Matplotlib，提供以下功能分頁：
  - **單股分析分頁**：輸入股票代號，繪製股價折線、20 MA 中軌以及布林通道上下軌（附半透明陰影填滿區間）。
  - **布林擠壓篩選分頁**：設定帶寬上限門檻值（例如 0.08）與持續天數 N（例如 5 天），一鍵篩選本地 SQLite 資料庫中符合條件的股票，點選列表可即時於圖表更新展示。
- **新增 yfinance 補檔模組**：新增 `market_crawler/yfinance_sync.py`，用來批次將台灣 50 等標的之歷史 K 線資料透過 yfinance 下載，並以 Batch 方式安全 Upsert 寫入本地 SQLite，以滿足選股所需的歷史數據基底。
- **命令列整合**：於 `Makefile` 新增 `make gui` 指令以方便啟動。

## Capabilities

### New Capabilities
- `stock-visualization`: 提供基於 Tkinter + Matplotlib 的個股 K 線與布林通道圖表繪製。
- `bollinger-screener`: 提供基於本地 SQLite 歷史數據的布林擠壓條件過濾與選股功能。

### Modified Capabilities
- 無。

## Impact

- **新增檔案**：
  - `market_crawler/gui.py` (GUI 介面實作)
  - `market_crawler/yfinance_sync.py` (yfinance 本地回補模組)
- **修改檔案**：
  - `pyproject.toml` (新增 optional dependencies viz 依賴)
  - `Makefile` (新增啟動 GUI 指令)
- **相容性**：不影響現有的 `twse-crawler` 命令列功能、休市分類邏輯與資料庫 Schema，完全向下相容。
