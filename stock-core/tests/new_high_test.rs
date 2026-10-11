use stock_core::backtest::new_high::NewHighBreakoutStrategy;
use stock_core::backtest::{Strategy, StrategyKind};
use stock_core::models::OhlcvRecord;

fn make_records(highs: &[f64], volume: i64) -> Vec<OhlcvRecord> {
    highs
        .iter()
        .enumerate()
        .map(|(i, &h)| OhlcvRecord {
            trade_date: format!("2024-01-{:02}", i + 1),
            open_price: Some(h - 1.0),
            high_price: Some(h),
            low_price: Some(h - 2.0),
            close_price: Some(h - 0.5),
            volume_shares: Some(volume),
        })
        .collect()
}

#[test]
fn test_insufficient_history_returns_no_signal() {
    let strategy = NewHighBreakoutStrategy {
        lookback_period: 20,
        use_high: true,
        min_volume: 0,
    };
    let records = make_records(&[10.0; 15], 1000);
    let signal = strategy.evaluate(&records, 14);
    assert!(!signal.has_signal);
}

#[test]
fn test_breakout_20_day_high() {
    let strategy = NewHighBreakoutStrategy {
        lookback_period: 20,
        use_high: true,
        min_volume: 500,
    };

    // 建立 20 天最高價均為 100.0，第 21 天（index 20）高點來到 105.0
    let mut highs = vec![100.0; 20];
    highs.push(105.0);
    let records = make_records(&highs, 1000);

    let signal = strategy.evaluate(&records, 20);
    assert!(signal.has_signal, "創 20 日新高應產生訊號");
    assert_eq!(signal.direction, 1);
    assert_eq!(*signal.metadata.get("lookback").unwrap(), 20.0);
    assert_eq!(*signal.metadata.get("prior_high").unwrap(), 100.0);
    assert_eq!(*signal.metadata.get("curr_price").unwrap(), 105.0);
}

#[test]
fn test_not_breaking_high_returns_no_signal() {
    let strategy = NewHighBreakoutStrategy {
        lookback_period: 20,
        use_high: true,
        min_volume: 0,
    };

    // 前 20 天最高價最高有 120.0，第 21 天僅有 110.0
    let mut highs = vec![100.0; 19];
    highs.push(120.0);
    highs.push(110.0);
    let records = make_records(&highs, 1000);

    let signal = strategy.evaluate(&records, 20);
    assert!(!signal.has_signal, "未突破波段高點不應產生訊號");
}

#[test]
fn test_new_high_volume_filter() {
    let strategy = NewHighBreakoutStrategy {
        lookback_period: 20,
        use_high: true,
        min_volume: 5000,
    };

    let mut highs = vec![100.0; 20];
    highs.push(105.0);
    let records = make_records(&highs, 1000); // 1000 < min_volume 5000

    let signal = strategy.evaluate(&records, 20);
    assert!(!signal.has_signal, "成交量不足時不應產生訊號");
}

#[test]
fn test_strategy_factory_creation() {
    let params = vec![
        ("lookback_period".to_string(), 60.0),
        ("use_high".to_string(), 1.0),
        ("min_volume".to_string(), 500.0),
    ];
    let kind = StrategyKind::create("new_high", &params).unwrap();
    assert_eq!(kind.id(), "new_high");
    assert_eq!(kind.min_history(), 61);
}
