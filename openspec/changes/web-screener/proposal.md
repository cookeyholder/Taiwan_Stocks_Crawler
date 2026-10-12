## Why

個股行情網站只能逐檔檢視，尚未涵蓋既有桌面端的策略篩選與回測工作流程。此階段將現有 Rust 策略引擎提供給網站使用，讓同一套核心結果可在瀏覽器篩選、解讀及比較。

## What Changes

- 提供網站選股，涵蓋 `StrategyKind` registry 的布林擠壓、均線交叉、RSI、均線排列、波段新高，以及既有桌面端「均線排列 AND 波段新高」複合模式。
- 建立選股條件表單、結果表格、策略理由標籤與連結至個股分析頁的介面。
- 提供歷史回測 API 與結果頁，使用既有訊號引擎、投資組合模擬器及績效計算器。
- 建立選股/回測共用且持久化的長時間工作管理器，供第四階段擴充資料寫入工作。
- 沿用階段一 API 契約及階段二的證券搜尋和圖表呈現。

## Capabilities

### New Capabilities
- `web-screener`: 提供策略參數化篩選及可解讀的結果檢視。
- `web-backtest`: 提供網站回測操作與結果呈現。
- `web-background-jobs`: 提供長時間網站工作的建立、查詢、取消與重啟狀態管理。

### Modified Capabilities

## Impact

- 擴充 `stock-web` 選股與回測 API，呼叫 `stock-core` 的 pipeline/backtest 模組。
- 前端新增選股及回測頁；不另建策略算法，僅將既有複合選股邏輯移入共用核心。
- 前置變更：`web-platform-foundation`、`web-market-analysis`。第四階段會擴充本階段的工作管理器，不是本階段實作的前置條件。
