use std::collections::HashMap;

use crate::backtest::{Signal, Strategy};
use crate::models::OhlcvRecord;

/// 創波段新高突破策略
///
/// 判定條件：
/// 1. 當日價格（最高價或收盤價）>= 過去 N 個交易日之最高價
/// 2. 成交量高於門檻
#[derive(Debug, Clone)]
pub struct NewHighBreakoutStrategy {
    pub lookback_period: usize,
    pub use_high: bool,
    pub min_volume: i64,
}

impl Default for NewHighBreakoutStrategy {
    fn default() -> Self {
        Self {
            lookback_period: 20,
            use_high: true,
            min_volume: 0,
        }
    }
}

impl Strategy for NewHighBreakoutStrategy {
    fn id(&self) -> &str {
        "new_high"
    }

    fn min_history(&self) -> usize {
        self.lookback_period + 1
    }

    fn evaluate(&self, ohlcv: &[OhlcvRecord], date_index: usize) -> Signal {
        let no_signal = Signal {
            has_signal: false,
            direction: 0,
            strength: 0.0,
            metadata: HashMap::new(),
        };

        if date_index < self.lookback_period || ohlcv.is_empty() {
            return no_signal;
        }

        let curr_record = &ohlcv[date_index];
        if self.min_volume > 0 {
            match curr_record.volume_shares {
                Some(v) if v >= self.min_volume => {}
                _ => return no_signal,
            }
        }

        let curr_price = if self.use_high {
            curr_record.high_price.or(curr_record.close_price)
        } else {
            curr_record.close_price
        };

        let curr_val = match curr_price {
            Some(p) => p,
            None => return no_signal,
        };

        let lookback_slice = &ohlcv[(date_index - self.lookback_period)..date_index];
        let mut prior_max = f64::NEG_INFINITY;
        let mut count = 0;

        for r in lookback_slice {
            let p = if self.use_high {
                r.high_price.or(r.close_price)
            } else {
                r.close_price
            };
            if let Some(val) = p {
                if val > prior_max {
                    prior_max = val;
                }
                count += 1;
            }
        }

        if count < self.lookback_period || prior_max <= 0.0 {
            return no_signal;
        }

        if curr_val > prior_max {
            let mut metadata = HashMap::new();
            metadata.insert("lookback".to_string(), self.lookback_period as f64);
            metadata.insert("prior_high".to_string(), prior_max);
            metadata.insert("curr_price".to_string(), curr_val);

            let strength = (curr_val - prior_max) / prior_max;
            Signal {
                has_signal: true,
                direction: 1,
                strength,
                metadata,
            }
        } else {
            no_signal
        }
    }
}
