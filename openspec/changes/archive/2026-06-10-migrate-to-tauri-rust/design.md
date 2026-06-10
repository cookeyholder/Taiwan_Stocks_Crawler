## Context

目前的專案是 Python 3.13 + tkinter + matplotlib/mplfinance 的台股布林通道擠壓選股桌面應用。GUI 約 1407 行，後端模組涵蓋 SQLite CRUD、HTTP 爬蟲（TWSE/TPEX/yfinance）、技術分析計算、證券分類。決定以 Tauri 2.0 + Rust 後端 + Vue 3 前端全量改寫，保留既有 SQLite 資料格式。

## Goals / Non-Goals

**Goals:**
- 建立三層架構：Rust 核心庫（stock-core）→ Tauri IPC → Vue 3 前端
- stock-core 可獨立於 Tauri 使用（CLI、測試、library）
- 所有技術指標在 Rust 端計算，ECharts 只負責渲染
- SQLite 沿用既有 `market_data.db`，schema 相容
- 跨平台編譯：Windows (WebView2)、macOS (WKWebView)、Linux (WebKitGTK)
- Bundle 大小控制在 15MB 以下

**Non-Goals:**
- 不刪除或修改既有 Python 程式碼（移至 `legacy/` 目錄）
- 不支援 WebSocket 推播式即時報價（僅支援 HTTP polling 方式）
- 不支援多使用者或網路同步

## Decisions

### 1. 專案目錄結構：Python 移至 legacy，Tauri 在根目錄

```text
/                               ← repo 根目錄
├── legacy/                     ← Python 程式碼原樣移入
│   ├── market_crawler/
│   ├── build/
│   ├── tests/
│   ├── pyproject.toml
│   └── Makefile
│
├── stock-core/                 ← 純 Rust 庫，零 Tauri 依賴
│   ├── src/
│   │   ├── db/         ← rusqlite CRUD + migration
│   │   ├── fetcher/    ← reqwest + scraper + wiremock
│   │   ├── analysis/   ← 技術指標計算
│   │   └── pipeline/   ← 編排邏輯
│   └── Cargo.toml
│
├── stock-cli/                  ← CLI 二進位
│   ├── src/main.rs     ← clap 解析參數
│   └── Cargo.toml
│
├── stock-tauri/                ← Tauri 桌面 App
│   ├── src-tauri/
│   │   ├── src/commands/  ← 薄 IPC wrapper
│   │   ├── src/main.rs
│   │   ├── Cargo.toml
│   │   └── tauri.conf.json
│   ├── src/              ← Vue 3 前端
│   │   ├── main.ts
│   │   ├── App.vue
│   │   ├── components/
│   │   ├── stores/
│   │   └── types/
│   ├── package.json
│   └── vite.config.ts
│
├── Cargo.toml                  ← workspace root
├── .gitignore
└── README.md
```

**Rationale:**
- Python 移至 `legacy/` 目錄，repo 根目錄留給 Rust 專案
- 三層分離讓 stock-core 可以被 CLI、Tauri 和測試共用
- Rust 編譯器確保三方行為一致

### 2. SQLite 連線管理：Arc\<Mutex\<Connection\>\>

```rust
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
}
```

桌面板單一使用者情境不需要 connection pool（r2d2 是 overkill）。rusqlite 的 `Connection` 不是 `Send`，需要用 `Mutex` 保護。

**Alternatives considered:**
- `r2d2` pool：多執行緒 Server 用，桌面 App 不需要
- `tokio::sync::Mutex`：在 `spawn_blocking` 內使用 `std::sync::Mutex` 更輕量

### 3. SQLite Migration 策略：version PRAGMA + 順序執行

```rust
pub fn migrate(conn: &Connection) -> Result<()> {
    let current: i32 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    let migrations: Vec<(&str, &dyn Fn(&Connection) -> Result<()>)> = vec![
        ("001_initial_schema", &|c| {
            c.execute_batch("CREATE TABLE IF NOT EXISTS market_quote_core (...)")
        }),
        ("002_add_instrument_type", &|c| {
            c.execute_batch("ALTER TABLE market_quote_core ADD COLUMN instrument_type TEXT")
        }),
    ];
    for (i, (_name, f)) in migrations.iter().enumerate() {
        let version = (i + 1) as i32;
        if version > current {
            f(conn)?;
            conn.pragma_update(None, "user_version", version)?;
        }
    }
    Ok(())
}
```

**Rationale:** `PRAGMA user_version` 是 SQLite 內建的版本號儲存機制，不需額外表格。檔案式 migration 腳本可以之後再加入。

### 3.5 首次啟動資料庫複製

```text
首次啟動流程：

  ┌──────────────────┐
  │  應用程式啟動     │
  └────────┬─────────┘
           ▼
  ┌──────────────────┐     有
  │ app_data_dir 裡   │─────────→ 直接開啟，跳過複製
  │ 已有 market_data  │
  │ .db？            │
  └────────┬─────────┘
           │ 沒有
           ▼
  ┌──────────────────────────┐
  │ 在工作目錄搜尋           │
  │ market_data.db           │
  │ (專案根目錄 / 執行目錄)  │
  └────────┬─────────────────┘
           │ 找到
           ▼
  ┌──────────────────────────┐
  │ 複製到 app_data_dir      │
  │ 開啟連線                 │
  └──────────────────────────┘
```

```rust
fn initialize_db(app: &AppHandle) -> Result<Connection> {
    let app_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_dir)?;
    let db_path = app_dir.join("market_data.db");

    if !db_path.exists() {
        // 第一次啟動：從既有位置複製
        let candidates = [
            std::env::current_dir().map(|p| p.join("market_data.db")),
            // 開發時可能放在專案根目錄
        ];
        for candidate in candidates.iter().flatten() {
            if candidate.exists() {
                std::fs::copy(candidate, &db_path)?;
                break;
            }
        }
    }

    let conn = Connection::open(&db_path)?;
    // 執行 migration
    migrate(&conn)?;
    Ok(conn)
}
```

**Rationale:** 使用者已有既有資料庫，首次啟動複製過去可以無痛銜接，不需手動搬移。

### 4. 非同步邊界：spawn_blocking 包裹 rusqlite

```
Tauri command (async)
    │
    ├─ 計算密集型（SMA/RSI/MACD）：直接在本機執行緒算
    │
    └─ 資料庫操作：tokio::task::spawn_blocking(move || {
            db.query(...)
        }).await
```

Rust 的 async fn 在 tokio runtime 執行。rusqlite 是同步的，在 `spawn_blocking` 內執行避免阻塞 IPC 事件迴圈。

### 4.5 HTTP 測試策略：wiremock

```rust
// 測試範例
#[tokio::test]
async fn test_twse_fetcher() {
    let mock_server = wiremock::MockServer::start().await;

    // 註冊 mock 回應
    Mock::given(method("GET"))
        .and(path("/api/exchange/TAIEX"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(mock_twse_json(), "big5"))
        .mount(&mock_server)
        .await;

    let client = HttpClient::new(mock_server.uri());
    let result = client.fetch_daily_quotes("2026-06-10").await;
    assert!(result.is_ok());
}
```

```toml
# stock-core/Cargo.toml
[dev-dependencies]
wiremock = "0.6"
```

**Rationale:**
- 不需要額外 trait 抽象層，直接 mock HTTP server
- wiremock 在 Rust 生態中最成熟，支援請求匹配、回應模板、狀態驗證
- 對 TWSE/TPEX 的回應以 JSON fixture 檔管理

### 5. TWSE Big5 編碼處理：encoding_rs

TWSE 老系統回應是 `text/html; charset=Big5`，reqwest 預設用 UTF-8 解碼會出現亂碼。

```rust
let resp = client.get(url).send().await?;
let bytes = resp.bytes().await?;
// 檢測編碼，如果是 Big5 就轉換
let (text, _, _) = encoding_rs::BIG5.decode(&bytes);
// text 現在是 &str，交給 scraper 解析
```

**scraper 使用方式：**
```rust
use scraper::{Html, Selector};

let doc = Html::parse_document(&text);
let selector = Selector::parse("table.data-table tr").unwrap();
for row in doc.select(&selector) {
    // 逐列解析
}
```

**Alternatives considered:**
- `encoding` crate：功能類似，但 `encoding_rs` 是 Firefox 使用的編碼庫，更成熟
- 直接 `text/html; charset=Big5` → reqwest 自動解碼：reqwest 0.12 的 `charset` 支援有限，Big5 需手動處理

### 6. Bollinger Bands 純 Rust 實作

```rust
pub struct BollingerResult {
    pub upper: Vec<Option<f64>>,
    pub middle: Vec<Option<f64>>,
    pub lower: Vec<Option<f64>>,
}

pub fn bollinger(data: &[f64], period: usize, num_std: f64) -> BollingerResult {
    let n = data.len();
    let mut upper = vec![None; n];
    let mut middle = vec![None; n];
    let mut lower = vec![None; n];

    for i in (period - 1)..n {
        let window = &data[i + 1 - period..=i];
        let sma: f64 = window.iter().sum::<f64>() / period as f64;
        let variance: f64 = window.iter().map(|v| (v - sma).powi(2)).sum::<f64>() / period as f64;
        let stddev = variance.sqrt();
        middle[i] = Some(sma);
        upper[i] = Some(sma + num_std * stddev);
        lower[i] = Some(sma - num_std * stddev);
    }
    BollingerResult { upper, middle, lower }
}
```

**Numerical stability 注意事項：**
- 使用 `sum::<f64>()` 累加較大資料集可能累積浮點誤差 → 人數夠大時用 `f64::mul_add` 或 Kahan summation
- `variance.sqrt()` 在 variance 為負數時（極小的浮點誤差導致）回傳 NaN → `variance.max(0.0).sqrt()`
- p 值為 0 時產生 NaN SMA → 預先檢查 window 是否全為 0
- 第 `period - 1` 筆以前的資料回傳 `None`，ECharts 會自動忽略不畫

### 7. Tauri 事件系統 — 長時間任務進度回報

取代 Python 的 `queue.Queue` + `after()` 定時輪詢：

```rust
// Rust 端 — 在背景任務中發送事件
use tauri::Emitter;

#[tauri::command]
async fn batch_download(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let dates = get_date_range();
    let total = dates.len();
    for (i, date) in dates.iter().enumerate() {
        // ... 實際下載邏輯 ...
        app.emit("download-progress", serde_json::json!({
            "current": i + 1,
            "total": total,
            "date": date.to_string(),
        })).map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

```typescript
// Vue 端
import { listen } from '@tauri-apps/api/event';

onMounted(() => {
  const unlisten = listen<{current: number; total: number; date: string}>(
    'download-progress',
    (event) => {
      progress.value = event.payload.current / event.payload.total;
      statusText.value = `正在下載 ${event.payload.date}`;
    }
  );
  onUnmounted(() => unlisten.then(f => f()));
});
```

**關鍵區別：** Tauri 事件是 push-based（Server Sent Events 風格），比起 polling 更即時、省資源。

### 8. Vue 可調整參數表單

```vue
<script setup lang="ts">
const params = reactive({
  maPeriods: [5, 10, 20],       // 三條 MA 各自的天數
  bbPeriod: 20,                   // 布林通道週期
  bbStd: 2,                       // 標準差倍數
  rsiPeriod: 14,                  // RSI 週期
  macdFast: 12, macdSlow: 26, macdSignal: 9,  // MACD 三參數
  showVolume: true,               // 是否顯示成交量副圖
  showBB: true,                   // 是否顯示布林通道
  showRSI: false,                  // 是否顯示 RSI 副圖
  showMACD: false,                 // 是否顯示 MACD 副圖
});
</script>

<template>
  <div class="params-panel">
    <fieldset>
      <legend>移動平均線</legend>
      <label v-for="(_, i) in params.maPeriods" :key="i">
        MA{{ [5, 10, 20][i] }} 天數
        <input type="number" v-model.number="params.maPeriods[i]" min="2" max="120" />
      </label>
    </fieldset>
    <fieldset>
      <legend>布林通道</legend>
      <label>週期 <input type="number" v-model.number="params.bbPeriod" min="5" max="100" /></label>
      <label>標準差 <input type="number" v-model.number="params.bbStd" min="1" max="4" step="0.1" /></label>
    </fieldset>
    <fieldset>
      <legend>RSI</legend>
      <label>週期 <input type="number" v-model.number="params.rsiPeriod" min="2" max="50" /></label>
      <label><input type="checkbox" v-model="params.showRSI" /> 顯示 RSI 副圖</label>
    </fieldset>
    <fieldset>
      <legend>MACD</legend>
      <label>快線 <input type="number" v-model.number="params.macdFast" min="2" max="50" /></label>
      <label>慢線 <input type="number" v-model.number="params.macdSlow" min="10" max="100" /></label>
      <label>信號 <input type="number" v-model.number="params.macdSignal" min="2" max="50" /></label>
      <label><input type="checkbox" v-model="params.showMACD" /> 顯示 MACD 副圖</label>
    </fieldset>
    <fieldset>
      <legend>顯示控制</legend>
      <label><input type="checkbox" v-model="params.showVolume" /> 成交量</label>
      <label><input type="checkbox" v-model="params.showBB" /> 布林通道</label>
    </fieldset>
  </div>
</template>
```

**資料流：** params 變數由 `watch` 監聽，變化時重新呼叫 Tauri command 取得新的技術指標資料。

### 9. 技術指標資料回傳格式

```rust
#[derive(Serialize)]
pub struct ChartResponse {
    pub dates: Vec<String>,
    pub ohlc: Vec<[f64; 4]>,
    pub volumes: Vec<i64>,
    pub sma_data: Vec<Vec<Option<f64>>>,   // sma_data[0] = MA5, sma_data[1] = MA10, ...
    pub bbu: Vec<Option<f64>>,
    pub bbm: Vec<Option<f64>>,
    pub bbl: Vec<Option<f64>>,
    pub rsi: Vec<Option<f64>>,
    pub macd_line: Vec<Option<f64>>,
    pub macd_signal: Vec<Option<f64>>,
    pub macd_histogram: Vec<Option<f64>>,
}
```

所有 `Vec<Option<f64>>` 長度與 `dates` 一致，前 N 筆為 `None`（window 未滿）。ECharts 的 line series 遇到 `null` 會自動中斷線條或略過點。

### 10. 錯誤顯示策略：Toast 通知

使用 Vue 3 的 toast 元件庫（如 `vue-sonner`）顯示 Rust 指令回傳的錯誤。

```
使用者操作
    │
    ▼
Tauri command → 回傳 Err(AppError)
    │
    ▼
Vue catch → toast.error("下載失敗：網路連線逾時")
```

```typescript
// Vue 端統一的錯誤處理 wrapper
import { toast } from 'vue-sonner';
import { invoke } from '@tauri-apps/api/core';

async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
    try {
        return await invoke<T>(cmd, args);
    } catch (err) {
        const msg = typeof err === 'string' ? err : '發生未知錯誤';
        toast.error(msg);
        return null;
    }
}
```

**Rationale:**
- Toast 不阻塞使用者操作、可疊加、自動消失
- 比起對話框更輕量，比起內嵌錯誤列不需要預留版面
- 所有 Tauri command 的錯誤統一從 Rust `AppError` 序列化為字串傳到前端

## 整體資料流

```
使用者操作（Vue）
     │
     ▼ invoke()
Tauri Command
     │
     ├── call stock-core::analysis::bollinger(&prices, period, std)
     ├── call stock-core::analysis::sma(&prices, period)
     ├── call stock-core::analysis::rsi(&prices, period)
     └── call stock-core::analysis::macd(&prices, fast, slow, signal)
     │
     ▼ 回傳 ChartResponse (JSON)
ECharts setOption()
```

### 11. TDD 開發流程

每個 task 遵循紅-綠-藍循環：

```text
1. 寫測試 (RED)    → cargo test 失敗
2. 最少實作 (GREEN) → cargo test 通過
3. 重構 (BLUE)     → 維持綠燈下改進結構
    │
    ▼
   git commit（小步提交）
```

**測試粒度對照表：**

| 層級 | 測試框架 | 內容 |
|------|---------|------|
| stock-core 純函式 | `#[test]` | 已知輸入/輸出斷言，邊界案例 |
| stock-core Repository | `#[test]` + temp sqlite | CRUD、query、upsert |
| stock-core Fetcher | `#[tokio::test]` + wiremock | HTTP 回應解析 |
| Tauri Commands | `#[cfg(test)]` 整合測試 | IPC 資料流 |
| Vue 元件 | Vitest + @vue/test-utils | 元件渲染、store 互動 |

**小步提交原則：**
- 每個 commit 是一個可編譯、測試通過的狀態
- commit 訊息格式：`feat(core): 實作 SMA 計算` / `test(core): 加入 SMA 邊界案例`
- 建議粒度：1–3 個 task 一個 commit

### 12. 即時股價輪詢（Real-Time Polling）

#### 12.1 資料來源

TWSE 基本市況報導網站的公開 JSON API：

```
GET https://mis.twse.com.tw/stock/api/getStockInfo.jsp
  ?ex_ch=tse_{symbol}.tw
  &json=1
  &delay=0
```

免費、不需認證，約有 15–20 秒延遲。

#### 12.2 狀態機

```
                    ┌──────────────────────┐
                    │        IDLE          │
                    │  非交易時段           │
                    │  顯示「非交易時間」    │
                    └──────────┬───────────┘
                               │ 09:00~13:30 盤中
                               ▼
          ┌────────────────────────────────────────┐
          │               POLLING                  │
          │  interval = 10s, countdown: 10→0      │
          │  成功 → reset interval = 10s           │
          └───────┬────────────────────┬───────────┘
                  │ fetch 失敗          │ 時間 > 13:30
                  ▼                    ▼
   ┌──────────────────────┐  ┌──────────────────┐
   │       BACKOFF        │  │     EOD_SYNC     │
   │  interval += 10s     │  │  每 60s 嘗試     │
   │  max = 600s (10分)   │  │  TWSE EOD API    │
   │  成功 → POLLING      │  │  有資料 → upsert │
   └──────────────────────┘  │  成功 → IDLE     │
                              └──────────────────┘
```

#### 12.3 Rust 端：單一 Tauri Command

```rust
#[tauri::command]
async fn fetch_realtime_quote(symbol: String) -> Result<RealTimeQuote, String>
```

回傳今日即時資料：price（未成交為 None）、open、high、low、volume、bid、ask、change、change_percent、timestamp。DB 層完全不需改動，即時資料是 ephemeral（不入 DB）。

#### 12.4 Vue 端：useRealtimePolling composable

```typescript
// composables/useRealtimePolling.ts
export function useRealtimePolling(symbol: Ref<string>) {
  const quote = ref<RealTimeQuote | null>(null)
  const countdown = ref(0)
  const interval = ref(10)
  const status = ref<'idle' | 'polling' | 'backoff' | 'nontrading'>('idle')

  // 每秒 tick，countdown 歸零時觸發 fetch
  // fetch 成功 → reset interval = 10s
  // fetch 失敗 → interval += 10s, max 600s
  // 非盤中 → status = 'nontrading'
  // 切換 symbol → 重啟

  return { quote, countdown, status, start, stop }
}
```

倒數計時邏輯：

```typescript
countdown.value--                    // 每秒
if (countdown.value <= 0) {
  poll()
  countdown.value = currentInterval  // 可能是 10/20/30...s
}
```

#### 12.5 K 線今日 bar 更新

`ChartResponse` 和 `AnalysisEngine` 完全不需要改動。前端直接修改 chartStore 裡最後一筆 OHLC：

```typescript
function updateTodayBar(rt: RealTimeQuote) {
  const bars = chartStore.chartData.ohlc
  const last = bars.length - 1
  bars[last] = [
    rt.open,                    // open 不變
    Math.max(rt.high, rt.price),
    Math.min(rt.low, rt.price),
    rt.price,
  ]
  chartStore.chartData.volumes[last] = rt.volume
}
```

#### 12.6 收盤後 EOD 補 DB

```
13:30 偵測到收盤 → 進入 EOD_SYNC 狀態
每 60 秒呼叫 TWSE STOCK_DAY_ALL (OpenAPI)
  有資料 → upsert_quotes_batch → 切到 IDLE
  無資料 → 繼續等（TWSE 通常 13:40 前發布）
```

同時提供手動按鈕：

```html
<button @click="manualEodSync">
  📥 下載今日資料
</button>

<script>
async function manualEodSync() {
  const result = await invoke('fetch_and_ingest_today')
  if (result.already_exists) {
    toast.info('今日資料已存在')
  } else {
    toast.success(`已下載 ${result.count} 筆資料`)
  }
}
</script>
```

#### 12.7 畫面元件

```
┌──────────────────────────────────────────────────────┐
│  🔍 [2330 台積電]   ⏰ 14:32:15   🔄 下次更新 7s   │
│                                                      │
│  ┌──────────────────────────────────────┐           │
│  │  K 線圖（今日 bar 隨即時價跳動）      │           │
│  │  SMA/BB/RSI/MACD 不動                │           │
│  └──────────────────────────────────────┘           │
│                                                      │
│  ┌──────────────┐  ┌──────────────────────────────┐ │
│  │ 即時報價       │  │ MA 5  10  20               │ │
│  │ 最新 1005.00  │  │ BB 20  2.0                 │ │
│  │ 漲跌 +15.00   │  │ RSI 14  顯示               │ │
│  │ 漲幅 +1.52%   │  │ MACD 12 26 9 顯示          │ │
│  │ 今高 1010.00  │  │                             │ │
│  │ 今低 1000.00  │  │                             │ │
│  │ 量 12,345,678 │  │                             │ │
│  └──────────────┘  └──────────────────────────────┘ │
└──────────────────────────────────────────────────────┘
```

#### 12.8 技術指標邊界

**SMA/RSI/MACD/布林通道以 daily close 為基準，不隨即時價格跳動。** 理由：10 秒一跳的 intraday 價格會讓季線每天抖 300 次，訊號雜訊比過低。

### 13. ECharts 即時更新動畫

#### 13.1 動畫參數

```typescript
// chart 初始化時設定
const chart = echarts.init(container)
chart.setOption({
  animationDuration: 800,          // 首次繪製（只看一次）
  animationDurationUpdate: 100,    // ★ 每次即時更新（100ms 平滑）
  animationEasingUpdate: 'linear', // 線性補間，不要彈跳
  series: [{
    type: 'candlestick',
    animation: true,
    data: ohlc,
  }, {
    type: 'bar',           // 成交量
    animation: true,
    data: volumes,
  }]
})
```

#### 13.2 更新模式：merge，不 replace

```typescript
// ✅ 只更新變動的資料
chart.setOption({
  series: [{
    data: updatedOhlc    // 陣列中只有最後一個元素不同
  }, {
    data: updatedVolumes
  }]
}, { notMerge: false })  // merge mode，ECharts 做 data diff
```

ECharts data diff 的行為：
- 只有最後一個 data point 變動 → 只動畫那一根
- candlestick 顏色翻轉（紅↔綠）內建處理，不需額外程式碼
- tooltip 會 auto-update，hover 顯示就跟著變

#### 13.3 多圖格同步

```typescript
// 四個圖格共用 dataZoom 實例
chart.setOption({
  grid: [
    { left: 60, right: 60, top: 10, height: '55%' },   // 價格
    { left: 60, right: 60, top: '68%', height: '10%' }, // 成交量
    { left: 60, right: 60, top: '80%', height: '10%' }, // RSI
    { left: 60, right: 60, top: '92%', height: '8%' },  // MACD
  ],
  dataZoom: [{
    type: 'inside',     // 滑鼠滾輪縮放（所有圖格共用）
    xAxisIndex: [0, 1, 2, 3],
  }, {
    type: 'slider',     // 底部滑桿
    xAxisIndex: [0, 1, 2, 3],
    bottom: 0,
  }]
})
```

四個圖格的縮放/平移永遠同步，一次拖動控制全部。

#### 13.4 dataZoom 策略：自動追尾 + 保持縮放

```
使用者行為狀態機：

  ┌──────────────────────────────────┐
  │        進入分析頁 / 換股          │
  │  mode = autoFollow               │
  │  顯示最近 60 根                   │
  │  end = 100% 固定                  │
  └──────────────┬───────────────────┘
                 │
          ┌──────┴──────┐
          │ 使用者拖動   │
          │ dataZoom    │
          └──────┬──────┘
                 ▼
  ┌──────────────────────────────────┐
  │  mode = keepZoom                 │
  │  zoom 位置固定，不隨即時更新滑動  │
  │  新資料可能在視野外               │
  └──────────────┬───────────────────┘
                 │
          ┌──────┴──────┐
          │ 使用者滑到   │
          │ 最右邊       │
          │ (end=100%)  │
          └──────┬──────┘
                 ▼
  ┌──────────────────────────────────┐
  │  mode = autoFollow（回到自動追尾）│
  └──────────────────────────────────┘
```

```typescript
const userZoomed = ref(false)

chart.on('dataZoom', (params) => {
  if (params.from !== 'auto') {
    userZoomed.value = true          // 使用者手動拖 → 停用自動追尾
  }
})

watch(quote, () => {
  if (!userZoomed.value) {
    // 自動追尾
    chart.dispatchAction({
      type: 'dataZoom',
      startValue: dates[dates.length - 61],
      endValue: dates[dates.length - 1],
      from: 'auto',
    })
  }
})

// 當使用者滑到最右邊 → 回到自動追尾
function onSliderChange(end: number) {
  if (end >= 99.9) {
    userZoomed.value = false
  }
}
```

**Rationale:** 進頁面時自動顯示最新資料；使用者想分析特定區間時可固定 zoom；滑到最右邊無痛回歸即時模式。

## Risks / Trade-offs

- **[TWSE mis API 穩定性]** TWSE 基本市況報導站的 API 無 SLA，可能偶爾失效 → 透過 backoff + toast 通知處理
- **[Rust 學習曲線]** 團隊需要熟悉 Rust 所有權、生命週期、trait 系統 → 可從 stock-core 的純函式（無生命週期參數）開始，逐步深入
- **[pandas 到 Rust 的 gap]** 部分資料處理（如 groupby 統計）在 Rust 較繁瑣 → 多數操作可透過 rusqlite SQL 語句完成，不需記憶體級 DataFrame
- **[前端打包體積]** AG Grid 社群版 ~200KB gzip，ECharts 可 tree-shaking → 只 import 使用的圖表類型（candlestick、line、bar）
- **[Big5 編碼]** TWSE 部分端點仍使用 Big5，encoding_rs 解碼後可能仍有特殊字元 → 加入 `String::from_utf8_lossy` fallback
- **[跨平台字型]** ECharts 無字型問題（瀏覽器處理）→ 但 macOS Safari 與 Windows WebView2 的渲染略有差異 → 使用 Tauri 2.0 的 webview 統一設定
