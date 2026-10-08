use std::collections::HashMap;

use crate::backtest::{Signal, Strategy};
use crate::models::OhlcvRecord;

pub struct BollingerSqueezeBreakout {
    pub bandwidth_limit: f64,
    pub consecutive_days: usize,
    pub min_volume: i64,
    pub require_break_upper: bool,
}

impl Default for BollingerSqueezeBreakout {
    fn default() -> Self {
        Self {
            bandwidth_limit: 0.08,
            consecutive_days: 5,
            min_volume: 0,
            require_break_upper: false,
        }
    }
}

impl Strategy for BollingerSqueezeBreakout {
    fn id(&self) -> &str {
        "bollinger_squeeze"
    }

    fn min_history(&self) -> usize {
        self.consecutive_days + 20
    }

    fn evaluate(&self, ohlcv: &[OhlcvRecord], date_index: usize) -> Signal {
        let start_idx = date_index.saturating_sub(self.min_history() - 1);
        let ohlcv_slice = &ohlcv[start_idx..=date_index];
        let prices: Vec<f64> = ohlcv_slice.iter().filter_map(|r| r.close_price).collect();

        if prices.len() < 20 {
            return Signal {
                has_signal: false,
                direction: 0,
                strength: 0.0,
                metadata: HashMap::new(),
            };
        }

        let bollinger = crate::analysis::bollinger(&prices, 20, 2.0);
        let bandwidth: Vec<Option<f64>> = bollinger
            .upper
            .iter()
            .zip(bollinger.lower.iter())
            .zip(bollinger.middle.iter())
            .map(|((u, l), m)| match (u, l, m) {
                (Some(u), Some(l), Some(m)) if *m != 0.0 => Some((u - l) / m),
                _ => None,
            })
            .collect();

        let valid_bw: Vec<f64> = bandwidth.iter().filter_map(|b| *b).collect();
        let needed_days = if self.require_break_upper {
            self.consecutive_days + 1
        } else {
            self.consecutive_days
        };
        if valid_bw.len() < needed_days {
            return Signal {
                has_signal: false,
                direction: 0,
                strength: 0.0,
                metadata: HashMap::new(),
            };
        }

        // 若要求突破，擠壓檢驗的是突破前 consecutive_days 天（避免當日突破拉大頻寬而被誤過濾）
        let recent: Vec<f64> = if self.require_break_upper {
            valid_bw
                .iter()
                .rev()
                .skip(1)
                .take(self.consecutive_days)
                .copied()
                .rev()
                .collect()
        } else {
            valid_bw
                .iter()
                .rev()
                .take(self.consecutive_days)
                .copied()
                .rev()
                .collect()
        };
        let max_bw = recent.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        if max_bw > self.bandwidth_limit {
            return Signal {
                has_signal: false,
                direction: 0,
                strength: 0.0,
                metadata: HashMap::new(),
            };
        }

        let latest = &ohlcv[date_index];
        if self.min_volume > 0 {
            match latest.volume_shares {
                Some(v) if v >= self.min_volume => {}
                _ => {
                    return Signal {
                        has_signal: false,
                        direction: 0,
                        strength: 0.0,
                        metadata: HashMap::new(),
                    };
                }
            }
        }

        let break_upper = match (latest.open_price, latest.close_price) {
            (Some(o), Some(c)) if c >= o => bollinger
                .upper
                .last()
                .and_then(|u| u.map(|upper| c > upper))
                .unwrap_or(false),
            _ => false,
        };

        if self.require_break_upper && !break_upper {
            return Signal {
                has_signal: false,
                direction: 0,
                strength: 0.0,
                metadata: HashMap::new(),
            };
        }

        let latest_upper_band = bollinger.upper.last().and_then(|v| *v);

        let mut metadata = HashMap::new();
        metadata.insert("max_bandwidth".to_string(), max_bw);
        metadata.insert("bandwidth".to_string(), *recent.last().unwrap_or(&0.0));
        if let Some(upper) = latest_upper_band {
            metadata.insert("upper_band".to_string(), upper);
        }
        metadata.insert(
            "break_upper".to_string(),
            if break_upper { 1.0 } else { 0.0 },
        );

        Signal {
            has_signal: true,
            direction: 1,
            strength: max_bw,
            metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::NaiveDate;

    use crate::backtest::bollinger::BollingerSqueezeBreakout;
    use crate::backtest::engine::BacktestEngine;
    use crate::db::MarketRepository;
    use crate::models::CanonicalQuote;

    fn make_quote(symbol: &str, date: &str, close: f64) -> CanonicalQuote {
        CanonicalQuote {
            trade_date: date.to_string(),
            market_code: "TWSE".to_string(),
            symbol_raw: symbol.to_string(),
            symbol_clean: symbol.to_string(),
            security_name: Some(symbol.to_string()),
            instrument_type: "stock".to_string(),
            instrument_subtype: None,
            isin_code: None,
            cfi_code: None,
            section_label: None,
            open_price: Some(close),
            high_price: Some(close),
            low_price: Some(close),
            close_price: Some(close),
            volume_shares: Some(1000),
            turnover_value: None,
            trade_count: None,
            change_sign: None,
            change_value: None,
            last_bid_price: None,
            last_bid_qty_raw: None,
            last_ask_price: None,
            last_ask_qty_raw: None,
            pe_ratio: None,
            issued_shares: None,
            next_limit_up: None,
            next_limit_down: None,
            source_file: "test".to_string(),
            source_encoding: "utf-8".to_string(),
            source_line_no: 0,
            ingest_run_id: "test-ingest".to_string(),
            ingested_at: "2024-01-01T00:00:00".to_string(),
            extension: HashMap::new(),
        }
    }

    #[test]
    fn test_empty_on_no_data() {
        let repo = MarketRepository::open_in_memory().unwrap();
        let engine = BacktestEngine::new(&repo);
        let strategy = BollingerSqueezeBreakout::default();
        let output = engine.run_date("2024-01-01", &strategy, None).unwrap();
        assert!(output.signals.is_empty());
    }

    #[test]
    fn test_finds_tight_symbol() {
        let repo = MarketRepository::open_in_memory().unwrap();
        let mut quotes = Vec::new();
        let start = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        for i in 0..40 {
            let date = start + chrono::Duration::days(i);
            let date_str = date.format("%Y-%m-%d").to_string();
            quotes.push(make_quote("TIGHT", &date_str, 100.0));
            let wide_close = if i % 2 == 0 { 100.0 } else { 200.0 };
            quotes.push(make_quote("WIDE", &date_str, wide_close));
        }
        repo.upsert_core_batch(&quotes).unwrap();

        let latest = repo.get_latest_trade_date().unwrap().unwrap();
        let engine = BacktestEngine::new(&repo);
        let strategy = BollingerSqueezeBreakout {
            bandwidth_limit: 0.1,
            consecutive_days: 3,
            min_volume: 0,
            require_break_upper: false,
        };
        let output = engine.run_date(&latest, &strategy, None).unwrap();
        let symbols: Vec<&str> = output.signals.iter().map(|s| s.symbol.as_str()).collect();
        assert!(symbols.contains(&"TIGHT"));
        assert!(!symbols.contains(&"WIDE"));
    }

    #[test]
    fn test_run_date_respects_instrument_type_filter() {
        let repo = MarketRepository::open_in_memory().unwrap();
        let mut quotes = Vec::new();
        let start = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        for i in 0..40 {
            let date = start + chrono::Duration::days(i);
            let date_str = date.format("%Y-%m-%d").to_string();
            let mut q = make_quote("STOCK_A", &date_str, 100.0);
            q.instrument_type = "stock".to_string();
            quotes.push(q);
            let mut q = make_quote("ETF_B", &date_str, 100.0);
            q.instrument_type = "etf".to_string();
            quotes.push(q);
        }
        repo.upsert_core_batch(&quotes).unwrap();

        let latest = repo.get_latest_trade_date().unwrap().unwrap();
        let engine = BacktestEngine::new(&repo);
        let strategy = BollingerSqueezeBreakout {
            bandwidth_limit: 10.0,
            consecutive_days: 1,
            min_volume: 0,
            require_break_upper: false,
        };
        let output = engine
            .run_date(&latest, &strategy, Some(&["stock".to_string()]))
            .unwrap();
        let symbols: Vec<&str> = output.signals.iter().map(|s| s.symbol.as_str()).collect();
        assert!(symbols.contains(&"STOCK_A"));
        assert!(!symbols.contains(&"ETF_B"));
    }

    #[test]
    fn test_run_range_finds_signals_over_multiple_dates() {
        let repo = MarketRepository::open_in_memory().unwrap();
        let mut quotes = Vec::new();
        let start = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        // Insert 40 trading days of tight price data for candidate
        for i in 0..40 {
            let date = start + chrono::Duration::days(i);
            let date_str = date.format("%Y-%m-%d").to_string();
            quotes.push(make_quote("CANDIDATE", &date_str, 100.0));
        }
        repo.upsert_core_batch(&quotes).unwrap();

        let engine = BacktestEngine::new(&repo);
        let strategy = BollingerSqueezeBreakout {
            bandwidth_limit: 0.1,
            consecutive_days: 3,
            min_volume: 0,
            require_break_upper: false,
        };
        // Run range over the last 10 trade dates
        let range_start = (start + chrono::Duration::days(30))
            .format("%Y-%m-%d")
            .to_string();
        let range_end = (start + chrono::Duration::days(39))
            .format("%Y-%m-%d")
            .to_string();
        let output = engine
            .run_range(&range_start, &range_end, &strategy, None)
            .unwrap();
        assert!(!output.signals.is_empty(), "expected signals in range");
        let all_in_range = output
            .signals
            .iter()
            .all(|s| s.trade_date >= range_start && s.trade_date <= range_end);
        assert!(all_in_range, "all signals should be within the date range");
    }

    #[test]
    fn test_require_break_upper_filter() {
        let repo = MarketRepository::open_in_memory().unwrap();
        let mut quotes = Vec::new();
        let start = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        for i in 0..39 {
            let date = start + chrono::Duration::days(i);
            let date_str = date.format("%Y-%m-%d").to_string();
            quotes.push(make_quote("SQUEEZE_FLAT", &date_str, 100.0));
            quotes.push(make_quote("SQUEEZE_BREAK", &date_str, 100.0));
        }
        // 第 40 天：SQUEEZE_FLAT 續平盤；SQUEEZE_BREAK 突破上軌
        let date_40 = (start + chrono::Duration::days(39)).format("%Y-%m-%d").to_string();
        quotes.push(make_quote("SQUEEZE_FLAT", &date_40, 100.0));
        let mut break_quote = make_quote("SQUEEZE_BREAK", &date_40, 115.0);
        break_quote.open_price = Some(101.0);
        break_quote.close_price = Some(115.0);
        quotes.push(break_quote);

        repo.upsert_core_batch(&quotes).unwrap();

        let engine = BacktestEngine::new(&repo);
        let strategy_with_break = BollingerSqueezeBreakout {
            bandwidth_limit: 0.1,
            consecutive_days: 3,
            min_volume: 0,
            require_break_upper: true,
        };
        let output = engine.run_date(&date_40, &strategy_with_break, None).unwrap();
        let symbols: Vec<&str> = output.signals.iter().map(|s| s.symbol.as_str()).collect();
        assert!(!symbols.contains(&"SQUEEZE_FLAT"), "未突破上軌不應入選");
        assert!(symbols.contains(&"SQUEEZE_BREAK"), "突破上軌應入選");
    }
}
