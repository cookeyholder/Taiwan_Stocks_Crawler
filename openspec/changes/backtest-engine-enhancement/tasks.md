## 1. 資料模型

- [x] 1.1 在 `models.rs` 新增 `PerformanceMetrics` struct（total_return_pct, annualized_return_pct, win_rate, total_trades, winning_trades, losing_trades, sharpe_ratio, max_drawdown_pct, final_value），derive `Serialize, Deserialize`
- [x] 1.2 新增 `TradeRecord` struct（symbol, name, buy_date, buy_price, sell_date, sell_price, shares, pnl, pnl_pct）
- [x] 1.3 新增 `BacktestReport` struct（signals, metrics, trades, equity_curve, trade_dates）— `equity_curve` 的 `Vec<f64>` 索引對應 `trade_dates` 排序後的位置
- [x] 1.4 新增 `StrategyInfo` struct（id, name, params: Vec<ParamDef>）
- [x] 1.5 新增 `ParamDef` struct（name, type_, default, min, max）

## 2. PortfolioSimulator

- [x] 2.1 新增 `src/backtest/portfolio.rs`，實作 `PortfolioSimulator`
      - `simulate(signals, initial_capital, prices_map, dates) → (Vec<TradeRecord>, Vec<f64>)`
      - `prices_map` 型別為 `HashMap<String, BTreeMap<String, f64>>`（symbol → (日期 → 收盤價)）
      - `dates: &[String]` 為回測區間內所有交易日（已排序），用於逐日計算權益曲線
      - 權益曲線每日記錄：現金 + 持股市值（以當日收盤價計算）
- [x] 2.2 處理 direction=1（買進）、direction=-1（賣出）、direction=0（無動作）
- [x] 2.3 處理資金不足與未持倉賣出

## 3. PerformanceMetrics

- [x] 3.1 新增 `src/backtest/performance.rs`，實作 `calculate_metrics(trades, initial_capital, equity_curve) → PerformanceMetrics`
- [x] 3.2 實作勝率、總報酬率計算
- [x] 3.3 實作 Sharpe ratio（以 0% 無風險利率簡化）
- [x] 3.4 實作最大回撤

## 4. 新策略：MaCross

- [x] 4.1 新增 `src/backtest/ma_cross.rs`，實作 `MaCrossStrategy`（fast_period, slow_period, min_volume）
- [x] 4.2 實作 `Strategy` trait：快 SMA 突破慢 SMA 產生方向訊號
- [x] 4.3 測試：上漲趨勢中產生買進訊號、下跌趨勢中產生賣出訊號

## 5. 新策略：RSI

- [x] 5.1 新增 `src/backtest/rsi.rs`，實作 `RsiStrategy`（period, overbought, oversold, min_volume）
- [x] 5.2 實作 `Strategy` trait：RSI 穿越超買/超賣線產生訊號
- [x] 5.3 測試：RSI 低於 oversold 後回升產生買進訊號

## 6. StrategyKind 策略註冊

- [x] 6.1 新增 `src/backtest/strategies.rs`，實作 `StrategyKind` enum
- [x] 6.2 實作 `list() -> Vec<StrategyInfo>`
- [x] 6.3 實作 `create(id, params) -> Result<Self>`
- [x] 6.4 實作 `min_history()` 與 `evaluate()` 委託
- [x] 6.5 在 `backtest/mod.rs` 匯出新模組

## 7. CLI 強化

- [x] 7.1 `backtest` 子命令新增 `--strategy <name>`（預設 `bollinger_squeeze`）
- [x] 7.2 新增 `--param key=value` 可重複指定
- [x] 7.3 新增 `--capital <N>`（預設 1000000）
- [x] 7.4 輸出績效摘要（勝率、Sharpe、總報酬率、最大回撤）
- [x] 7.5 輸出交易明細（日期、代號、買/賣、價格、損益）
- [x] 7.6 從 `run_range` 取得的 `ohlcv_map` 建構 `prices_map: HashMap<String, BTreeMap<String, f64>>` 與 `dates: Vec<String>` 傳入 `PortfolioSimulator::simulate()`

## 8. 格式化與測試

- [x] 8.1 `cargo test` 確認原有 48 項測試 + 新增測試全部通過
- [x] 8.2 `cargo fmt --all -- --check`
