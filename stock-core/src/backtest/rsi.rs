use std::collections::HashMap;

use crate::analysis;
use crate::backtest::{Signal, Strategy};
use crate::models::OhlcvRecord;

pub struct RsiStrategy {
    pub period: usize,
    pub overbought: f64,
    pub oversold: f64,
    pub min_volume: i64,
}

impl Strategy for RsiStrategy {
    fn id(&self) -> &str {
        "rsi"
    }

    fn min_history(&self) -> usize {
        self.period + 1
    }

    fn evaluate(&self, ohlcv: &[OhlcvRecord], date_index: usize) -> Signal {
        let ohlcv_slice = &ohlcv[..=date_index];
        let prices: Vec<f64> = ohlcv_slice.iter().filter_map(|r| r.close_price).collect();
        if prices.len() < self.period + 2 {
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

        let rsi_values = analysis::rsi(&prices, self.period);
        let curr_rsi = rsi_values.values[date_index];
        let prev_rsi = rsi_values.values[date_index - 1];

        match (prev_rsi, curr_rsi) {
            (Some(pr), Some(cr)) if pr <= self.oversold && cr > self.oversold => {
                let mut metadata = HashMap::new();
                metadata.insert("rsi".to_string(), cr);
                Signal {
                    has_signal: true,
                    direction: 1,
                    strength: (self.oversold - cr).abs() / self.oversold,
                    metadata,
                }
            }
            (Some(pr), Some(cr)) if pr >= self.overbought && cr < self.overbought => {
                let mut metadata = HashMap::new();
                metadata.insert("rsi".to_string(), cr);
                Signal {
                    has_signal: true,
                    direction: -1,
                    strength: (cr - self.overbought).abs() / (100.0 - self.overbought),
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
