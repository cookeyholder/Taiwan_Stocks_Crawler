# Tasks: 集中市場當日交易資料（券商買賣分點日報表）自動化抓取與入庫

## 1. 資料庫遷移與核心資料模型 (stock-core)

- [x] 1.1 於 `stock-core/src/db/migrations.rs` 新增 Migration V8（建立 `broker_trades` 資料表與 `(symbol_clean, trade_date)`、`(broker_id, trade_date)` 索引）
- [x] 1.2 於 `stock-core/src/models.rs` 新增 `BrokerTradeRecord` 與 `BrokerSummary` 資料結構
- [x] 1.3 於 `stock-core/src/db/mod.rs` 實作 `upsert_broker_trades_batch` 批次寫入方法（確保衝突更新冪等性）
- [x] 1.4 於 `stock-core/src/db/mod.rs` 實作 `get_broker_trades`（查詢特定個股與日期之分點明細）與 `get_broker_completed_symbols`（查詢當日已入庫之股票代號清單，供斷點續傳使用）
- [x] 1.5 撰寫資料庫模組單元測試：驗證 Migration V8 結構、重複寫入冪等性與依日期/個股查詢效能

## 2. 雙欄 CSV 報表剖析器 (stock-core::fetcher::bsr::parser)

- [x] 2.1 於 `stock-core/src/fetcher/bsr/parser.rs` 實作 `parse_bsr_csv` 雙欄 CSV 解析函式
- [x] 2.2 實作檔頭略過邏輯（忽略前 3 列說明與表頭）並提取第 2 列之股票代碼（清洗 `="2330"` 前綴）
- [x] 2.3 實作左右對稱雙欄資料列切分（左側 5 欄與右側 5 欄獨立處理，忽略無效空列）
- [x] 2.4 實作券商字串清洗：切分前 4 碼 `broker_id` 與後續 `broker_name`（過濾半形與全形空白），並轉換價格與股數數值
- [x] 2.5 撰寫解析器單元測試：使用真實樣例檔案 `docs/集中市場當日交易資料_2026-10-08_2330.csv` 驗證完整 4,431 筆記錄剖析正確性

## 3. 圖形驗證碼辨識模組 (stock-core::fetcher::bsr::captcha)

- [x] 3.1 於 `stock-core/src/fetcher/bsr/captcha.rs` 實作影像前處理模組（灰階轉換、亮度門檻二值化、去除雪花孤立雜訊與單條干擾線）
- [x] 3.2 實作 5 碼字符分割與特徵識別器（`CaptchaSolver`），支援解析大寫英數字元
- [x] 3.3 撰寫驗證碼辨識單元測試：針對多組真實下載之 BSR 驗證碼樣例圖片驗證去噪與字符提取準確度

## 4. BSR 網路客戶端與連線管理 (stock-core::fetcher::bsr::client)

- [ ] 4.1 於 `stock-core/src/fetcher/bsr/client.rs` 實作 `BsrClient`，維護連線 Session、Cookie（`ASP.NET_SessionId`）與自訂標頭（User-Agent, Referer）
- [ ] 4.2 實作初次 GET `bsMenu.aspx` 取得表單狀態參數（`__VIEWSTATE`, `__EVENTVALIDATION` 等）與最新 Captcha GUID
- [ ] 4.3 實作下載驗證碼圖片並調用 `CaptchaSolver` 自動辨識
- [ ] 4.4 實作 POST `bsMenu.aspx` 提交股票代號與驗證碼，並判斷回傳內容是否成功產生報表
- [ ] 4.5 實作 GET `bsContent.aspx` 串流下載雙欄 CSV 內容
- [ ] 4.6 實作驗證碼失敗自動換碼重試邏輯（上限 3 次重試，超過記錄失敗）

## 5. 普通股範圍過濾與批次下載流水線 (stock-core::pipeline::broker)

- [ ] 5.1 於 `stock-core/src/pipeline/mod.rs` 實作 `get_twse_stock_symbols`：僅提取 TWSE 上市普通股（4 碼純數字，嚴格排除 ETF 與權證）
- [ ] 5.2 於 `stock-core/src/pipeline/broker.rs` 實作 `BrokerFetchPipeline`，整合 `BsrClient`、資料庫倉儲與取消旗標（`AtomicBool`）
- [ ] 5.3 實作斷點續傳檢核：比對資料庫當日已完成股票名單，自動跳過已下載標的
- [ ] 5.4 實作智慧節流：每檔股票抓取間隔加入 1.5 ~ 3.0 秒隨機延遲（Jitter）
- [ ] 5.5 實作異常連線指數退避：遭遇網路逾時或伺服器異常時自動冷卻暫停 30 秒再重試
- [ ] 5.6 實作進度通知回呼（包含已完成筆數、總檔數、目前股票代號、成功/失敗數）
- [ ] 5.7 撰寫批次流水線整合測試（以 Mock 服務驗證斷點續傳、隨機延遲與安全取消機制）

## 6. CLI 命令列工具子命令 (stock-cli)

- [ ] 6.1 於 `stock-cli/src/main.rs` 新增 `fetch-broker` 子指令與參數定義（`--date <YYYY-MM-DD>`, `--symbol <SYMBOL>`, `--delay-min <SECS>`, `--delay-max <SECS>`）
- [ ] 6.2 於 CLI 實作即時進度條（Progress Bar）、當前處理狀態與批次完成摘要輸出
- [ ] 6.3 支援 Ctrl+C 優雅中斷並保存當前進度狀態以利續傳

## 7. 桌面端介面整合 (stock-gpui)

- [ ] 7.1 於 `stock-gpui/src/state.rs` 擴充 `CoreState::fetch_broker_trades` 批次下載非同步任務方法
- [ ] 7.2 於 `stock-gpui/src/views/download.rs` 新增「集中市場券商買賣日報表」下載專屬卡片面板
- [ ] 7.3 在介面提供目標交易日選取器、普通股總檔數統計與「開始下載 / 繼續下載」按鈕
- [ ] 7.4 實作即時下載進度條、目前抓取個股名稱代號、預估剩餘時間與取消控制項
- [ ] 7.5 撰寫無頭 UI 渲染測試，驗證下載面板元件與狀態連動正常

## 8. 驗證與品質檢核

- [ ] 8.1 執行全工作區單元與整合測試（`cargo test --workspace`）確保所有測試通過
- [ ] 8.2 執行靜態程式碼分析（`cargo clippy --workspace --all-targets -- -D warnings`）確保零警告
- [ ] 8.3 執行排版檢查（`cargo fmt --all -- --check`）確保程式碼格式符合規範
- [ ] 8.4 驗證 OpenSpec 規範檔案（`openspec validate broker-trades`）無誤
