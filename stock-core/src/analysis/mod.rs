use crate::models::{BollingerResult, MacdResult, RsiResult};

pub fn sma(values: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = vec![None; values.len()];
    if values.len() < period || period == 0 {
        return result;
    }
    let mut sum: f64 = values.iter().take(period).sum();
    result[period - 1] = Some(sum / period as f64);
    for i in period..values.len() {
        sum += values[i] - values[i - period];
        result[i] = Some(sum / period as f64);
    }
    result
}

pub fn ema(values: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = vec![None; values.len()];
    if values.len() < period || period == 0 {
        return result;
    }
    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema_val: f64 = values.iter().take(period).sum::<f64>() / period as f64;
    result[period - 1] = Some(ema_val);
    for i in period..values.len() {
        ema_val = (values[i] - ema_val) * multiplier + ema_val;
        result[i] = Some(ema_val);
    }
    result
}

pub fn bollinger(values: &[f64], period: usize, std_dev: f64) -> BollingerResult {
    let middle = sma(values, period);
    let len = values.len();
    let mut upper = vec![None; len];
    let mut lower = vec![None; len];
    for i in (period - 1)..len {
        if let Some(m) = middle[i] {
            let slice = &values[i + 1 - period..=i];
            let mean = slice.iter().sum::<f64>() / period as f64;
            let variance = slice.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / period as f64;
            let std = variance.sqrt();
            upper[i] = Some(m + std_dev * std);
            lower[i] = Some(m - std_dev * std);
        }
    }
    BollingerResult {
        upper,
        middle,
        lower,
    }
}

pub fn rsi(values: &[f64], period: usize) -> RsiResult {
    let mut result = vec![None; values.len()];
    if values.len() <= period || period == 0 {
        return RsiResult { values: result };
    }
    let mut gains = 0.0;
    let mut losses = 0.0;
    for i in 1..=period {
        let diff = values[i] - values[i - 1];
        if diff > 0.0 {
            gains += diff;
        } else {
            losses -= diff;
        }
    }
    let avg_gain = gains / period as f64;
    let avg_loss = losses / period as f64;
    if avg_loss == 0.0 {
        result[period] = Some(100.0);
    } else {
        let rs = avg_gain / avg_loss;
        result[period] = Some(100.0 - 100.0 / (1.0 + rs));
    }
    for i in (period + 1)..values.len() {
        let diff = values[i] - values[i - 1];
        let gain = if diff > 0.0 { diff } else { 0.0 };
        let loss = if diff < 0.0 { -diff } else { 0.0 };
        let avg_gain = (avg_gain * (period as f64 - 1.0) + gain) / period as f64;
        let avg_loss = (avg_loss * (period as f64 - 1.0) + loss) / period as f64;
        if avg_loss == 0.0 {
            result[i] = Some(100.0);
        } else {
            let rs = avg_gain / avg_loss;
            result[i] = Some(100.0 - 100.0 / (1.0 + rs));
        }
    }
    RsiResult { values: result }
}

pub fn macd(
    values: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> MacdResult {
    let fast_ema = ema(values, fast_period);
    let slow_ema = ema(values, slow_period);
    let len = values.len();
    let mut macd_line = vec![None; len];
    for i in 0..len {
        if let (Some(f), Some(s)) = (fast_ema[i], slow_ema[i]) {
            macd_line[i] = Some(f - s);
        }
    }
    let macd_values: Vec<f64> = macd_line.iter().filter_map(|v| *v).collect();
    let signal_ema = ema(&macd_values, signal_period);
    let mut signal_line = vec![None; len];
    let mut hist = vec![None; len];
    let macd_start = fast_period.max(slow_period) - 1;
    let mut signal_idx = 0;
    for i in macd_start..len {
        if let Some(m) = macd_line[i]
            && signal_idx < signal_ema.len()
        {
            if let Some(s) = signal_ema[signal_idx] {
                signal_line[i] = Some(s);
                hist[i] = Some(m - s);
            }
            signal_idx += 1;
        }
    }
    MacdResult {
        macd_line,
        signal_line,
        histogram: hist,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma() {
        let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = sma(&prices, 3);
        assert!(result[0].is_none());
        assert!(result[1].is_none());
        assert!((result[2].unwrap() - 2.0).abs() < 1e-10);
        assert!((result[3].unwrap() - 3.0).abs() < 1e-10);
        assert!((result[4].unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_ema() {
        let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let result = ema(&prices, 3);
        assert!(result[0].is_none());
        assert!(result[1].is_none());
        assert!((result[2].unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_bollinger() {
        let prices = (1..=20).map(|i| (i + 9) as f64).collect::<Vec<_>>();
        let result = bollinger(&prices, 5, 2.0);
        assert!(result.upper[4].is_some());
        assert!(result.middle[4].is_some());
        assert!(result.lower[4].is_some());
    }

    #[test]
    fn test_rsi() {
        let prices = vec![
            44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03,
            45.61, 46.28, 46.28, 46.00, 46.03, 46.41, 46.22, 46.21,
        ];
        let result = rsi(&prices, 14);
        assert!(result.values[0].is_none());
        assert!(result.values[14].is_some());
        let rsi_val = result.values[14].unwrap();
        assert!((0.0..=100.0).contains(&rsi_val));
    }

    #[test]
    fn test_rsi_extreme_values() {
        let prices = (1..=20).map(|i| (i + 9) as f64).collect::<Vec<_>>();
        let result = rsi(&prices, 5);
        assert!(result.values[5].is_some());
    }

    #[test]
    fn test_macd() {
        let prices = vec![
            10.0, 10.5, 11.0, 11.5, 12.0, 12.5, 13.0, 13.5, 14.0, 14.5, 15.0, 15.5, 16.0, 16.5,
            17.0, 17.5, 18.0, 18.5, 19.0, 19.5, 20.0, 20.5, 21.0, 21.5, 22.0, 22.5, 23.0, 23.5,
            24.0, 24.5,
        ];
        let result = macd(&prices, 12, 26, 9);
        assert!(!result.macd_line.is_empty());
        assert!(!result.signal_line.is_empty());
        assert!(!result.histogram.is_empty());
    }
}
