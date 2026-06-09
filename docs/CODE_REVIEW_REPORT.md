# 臺灣證券行情爬蟲專案資安與效能審查報告

本報告針對專案中的核心代碼（特別是 `market_crawler/` 資料夾）進行了全面的資訊安全與執行效能審查。報告中的分級說明如下：
- `P0_CRITICAL`：致命風險，可能導致系統崩潰或嚴重的寫入效能黑洞。
- `P1_HIGH`：嚴重風險，包含可能被攻擊的安全缺陷或顯著的 I/O 效能耗損。
- `P2_MEDIUM`：中等風險，影響隱私、代碼重用性或局部效能。
- `P3_LOW`：輕微風險，代碼健壯性或配置優化建議。

---

## 1. 審查發現與風險評估

### 🔴 P0_CRITICAL：SQLite 逐筆 Insert/Upsert 效能瓶頸（SQL N+1 寫入問題）
*   **檔案與行號**：
    *   [market_crawler/pipeline.py:L164-L165](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/pipeline.py#L164-L165)
    *   [market_crawler/repository.py:L109](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/repository.py#L109) (`upsert_core`) 與 [L164](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/repository.py#L164) (`upsert_extension`)
*   **風險說明**：
    當日行情資料通常包含數千筆交易紀錄。在 `_ingest_records` 的 `for` 迴圈中，每一筆 record 都會呼叫一次 `repo.upsert_core` 與 `repo.upsert_extension`。這兩個方法在內部皆是直接調用 `self.connection.execute(...)`。
    雖然整個過程被包裹在一個 `with conn:` 交易（Transaction）區塊內，但逐筆呼叫 `execute` 會導致 Python 與 SQLite 之間產生數千次的跨語言/C API 呼叫，且每次都需要在 runtime 解析 SQL 語意與綁定參數，這在大資料量寫入時是極為嚴重的 CPU 效能瓶頸。

---

### 🔴 P1_HIGH：停用 SSL/TLS 憑證驗證與抑制安全警告（中間人攻擊風險）
*   **檔案與行號**：
    *   [market_crawler/pipeline.py:L48](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/pipeline.py#L48) `urllib3.disable_warnings(...)`
    *   [market_crawler/pipeline.py:L320](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/pipeline.py#L320) `ssl_verify=False`
    *   [market_crawler/pipeline.py:L412](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/pipeline.py#L412) `ssl_verify=False`
    *   [market_crawler/pipeline.py:L589](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/pipeline.py#L589) `ssl_verify=False`
*   **風險說明**：
    在 `pipeline.py` 中，預設的 HTTP 請求策略硬編碼將 `ssl_verify` 設定為 `False`，並且在檔案開頭抑制了 `InsecureRequestWarning` 安全警告。這意味著爬蟲在與台灣證券交易所（TWSE）及櫃買中心（TPEX）進行連線時，完全不驗證其 SSL/TLS 憑證。
    此設定極易受到**中間人攻擊 (Man-in-the-Middle, MITM)**。攻擊者可透過 DNS 劫持、ARP 欺騙或假冒憑證來攔截並偽造傳回的 API 資料，進而寫入假交易資訊到資料庫中，甚至可藉此注入特製的惡意 Payload 以利用後續解析邏輯的安全漏洞。

### 🔴 P1_HIGH：重複載入 Security Master 主檔造成大量 I/O 與 CPU 浪費
*   **檔案與行號**：
    *   [market_crawler/pipeline.py:L331](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/pipeline.py#L331)
*   **風險說明**：
    在進行多日批次補檔（如使用 `--backfill 30` 或指定一段日期區間）時，`orchestrator.py` 會逐日呼叫 `fetcher`。這導致每一個開市交易日都會重新開啟 `StockID.csv` 檔案，以 `csv.DictReader` 逐行解析，並在記憶體中建立數千筆 `SecurityMasterEntry`。如果執行 30 天的歷史補檔，就會重複進行 30 次完全相同的實體檔案 I/O 與 CSV 解析，對效能與硬碟壽命造成不必要的負擔。

### 🔴 P1_HIGH：頻繁建立與關閉 SQLite 連線
*   **檔案與行號**：
    *   [market_crawler/pipeline.py:L335](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/pipeline.py#L335) (`sqlite3.connect`) 與 [L376](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/pipeline.py#L376) (`conn.close`)
*   **風險說明**：
    `fetch_and_ingest_day` 將資料庫連線生命週期限制在單次呼叫內。在批次執行模式中，每一天都會重新建立一個全新的 SQLite 連線、開啟 Transaction、執行 `repo.init_schema()`，最後再關閉。這會造成頻繁的連線握手與資源釋放 overhead，在批次補檔時會顯著拉低執行速度。

### 🔴 P1_HIGH：未複用 HTTP Connection（連線池 Connection Pool 缺失）
*   **檔案與行號**：
    *   [market_crawler/http_client.py:L22](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/http_client.py#L22)
*   **風險說明**：
    `get_with_retry` 內部直接呼叫 `requests.get`，沒有使用 `requests.Session`。這代表每次發送 HTTP 請求（包括抓取 TWSE/TPEX 行情、年份國定假日清單等），都會重新建立一個全新的 TCP 連線並重新進行 TLS 握手。在 backfill 大量歷史資料時，網路建立連線的延遲（RTT）會大幅累加，且可能因為短時間內向交易所發起大量新 socket 連線，而被伺服器判定為惡意存取而阻擋 IP。

---

### 🟡 P2_MEDIUM：敏感資訊外洩風險（開發者隱私洩漏）
*   **檔案與行號**：
    *   [pyproject.toml:L5-L7](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/pyproject.toml#L5-L7)
*   **風險說明**：
    專案設定檔中直接公開了開發者的真實姓名與個人電子郵件信箱。如果本專案在公開託管平台（如 GitHub/GitLab）開源或分享，該信箱容易被自動化爬蟲搜集，增加開發者遭受垃圾郵件 (Spam) 與精準魚叉式網路釣魚攻擊的風險。

### 🟡 P2_MEDIUM：同一個 Quote Payload 被重複規整化（Normalize）兩次
*   **檔案與行號**：
    *   [market_crawler/repository.py:L110](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/repository.py#L110) 與 [L165](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/repository.py#L165)
*   **風險說明**：
    同一個 `quote` 資料在寫入核心表（`upsert_core`）與擴充表（`upsert_extension`）時，都會被重複執行一次 `_normalize_quote_payload`。此方法會遞迴走訪整個 dict/list並進行 UTF-8 編解碼處理，造成同一個 payload 被重複清洗兩次，浪費了無謂的 CPU 計算資源。

### 🟡 P2_MEDIUM：日期轉換重複解析 Trade Date
*   **檔案與行號**：
    *   [market_crawler/normalize.py:L133](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/normalize.py#L133)
*   **風險說明**：
    對於同一個交易日的所有 records（可能達數千筆），其 `trade_date` 都是同一個字串（例如 `"2026-03-17"`）。然而，`normalize_record` 對於每一筆 record 都會重新呼叫一次 `roc_to_ad_date`，這包含了 `datetime.strptime` 的解析與 `strftime` 的格式化。在 Python 中 `strptime` 是相當緩慢的操作，對同一個日期重複解析數千次是顯著的 CPU 資源浪費。

---

### 🟢 P3_LOW：`to_int` 轉型缺乏錯誤處理（局部拒絕服務 / 爬蟲崩潰風險）
*   **檔案與行號**：
    *   [market_crawler/normalize.py:L64-L66](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/normalize.py#L64-L66)
*   **風險說明**：
    在數值轉換邏輯中，`to_float` 函式使用了 `try-except ValueError` 進行異常防禦，但 `to_int` 函式僅單純執行 `return int(cleaned)`。若未來交易所 API 回傳了未預期的非數字字串（例如特殊除權息標記、"N/A" 或格式異動），`to_int` 將會拋出未捕獲的 `ValueError`，導致整個批次爬蟲 Pipeline 崩潰中斷，構成局部拒絕服務 (DoS) 狀態。

### 🟢 P3_LOW：專案中包含未經驗證的第三方二進位執行檔
*   **檔案與行號**：
    *   [sqlite3/](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/sqlite3/) 下的所有 `.exe` 與 `.dll` 檔案
*   **風險說明**：
    專案的 Git 版本控制中直接納入了 Windows 版本的 SQLite3 執行檔（`sqlite3.exe`、`sqldiff.exe`、`sqlite3.dll` 等）。直接簽入未經雜湊驗證與簽名查驗的二進位執行檔存在安全風險，可能導致惡意程式替換 (Binary Planting)，且增加不必要的程式庫體積。

### 🟢 P3_LOW：依賴套件未限制特定版本
*   **檔案與行號**：
    *   [pyproject.toml:L9-L14](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/pyproject.toml#L9-L14)
*   **風險說明**：
    專案在 `dependencies` 中使用了寬鬆的範圍界定（例如 `requests>=2.28.0`、`pandas>=1.5.0`），且缺乏釘選（Pin）具體版本之 Lock 檔。若依賴之第三方套件釋出含有安全漏洞 (CVE) 或重大破壞性變更的新版本，執行環境在重新安裝時可能會自動下載到受害版本。

### 🟢 P3_LOW：靜態 Regular Expression 未預先編譯（Pre-compiled）
*   **檔案與行號**：
    *   [market_crawler/classification.py:L63-L81](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/classification.py#L63-L81)
    *   [market_crawler/normalize.py:L48](file:///home/cookeyholder/Projects/Taiwan_Stocks_Crawler/market_crawler/normalize.py#L48)
*   **風險說明**：
    這些正則表達式（Regex Patterns）皆為完全靜態的條件。在模組層級先使用 `re.compile` 進行預編譯，能消除 runtime 的正則解析與檢索耗損。

---

## 2. 修復建議與設計方案

### 1️⃣ 重構寫入機制，引入批次寫入（Batch Ingestion）與預先規整化（P0_CRITICAL 改善）
*   **修復作法**：
    1. 在 `SQLiteMarketRepository` 中，新增批次寫入方法 `upsert_core_batch(quotes: list[dict[str, Any]])` 與 `upsert_extension_batch(quotes: list[dict[str, Any]])`，並在內部使用 `self.connection.executemany(...)`。
    2. 將 `_normalize_quote_payload` 移動到 Repository 外部，在 Pipeline 階段一次性將所有 quotes 進行清洗，避免 core 與 extension 重複規整化。
    3. 修改 `_ingest_records`，先將規整化後的 quotes 收集到 list 中，再呼叫 `executemany`。
*   **驗證方法**：
    *   對比優化前與優化後的寫入時間，預期效能可提升 3 至 5 倍。

### 2️⃣ 啟用 SSL/TLS 憑證驗證，並使用 HTTP Session 連線池（P1_HIGH 改善）
*   **修復作法**：
    1. 將 `market_crawler/pipeline.py` 中所有的 `ssl_verify=False` 修改為 `ssl_verify=True`，並移除開頭的 `urllib3.disable_warnings(...)`。
    2. 在 `http_client.py` 中重構 `get_with_retry`，使其支援傳入選用的 `requests.Session` 實例。
    3. 在 `pipeline.py` 的批次執行生命週期中，建立並共享一個 `requests.Session` 連線池。
*   **驗證方法**：
    *   驗證在連續抓取時能維持安全憑證驗證，且底層 socket 連線能正確複用。

### 3️⃣ 優化批次架構，共用 DB 連線與 Security Master 主檔（P1_HIGH 改善）
*   **修復作法**：
    1. 在 `pipeline.py#make_fetcher` 的閉包中載入一次 `security_master` 並建立共享的 `Connection`。
    2. 修改 `fetch_and_ingest_day`，使其可接受選用的 `conn` 與 `security_master`，若有傳入則直接複用，免除重複開啟連線與 CSV I/O 解析。
*   **驗證方法**：
    *   執行 `--backfill 30` 測試，預期總執行時間可大幅縮短。

---

## 3. 不確定假設與需要人工確認項目

1. **Root CA 憑證相容性**：原作者提及「政府機關網站的舊憑證有時不符合規範」因而停用 SSL。需確認實際執行環境上，若將 `ssl_verify` 設為 `True`，其內建的 `ca-certificates` 是否已能正常信任 TWSE/TPEX 憑證庫。
2. **Security Master CSV 動態異動頻率**：在執行批次補檔期間，`StockID.csv` 被假設為靜態不變的。需確認是否有本機/外部排程會在中途更新該檔案。
3. **SQLite 連線的 Thread-safety**：目前是單執行緒循序執行，因此複用 Connection 是安全的。若未來需要改為多執行緒併發，需改用 Connection Pool。
