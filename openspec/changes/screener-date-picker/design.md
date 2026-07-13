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

## Data Flow

```
使用者選擇日期 → ScreenerView.vue
  → screenerStore.date (reactive state)
    → invoke("screen_bollinger", { maxBandwidth, minDays, date, ... })
      → Pipeline::screen(bandwidth, days, types, volume, date)
        → date? run_date(date) : run_date(get_latest_trade_date())
```

## Risks / Trade-offs

- [向後相容] 現有呼叫者不傳 date → 行為不變
- [無效日期] 後端已有 `EngineError::InvalidDate` 處理
- [無資料日期] `BacktestEngine::run_date` 對無資料日期回傳空 signals，前端顯示空結果
