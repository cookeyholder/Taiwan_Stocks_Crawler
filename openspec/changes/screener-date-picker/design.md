## Context

布林通道選股（ScreenerView）目前固定以 `repo.get_latest_trade_date()` 作為篩選日期，無法查詢歷史特定日期的突破訊號。使用者需要日期選擇器來指定要篩選的交易日。

## Goals / Non-Goals

**Goals:**
- 前端 ScreenerView 新增日期選擇器，預設填入最新交易日
- 後端 `Pipeline::screen()` 接受 `Option<&str>` 日期參數
- Tauri command `screen_bollinger` 接受 `Option<String>` 日期參數
- CLI `screen` 子命令新增 `--date` 選項
- `BacktestEngine::run_date()` 已支援任意日期，無需修改

**Non-Goals:**
- 不引入第三方日期選擇器套件（使用原生 `<input type="date">`）
- 不修改現有的 `BacktestEngine` 或 `BollingerSqueezeBreakout` 策略邏輯
- 不處理日期區間篩選（僅單日）

## Decisions

| 決策 | 選項 | 選擇理由 |
|------|------|---------|
| 日期 UI | `<input type="date">` vs 自訂選擇器 | 原生 `type="date"` 支援良好（Chrome/Firefox/Edge），無需額外套件；placeholder 搭配 `YYYY-MM-DD` 驗證 |
| 預設值 | 從 `get_latest_trade_date` 取得 | 與現有行為一致（最新 = 最新交易日），減少使用者操作 |
| 後端簽名 | `Option<&str>` vs 多載函式 | 單一選項參數最簡潔；`None` = 向後相容（使用最新日期） |
| 日期驗證 | 後端 `NaiveDate::parse_from_str` | `BacktestEngine::run_date()` 已有驗證，回傳 `EngineError::InvalidDate` |
| 初始化 race | store 加 `dateLoading` flag | mount 時非同步取得最新交易日，取得前禁用篩選按鈕，避免送出空日期 |
| 無資料提示 | 前端判斷 `results.length == 0 && date 有值` | spec 已定義此行為，不需後端變更；前端顯示提示而非空白表格 |

## Data Flow

```
┌─ Mount ─────────────────────────────────────┐
│ ScreenerView.vue onMounted                  │
│   → screenerStore.fetchLatestTradeDate()    │
│     → dateLoading = true                    │
│     → invoke("get_latest_trade_date")       │
│     → screenerStore.date = "2026-07-14"     │
│     → dateLoading = false                   │
│     → button enabled                        │
└─────────────────────────────────────────────┘

┌─ Screen ─────────────────────────────────────┐
│ 使用者點擊篩選 → ScreenerView.vue            │
│   → (dateLoading ? 略過 : 繼續)              │
│   → screenerStore.runScreen()                │
│     → invoke("screen_bollinger", {           │
│         maxBandwidth, minDays,               │
│         instrumentTypes, minVolume,          │
│         date: store.date                     │
│       })                                     │
│       → screen_bollinger Tauri command       │
│         → Pipeline::screen(..., date)        │
│           → Some(d)? run_date(d)             │
│           : None → run_date(latest)          │
│                                             │
│ 後端回傳空陣列 + date 有值                    │
│   → 顯示「該日期無交易資料」                  │
└─────────────────────────────────────────────┘
```

## Risks / Trade-offs

- [向後相容] 現有呼叫者不傳 date → 行為不變
- [無效日期] 後端已有 `EngineError::InvalidDate` 處理
- [無資料日期] `BacktestEngine::run_date` 對無資料日期回傳空 signals，前端需判斷並顯示提示
- [初始化 race] mount 時 `get_latest_trade_date` 尚未回傳，使用者可能提前點擊篩選 → 以 `dateLoading` flag 防護
- [日期範圍] 無 min/max 限制時，使用者可選未來日期或過早年份 → 回傳空結果（不影響系統，僅 UX 問題，暫不處理）
