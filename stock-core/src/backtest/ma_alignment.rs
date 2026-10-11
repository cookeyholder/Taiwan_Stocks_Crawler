use std::collections::HashMap;

use crate::analysis;
use crate::backtest::{Signal, Strategy};
use crate::models::OhlcvRecord;

/// 均線多頭排列策略
///
/// 判定條件：
/// 1. 收盤價高於短期均線 (Close >= Short MA)
/// 2. 短期均線大於中期均線 (Short MA > Mid MA)
/// 3. 中期均線大於長期均線 (Mid MA > Long MA)
/// 4. 成交量高於門檻
#[derive(Debug, Clone)]
pub struct MaBullishAlignmentStrategy {
    pub short_period: usize,
    pub mid_period: usize,
    pub long_period: usize,
    pub min_volume: i64,
}

impl Default for MaBullishAlignmentStrategy {
    fn default() -> Self {
        Self {
            short_period: 5,
            mid_period: 10,
            long_period: 20,
            min_volume: 0,
        }
    }
}

impl Strategy for MaBullishAlignmentStrategy {
    fn id(&self) -> &str {
        "ma_alignment"
    }

    fn min_history(&self) -> usize {
        self.long_period
    }

    fn evaluate(&self, ohlcv: &[OhlcvRecord], date_index: usize) -> Signal {
        let no_signal = Signal {
            has_signal: false,
            direction: 0,
            strength: 0.0,
            metadata: HashMap::new(),
        };

        if date_index + 1 < self.long_period || ohlcv.is_empty() {
            return no_signal;
        }

        let curr_record = &ohlcv[date_index];
        if self.min_volume > 0 {
            match curr_record.volume_shares {
                Some(v) if v >= self.min_volume => {}
                _ => return no_signal,
            }
        }

        let ohlcv_slice = &ohlcv[..=date_index];
        let prices: Vec<f64> = ohlcv_slice.iter().filter_map(|r| r.close_price).collect();
        if prices.len() < self.long_period {
            return no_signal;
        }

        let short_sma = analysis::sma(&prices, self.short_period);
        let mid_sma = analysis::sma(&prices, self.mid_period);
        let long_sma = analysis::sma(&prices, self.long_period);

        let curr_close = curr_record.close_price;
        let s_val = short_sma.last().and_then(|v| *v);
        let m_val = mid_sma.last().and_then(|v| *v);
        let l_val = long_sma.last().and_then(|v| *v);

        match (curr_close, s_val, m_val, l_val) {
            (Some(c), Some(s), Some(m), Some(l)) if c >= s && s > m && m > l => {
                let mut metadata = HashMap::new();
                metadata.insert("ma_short".to_string(), s);
                metadata.insert("ma_mid".to_string(), m);
                metadata.insert("ma_long".to_string(), l);
                let strength = if l > 0.0 { (s - l) / l } else { 0.0 };
                Signal {
                    has_signal: true,
                    direction: 1,
                    strength,
                    metadata,
                }
            }
            _ => no_signal,
        }
    }
}
