use stock_core::backtest::ma_alignment::MaBullishAlignmentStrategy;
use stock_core::backtest::{Strategy, StrategyKind};
use stock_core::models::OhlcvRecord;

fn make_ohlcv(_symbol: &str, dates_and_closes: &[(&str, f64)], volume: i64) -> Vec<OhlcvRecord> {
    dates_and_closes
        .iter()
        .map(|(d, c)| OhlcvRecord {
            trade_date: d.to_string(),
            open_price: Some(*c),
            high_price: Some(*c * 1.02),
            low_price: Some(*c * 0.98),
            close_price: Some(*c),
            volume_shares: Some(volume),
        })
        .collect()
}

#[test]
fn test_insufficient_history_returns_no_signal() {
    let strategy = MaBullishAlignmentStrategy::default();
    let records = make_ohlcv(
        "2330",
        &[("2024-01-01", 100.0), ("2024-01-02", 101.0)],
        1000,
    );
    let signal = strategy.evaluate(&records, 1);
    assert!(!signal.has_signal);
}

#[test]
fn test_bullish_alignment_detected() {
    let strategy = MaBullishAlignmentStrategy {
        short_period: 3,
        mid_period: 5,
        long_period: 10,
        min_volume: 500,
    };

    // 建立穩定上漲數列：從 10 上升到 30
    let mut data = Vec::new();
    for i in 1..=15 {
        let date = format!("2024-01-{:02}", i);
        let close = 10.0 + (i as f64) * 2.0;
        data.push((date, close));
    }
    let data_refs: Vec<(&str, f64)> = data.iter().map(|(d, c)| (d.as_str(), *c)).collect();
    let records = make_ohlcv("2330", &data_refs, 1000);

    let signal = strategy.evaluate(&records, 14);
    assert!(signal.has_signal, "持續上漲應符合均線多頭排列");
    assert_eq!(signal.direction, 1);
    let ma_short = *signal.metadata.get("ma_short").unwrap();
    let ma_mid = *signal.metadata.get("ma_mid").unwrap();
    let ma_long = *signal.metadata.get("ma_long").unwrap();
    assert!(ma_short > ma_mid);
    assert!(ma_mid > ma_long);
}

#[test]
fn test_bearish_or_entangled_returns_no_signal() {
    let strategy = MaBullishAlignmentStrategy {
        short_period: 3,
        mid_period: 5,
        long_period: 10,
        min_volume: 0,
    };

    // 建立下跌數列：從 50 下跌到 20
    let mut data = Vec::new();
    for i in 1..=15 {
        let date = format!("2024-01-{:02}", i);
        let close = 50.0 - (i as f64) * 2.0;
        data.push((date, close));
    }
    let data_refs: Vec<(&str, f64)> = data.iter().map(|(d, c)| (d.as_str(), *c)).collect();
    let records = make_ohlcv("2330", &data_refs, 1000);

    let signal = strategy.evaluate(&records, 14);
    assert!(!signal.has_signal, "空頭排列不應產生買進訊號");
}

#[test]
fn test_min_volume_filter() {
    let strategy = MaBullishAlignmentStrategy {
        short_period: 3,
        mid_period: 5,
        long_period: 10,
        min_volume: 5000,
    };

    let mut data = Vec::new();
    for i in 1..=15 {
        let date = format!("2024-01-{:02}", i);
        let close = 10.0 + (i as f64) * 2.0;
        data.push((date, close));
    }
    let data_refs: Vec<(&str, f64)> = data.iter().map(|(d, c)| (d.as_str(), *c)).collect();
    let records = make_ohlcv("2330", &data_refs, 1000); // 1000 < min_volume 5000

    let signal = strategy.evaluate(&records, 14);
    assert!(!signal.has_signal, "成交量不足時不應產生訊號");
}

#[test]
fn test_strategy_factory_creation() {
    let params = vec![
        ("short_period".to_string(), 5.0),
        ("mid_period".to_string(), 10.0),
        ("long_period".to_string(), 20.0),
        ("min_volume".to_string(), 1000.0),
    ];
    let kind = StrategyKind::create("ma_alignment", &params).unwrap();
    assert_eq!(kind.id(), "ma_alignment");
    assert_eq!(kind.min_history(), 20);
}

#[test]
fn test_screen_result_backward_compatibility() {
    let legacy_json = r#"{
        "symbol": "2330",
        "name": "台積電",
        "trade_date": "2024-01-02",
        "latest_close": 590.0,
        "latest_volume": 12000,
        "max_bandwidth": 0.05,
        "latest_bandwidth": 0.03,
        "latest_upper_band": 600.0,
        "break_upper": true
    }"#;
    let res: stock_core::models::ScreenResult = serde_json::from_str(legacy_json).unwrap();
    assert_eq!(res.symbol, "2330");
    assert!(res.tags.is_empty(), "舊版無 tags 時應預設為空向量");
}
