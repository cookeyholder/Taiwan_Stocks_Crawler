## ADDED Requirements

### Requirement: 問題排查文件

`operations/troubleshooting.md` 必須涵蓋使用此軟體時可能遇到的常見問題與解決方法。

#### Scenario: 涵蓋市場休市判斷
- **WHEN** 維運者遇到資料抓取失敗
- **THEN** 文件說明如何判斷是市場休市（API 回傳日期 ≠ 請求日期）還是真正的資料來源錯誤

#### Scenario: 涵蓋 API 格式變動
- **WHEN** TWSE 或 TPEX 變更 API 格式導致抓取失敗
- **THEN** 文件提供排查步驟：檢查 raw response、比對預期格式、更新 parser

#### Scenario: 涵蓋資料庫問題
- **WHEN** 維運者遇到資料庫相關錯誤
- **THEN** 文件說明如何檢查資料庫完整性、重建索引、或從備份還原

#### Scenario: 涵蓋連線問題
- **WHEN** 維運者遇到 HTTP 連線錯誤
- **THEN** 文件說明 retry/backoff 機制的行為，以及如何調整逾時設定

### Requirement: 自動化排程文件

`operations/automation.md` 必須說明如何設定定時任務以自動化資料抓取流程。

#### Scenario: 涵蓋 cron 排程
- **WHEN** 維運者想要設定定時抓取
- **THEN** 文件提供 cron 表示式範例（例如每個交易日收盤後執行）

#### Scenario: 涵蓋批次補檔
- **WHEN** 維運者需要補抓歷史資料
- **THEN** 文件說明如何使用 `sync` 指令批次補檔，以及建議的批次大小

#### Scenario: 涵蓋 Windows 排程
- **WHEN** 維運者使用 Windows 系統
- **THEN** 文件提供工作排程器的設定指引（如適用）

#### Scenario: 涵蓋資料庫維護
- **WHEN** 維運者需要維護資料庫
- **THEN** 文件說明資料庫備份方式與建議的維護排程
