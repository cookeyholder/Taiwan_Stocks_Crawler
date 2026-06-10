## Context

Tab 2（布林擠壓選股器）目前計算所有證券的布林通道，包含權證（11,155 檔）、普通股（2,035 檔）、ETF（195 檔，但實際有 66K 被誤歸類為 other）等。由於分類規則缺口，66K 證券被歸入「other」黑盒子。右側圖表僅顯示收盤價折線圖，無 K 線。

## Goals / Non-Goals

**Goals:**
- 補強證券分類規則，涵蓋 6 碼純數字 ETF、5 碼+特定字母槓桿/反向/主動型 ETF、可轉債
- 對既有資料庫執行一次性重新分類
- Tab 2 新增證券類型篩選 checkboxes（股票固定顯示）
- 新增最低成交量門檻篩選
- Treeview 欄位點擊排序（遞增/遞減切換）
- 右側圖表改為 K 線圖疊加布林通道，成交量柱依漲跌著色

**Non-Goals:**
- 不修改 Tab 1（個股查詢）、Tab 3（下載管理）
- 不更動爬蟲 pipeline 的資料擷取邏輯
- 不引入資料庫 schema 變更（instrument_type 已存在）

## Decisions

### 1. 分類規則補強順序與模式

在 `classification.py` 的 `_from_ticker_pattern()` 中，新增以下規則（插入現有規則之間）：

```
現有: \d{4}                     → stock
現有: \d{4,5}B                  → etf (債券ETF)
新增: \d{6}                     → etf (6碼純數字：一般ETF、ETN、封閉式基金)
現有: \d{4}[A-Y]               → preferred_stock
現有: \d{4}Z[1-9]              → exchangeable_preferred_stock
現有: \d{4}G[1-9]              → warrant_certificate
現有: \d{2}\d{3}[PUTFQCBXY]    → warrant
新增: \d{5}[KLMRSAD]           → etf (槓桿/反向/外幣/主動式ETF)
新增: \d{5}                    → convertible_bond (5碼純數字)
現有: [BGFAHPI]\w{5}           → bond
現有: T\w{5}                   → fund
```

**理由**：6 碼純數字 ETF（如 006208 富邦台50）目前落入 other；5 碼+L/R/K/M/S/A/D（如 00631L、00632R）涵蓋槓桿/反向/主動 ETF。兩者在實際資料中確認無誤。5 碼純數字為可轉債，獨立類型以便篩選。

### 2. 既有資料重新分類策略

使用 `UPDATE market_quote_core SET instrument_type = ...` 一次性的 SQL 操作，在 `classification.py` 中提供 `reclassify_database(conn, stock_master_path)` 函數，於 GUI 啟動時自動執行。

**替代方案考量**：
- 僅在新爬蟲時更新 × 既有 other 永不修正
- 建立 migration 腳本獨立執行 × 使用者需手動觸發
- **選擇：自動執行** → 低風險（僅更新已存在的欄位值），對使用者透明

### 3. Screener 參數傳遞

```python
def screen_bollinger_squeeze(
    conn,
    bandwidth_limit=0.08,
    consecutive_days=5,
    instrument_types: list[str] | None = None,  # 新參數
    min_volume: int = 0,                         # 新參數
) -> pd.DataFrame:
```

篩選在 SQL 層級執行 (`WHERE instrument_type IN (...) AND volume_shares >= ?`)，減少 Python 端計算量。對於大量權證資料尤其重要（11K 檔可略過計算）。

### 4. GUI 佈局

篩選控制區擴充：

```
┌─ 篩選設定 ──────────────────────────────────────┐
│ 帶寬上限 [0.08]  持續天數 [5]  最低成交量 [0]    │
│                                                   │
│ 證券類型:                                          │
│  ☑ 股票 (必選)  ☐ ETF  ☐ 權證  ☐ 可轉債          │
│  ☐ 特別股       ☐ 其他                            │
│                                                   │
│ [🔍 開始本地篩選]  [☁️ 批次同步台灣50歷史日K]     │
└─────────────────────────────────────────────────┘
```

排序狀態透過 `self.t2_sort_col` / `self.t2_sort_asc` 追蹤，點擊欄位標題時重新排序 Treeview items（不重新查詢資料庫）。

### 5. K 線圖實作方式

引入 `mplfinance`，使用 `mpf.plot(..., type='candle', addplot=...)` 疊加布林通道。成交量柱使用 `mpf.make_addplot()` 置於下方子圖，依收盤漲跌著色（紅漲綠跌）。

**替代方案考量**：
- 手刻 matplolib 蠟燭圖 × 程式碼量較大，需自行處理顏色/寬度
- **選擇 mplfinance** → 專用套件，開發成本低，維護簡單

## Risks / Trade-offs

- **[分類邊界]** 5 碼純數字可能包含少數非可轉債商品。若發生誤判，可透過 override dict 手動修正，或引入 SecurityMaster 精確判斷。
- **[mplfinance 相容性]** 需確認 mplfinance 支援目前 matplotlib 版本。加入依賴後需更新 pyproject.toml 與鎖定檔案。
- **[DB migration 時機]** 若資料庫龐大（400 萬筆），UPDATE 可能耗時數秒。GUI 啟動時以背景執行緒執行，不阻塞 UI。
- **[排序效能]** Treeview 排序在瀏覽器級（數百 row）無效能問題，超過則需虛擬滾動。
