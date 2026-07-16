use crate::models::{PerformanceMetrics, TradeRecord};

pub fn calculate_metrics(
    trades: &[TradeRecord],
    initial_capital: f64,
    equity_curve: &[f64],
) -> PerformanceMetrics {
    let total_trades = trades.len();
    let winning_trades = trades.iter().filter(|t| t.pnl > 0.0).count();
    let losing_trades = trades.iter().filter(|t| t.pnl <= 0.0).count();
    let win_rate = if total_trades > 0 {
        winning_trades as f64 / total_trades as f64
    } else {
        0.0
    };

    let final_value = *equity_curve.last().unwrap_or(&initial_capital);
    let total_return_pct = if initial_capital > 0.0 {
        ((final_value - initial_capital) / initial_capital) * 100.0
    } else {
        0.0
    };

    let trading_days = equity_curve.len().max(1);
    let annualized_return_pct =
        ((1.0 + total_return_pct / 100.0).powf(252.0 / trading_days as f64) - 1.0) * 100.0;

    let mut daily_returns: Vec<f64> = Vec::new();
    for i in 1..equity_curve.len() {
        let prev = equity_curve[i - 1];
        let curr = equity_curve[i];
        if prev > 0.0 {
            daily_returns.push((curr - prev) / prev);
        }
    }

    let sharpe_ratio = if daily_returns.len() > 1 {
        let mean = daily_returns.iter().sum::<f64>() / daily_returns.len() as f64;
        let variance = daily_returns
            .iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>()
            / (daily_returns.len() - 1) as f64;
        let std = variance.sqrt();
        if std > 0.0 {
            (mean / std) * (252.0_f64).sqrt()
        } else {
            0.0
        }
    } else {
        0.0
    };

    let max_drawdown_pct = if equity_curve.len() > 1 {
        let mut peak = equity_curve[0];
        let mut max_dd = 0.0_f64;
        for &v in equity_curve {
            if v > peak {
                peak = v;
            }
            let dd = (peak - v) / peak * 100.0;
            if dd > max_dd {
                max_dd = dd;
            }
        }
        max_dd
    } else {
        0.0
    };

    PerformanceMetrics {
        total_return_pct,
        annualized_return_pct,
        win_rate,
        total_trades,
        winning_trades,
        losing_trades,
        sharpe_ratio,
        max_drawdown_pct,
        final_value,
    }
}
