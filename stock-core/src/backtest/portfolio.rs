use std::collections::HashMap;

use crate::backtest::SignalRecord;
use crate::models::{OhlcvRecord, TradeRecord};

struct OpenPosition {
    buy_price: f64,
    shares: i64,
}

pub struct PortfolioSimulator;

impl PortfolioSimulator {
    pub fn simulate(
        signals: &[SignalRecord],
        initial_capital: f64,
        ohlcv_map: &HashMap<String, Vec<OhlcvRecord>>,
        trade_dates: &[String],
    ) -> (Vec<TradeRecord>, Vec<f64>) {
        let mut trades: Vec<TradeRecord> = Vec::new();
        let mut equity: Vec<f64> = Vec::new();

        // Phase 1: process signals chronologically into trades
        let mut cash = initial_capital;
        let mut positions: HashMap<String, OpenPosition> = HashMap::new();

        for signal in signals {
            match signal.direction {
                1 => {
                    if positions.contains_key(&signal.symbol) {
                        continue;
                    }
                    let price = signal.close_price.unwrap_or(0.0);
                    if price <= 0.0 || cash < price {
                        continue;
                    }
                    let shares = (cash / price) as i64;
                    if shares < 1 {
                        continue;
                    }
                    cash -= price * shares as f64;
                    positions.insert(
                        signal.symbol.clone(),
                        OpenPosition {
                            buy_price: price,
                            shares,
                        },
                    );
                    trades.push(TradeRecord {
                        symbol: signal.symbol.clone(),
                        name: signal.name.clone(),
                        buy_date: signal.trade_date.clone(),
                        buy_price: price,
                        sell_date: None,
                        sell_price: None,
                        shares,
                        pnl: 0.0,
                        pnl_pct: 0.0,
                    });
                }
                -1 => {
                    if let Some(pos) = positions.remove(&signal.symbol) {
                        let price = signal.close_price.unwrap_or(0.0);
                        if price <= 0.0 {
                            continue;
                        }
                        let proceeds = price * pos.shares as f64;
                        let cost = pos.buy_price * pos.shares as f64;
                        let pnl = proceeds - cost;
                        let pnl_pct = (price - pos.buy_price) / pos.buy_price * 100.0;
                        cash += proceeds;
                        if let Some(t) = trades
                            .iter_mut()
                            .rev()
                            .find(|t| t.symbol == signal.symbol && t.sell_date.is_none())
                        {
                            t.sell_date = Some(signal.trade_date.clone());
                            t.sell_price = Some(price);
                            t.pnl = pnl;
                            t.pnl_pct = pnl_pct;
                        }
                    }
                }
                _ => {}
            }
        }

        // Phase 2: build equity curve from trades + OHLCV data
        let mut active: HashMap<String, (f64, i64)> = HashMap::new();
        let mut trade_idx = 0usize;

        for date in trade_dates {
            // Apply trades for this date
            while trade_idx < trades.len() {
                let t = &trades[trade_idx];

                // Skip trades whose buy date hasn't been reached yet
                if t.buy_date > *date {
                    break;
                }

                if t.sell_date.is_none() {
                    // Still holding (no sell signal yet)
                    active.insert(t.symbol.clone(), (t.buy_price, t.shares));
                } else if t.sell_date.as_deref() == Some(date.as_str()) {
                    // Selling on this date
                    active.remove(&t.symbol);
                } else if t.sell_date.as_deref().unwrap_or("") < date.as_str() {
                    // Sold before this date - just skip
                }

                trade_idx += 1;
            }

            let mut total = cash;
            for (sym, &(_, shares)) in &active {
                if let Some(records) = ohlcv_map.get(sym)
                    && let Some(rec) = records.iter().find(|r| r.trade_date == *date) {
                        total += rec.close_price.unwrap_or(0.0) * shares as f64;
                    }
            }
            equity.push(total);
        }

        (trades, equity)
    }
}
