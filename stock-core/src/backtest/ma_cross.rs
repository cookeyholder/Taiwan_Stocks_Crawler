use std::collections::HashMap;

use crate::analysis;
use crate::backtest::{Signal, Strategy};
use crate::models::OhlcvRecord;

pub struct MaCrossStrategy {
    pub fast_period: usize,
    pub slow_period: usize,
    pub min_volume: i64,
}

impl Strategy for MaCrossStrategy {
    fn id(&self) -> &str {
        "ma_cross"
    }

    fn min_history(&self) -> usize {
        self.slow_period
    }

    fn evaluate(&self, ohlcv: &[OhlcvRecord], date_index: usize) -> Signal {
        let ohlcv_slice = &ohlcv[..=date_index];
        let prices: Vec<f64> = ohlcv_slice.iter().filter_map(|r| r.close_price).collect();
        if prices.len() < self.slow_period + 1 {
            return Signal {
                has_signal: false,
                direction: 0,
                strength: 0.0,
                metadata: HashMap::new(),
            };
        }

        if self.min_volume > 0 {
            match ohlcv[date_index].volume_shares {
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

        let fast = analysis::sma(&prices, self.fast_period);
        let slow = analysis::sma(&prices, self.slow_period);

        let prev_fast = fast[date_index - 1];
        let prev_slow = slow[date_index - 1];
        let curr_fast = fast[date_index];
        let curr_slow = slow[date_index];

        match (prev_fast, prev_slow, curr_fast, curr_slow) {
            (Some(pf), Some(ps), Some(cf), Some(cs)) if pf <= ps && cf > cs => {
                let mut metadata = HashMap::new();
                metadata.insert("fast_sma".to_string(), cf);
                metadata.insert("slow_sma".to_string(), cs);
                Signal {
                    has_signal: true,
                    direction: 1,
                    strength: (cf - cs) / cs,
                    metadata,
                }
            }
            (Some(pf), Some(ps), Some(cf), Some(cs)) if pf >= ps && cf < cs => {
                let mut metadata = HashMap::new();
                metadata.insert("fast_sma".to_string(), cf);
                metadata.insert("slow_sma".to_string(), cs);
                Signal {
                    has_signal: true,
                    direction: -1,
                    strength: (cs - cf) / cs,
                    metadata,
                }
            }
            _ => Signal {
                has_signal: false,
                direction: 0,
                strength: 0.0,
                metadata: HashMap::new(),
            },
        }
    }
}
