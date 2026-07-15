## Context

布林通道選股目前每次按「開始篩選」都完整執行 `BacktestEngine::run_date()`，從 SQLite 讀取大量 OHLCV → 迭代全部股號計算布林通道 → 回傳結果。當使用者想比較不同參數組合（例如不同帶寬或不同日期），相同條件的結果不會被保留，必須重新等待運算。

現有架構：
- 前端 `ScreenerView.vue` → Pinia store `runScreen()` → `invoke("screen_bollinger")`
- 後端 `screen_bollinger` → `Pipeline::screen()` → `BacktestEngine::run_date()`
- 資料庫位於 `~/.local/share/stock-screener/market_data.db`（WAL mode，SQLite）

## Goals / Non-Goals

**Goals:**
- 快取篩選結果，相同參數直接回傳不重算
- 快取鍵包含演算法版本，版本更新時自動失效
- 提供手動清除快取的按鈕
- 空結果也快取，避免重複查無資料的日期

**Non-Goals:**
- 新資料入庫後自動失效（資料只往前增加，舊結果不受影響）
- 跨 session 的快取共享（單機單使用者，不需要）
- 快取大小的自動清理（預期 < 100 筆，每筆約 16KB）

## Decisions

### 1. 快取存放在後端 SQLite，非前端記憶體

| 選項 | 結論 |
|------|------|
| 前端 Pinia Map + localStorage | 捨棄。前端無持久化邏輯，且每次 Tauri 啟動後 localStorage 不會自動回復到 Pinia state |
| **後端 SQLite `screener_cache` 表** | **採用**。與現有 DB 共用連線，migration 機制一致。持久化跨 session 有效 |

後端連線已使用 `PRAGMA journal_mode=WAL`，多連線讀寫無衝突。

### 2. 單一 command 取代查詢→計算→回存三趟

設計成 `screen_bollinger_with_cache` 一個 command：
1. 用參數組合出 cache key
2. SELECT `screener_cache` → hit 則直接回傳
3. miss 則執行 `BacktestEngine::run_date()`
4. INSERT OR REPLACE 存入快取
5. 回傳結果

前端只需呼叫一次，miss 時 latency 與現狀相同。

### 3. Cache key 格式

```
{SCREEN_CACHE_VERSION}|{query_date}|{sorted_types}|{bandwidth}|{min_days}|{min_volume}
```

舉例：`v1|2026-07-14|etf,stock|0.0800|5|1000000`

- 版本號：`const SCREEN_CACHE_VERSION: &str = "v1"`，放在 Tauri command 旁，演算法改版時手動遞增
- `sorted_types`：前端傳來的 `instrument_types` 陣列排序後逗號合併，確保順序不影響 key
- 所有數值以 `format!` 固定格式序列化，避免小數點精確度差異

### 4. `screener_cache` 表結構

```sql
CREATE TABLE IF NOT EXISTS screener_cache (
    cache_key        TEXT NOT NULL PRIMARY KEY,
    query_date       TEXT NOT NULL,
    instrument_types TEXT NOT NULL,
    max_bandwidth    REAL NOT NULL,
    min_days         INTEGER NOT NULL,
    min_volume       INTEGER NOT NULL,
    results_json     TEXT NOT NULL,
    created_at       TEXT NOT NULL DEFAULT (datetime('now'))
);
```

`results_json` 以 `serde_json::to_string(&Vec<ScreenResult>)` 儲存，讀取時 `serde_json::from_str` 反序列化。

#### 4.1 反序列化失敗處理

若 `results_json` 內容損毀（例如磁碟寫入錯誤、手動修改），`serde_json::from_str` 回傳 error。此情況 SHALL 視同 cache miss：**log warning** 後重新執行篩選計算，不中斷使用者流程，不 crash。

#### 4.2 `instrument_types` 欄位儲存格式

`instrument_types` TEXT 欄位以排序後的逗號合併字串儲存，與 cache key 中的 `sorted_types` 片段一致，方便除錯時直接比對。

### 5. `CacheParams` 結構體

在 `stock-core/src/models.rs` 新增一個輕量 struct，封裝寫入 `screener_cache` 表所需的五個參數：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheParams {
    pub query_date: String,
    pub instrument_types: String,    // 排序後逗號合併
    pub max_bandwidth: f64,
    pub min_days: i32,
    pub min_volume: i64,
}
```

### 6. 清除快取實作

單一 command `clear_screener_cache()` → `DELETE FROM screener_cache`。前端 params bar 新增一個按鈕觸發。

## Risks / Trade-offs

- **[Stale cache] → 版本號解決**：若演算法邏輯改變，手動遞增 `SCREEN_CACHE_VERSION` 即可讓所有舊 key 失效
- **[JSON 序列化成本] → 可忽略**：每筆約 16KB，序列化/反序列化成本遠低於一次篩選計算
- **[空結果被誤解] → 前端行為不變**：空陣列 `[]` 存入快取，前端 `lastEmptyDate` 邏輯不受影響
- **[Concurrent miss] → 可接受**：兩次相同參數的請求同時抵達時，雙方都 miss cache、各自計算、各自 INSERT OR REPLACE。不影響正確性，僅浪費一次計算。若需解決可在計算前加 distributed lock，但單機場景下不必要。
- **[清除快取按鈕 loading 遺漏] → 前端補上**：清除快取按鈕需比照「開始篩選」加入 `clearing` loading state，清除期間按鈕 disabled 顯示「清除中...」，清除失敗時顯示錯誤訊息。
