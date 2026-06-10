# twse-realtime-quote Specification

## Purpose
TBD - created by archiving change migrate-to-tauri-rust. Update Purpose after archive.
## Requirements
### Requirement: 即時股價輪詢
盤中交易時段（週一～週五 09:00–13:30），每 10 秒從 TWSE 基本市況報導 API 抓取即時報價。

#### Scenario: 盤中自動輪詢
- **GIVEN** 使用者在分析頁面選定一支股票
- **WHEN** 目前時間為 09:00–13:30 的交易日
- **THEN** 每 10 秒自動呼叫 `fetch_realtime_quote` Tauri command
- **AND** 畫面顯示倒數計時（10→0）

#### Scenario: 非交易時段
- **GIVEN** 使用者停留在分析頁面
- **WHEN** 目前時間為非交易時段（週末、假日、13:30–09:00）
- **THEN** 顯示「非交易時間」文字
- **AND** 不發送輪詢請求

#### Scenario: 輪詢失敗 backoff
- **GIVEN** TWSE API 回應失敗
- **WHEN** 輪詢請求失敗
- **THEN** 每次失敗增加 10 秒間隔（10s → 20s → 30s → …）
- **AND** 上限 600 秒（10 分鐘）
- **AND** 成功後恢復 10 秒間隔

### Requirement: 即時報價顯示
畫面需顯示即時報價面板與 K 線今日 bar 同步更新。

#### Scenario: 即時報價面板
- **GIVEN** 輪詢成功取得 RealTimeQuote
- **THEN** 面板顯示：最新價、漲跌、漲幅、今高、今低、累積成交量
- **AND** 買賣五檔價量（如有資料）

#### Scenario: K 線今日 bar 即時更新
- **GIVEN** 輪詢成功取得 RealTimeQuote
- **WHEN** 價格變動
- **THEN** 圖表最後一根 K 線的 high/low/close 隨即時資料更新
- **AND** open 保持今日開盤價不變
- **AND** 成交量欄位同步更新

#### Scenario: ECharts 動畫
- **GIVEN** K 線資料更新
- **WHEN** setOption 觸發
- **THEN** animationDurationUpdate = 100ms
- **AND** animationEasingUpdate = 'linear'
- **AND** 四個圖格（價格/量/RSI/MACD）同步更新

### Requirement: 時鐘元件
畫面右上角顯示即時時間。

#### Scenario: 時鐘顯示
- **GIVEN** 應用程式運行中
- **WHEN** 任何時刻
- **THEN** 顯示 `HH:MM:SS` 格式（雙位數字）
- **AND** 每秒更新一次

### Requirement: dataZoom 策略
圖表縮放支援自動追尾與保持縮放兩種模式。

#### Scenario: 自動追尾（預設）
- **GIVEN** 使用者剛進圖表或剛換股
- **WHEN** 新 K 線資料進來
- **THEN** dataZoom 自動推進，最後一根永遠在視野內
- **AND** 顯示最近 60 根

#### Scenario: 使用者手動縮放
- **GIVEN** 使用者拖動 dataZoom 滑桿
- **WHEN** 新 K 線資料進來
- **THEN** zoom 位置固定，不自動滑動
- **AND** 使用者滑到最右邊（end=100%）時恢復自動追尾

### Requirement: 收盤後 EOD 補 DB
收盤後自動嘗試補今日收盤資料到資料庫。

#### Scenario: 自動補資料
- **GIVEN** 時間超過 13:30
- **WHEN** 自動停止即時輪詢
- **THEN** 每 60 秒嘗試 TWSE STOCK_DAY_ALL OpenAPI
- **AND** 取得資料後 upsert 到 SQLite
- **AND** 成功後顯示「今日已收盤，資料已更新」

#### Scenario: 手動下載按鈕
- **GIVEN** 分析頁面
- **WHEN** 使用者點擊「下載今日資料」按鈕
- **THEN** 呼叫 Tauri 的 EOD 下載指令
- **AND** 若資料已存在則 toast 提示「今日資料已存在」
- **AND** 若下載成功則 toast 顯示筆數

