use chrono::NaiveDate;
use std::collections::HashMap;
use stock_core::models::CanonicalQuote;
use stock_gpui::state::{CoreState, ScreenerMode, ScreenerParams};

fn make_quote(
    symbol: &str,
    name: &str,
    date: &str,
    close: f64,
    high: f64,
    vol: i64,
) -> CanonicalQuote {
    CanonicalQuote {
        trade_date: date.to_string(),
        market_code: "TWSE".to_string(),
        symbol_raw: symbol.to_string(),
        symbol_clean: symbol.to_string(),
        security_name: Some(name.to_string()),
        instrument_type: "stock".to_string(),
        instrument_subtype: None,
        isin_code: None,
        cfi_code: None,
        section_label: None,
        open_price: Some(close),
        high_price: Some(high),
        low_price: Some(close * 0.99),
        close_price: Some(close),
        volume_shares: Some(vol),
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

#[tokio::test]
async fn test_composite_screener_filters_and_tags() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();
    let state = CoreState::new_with_db_path(db_path).unwrap();

    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let mut quotes = Vec::new();

    // 建立 30 天數據
    // STOCK_A: 每日穩定上漲，從 100 到 160，既符合多頭排列也創 20 日新高
    // STOCK_B: 每日下跌，從 160 到 100，不符合多頭排列，亦未創新高
    // STOCK_C: 前 5 天為 200 高點，後面在 100 附近小幅上漲，符合多頭排列但未創 20 日新高
    for i in 0..30 {
        let date = (start + chrono::Duration::days(i))
            .format("%Y-%m-%d")
            .to_string();

        let a_price = 100.0 + (i as f64) * 2.0;
        quotes.push(make_quote(
            "2330",
            "台積電",
            &date,
            a_price,
            a_price + 2.0,
            1000,
        ));

        let b_price = 160.0 - (i as f64) * 2.0;
        quotes.push(make_quote(
            "2317",
            "鴻海",
            &date,
            b_price,
            b_price + 1.0,
            1000,
        ));

        // STOCK_C: 在 i = 15 有 500.0 高點，但在 25..29 時持續走多（符合多頭排列但未創 20 日新高）
        let c_price = if i == 15 {
            500.0
        } else {
            80.0 + (i as f64) * 2.0
        };
        quotes.push(make_quote(
            "2454",
            "聯發科",
            &date,
            c_price,
            c_price + 1.0,
            1000,
        ));
    }

    {
        let pipeline = state.pipeline.lock().await;
        pipeline.repo.upsert_core_batch(&quotes).unwrap();
    }

    let query_start = (start + chrono::Duration::days(25))
        .format("%Y-%m-%d")
        .to_string();
    let query_end = (start + chrono::Duration::days(29))
        .format("%Y-%m-%d")
        .to_string();

    // 1. 測試複合選股（多頭排列 + 創波段新高）
    let params_composite = ScreenerParams {
        mode: ScreenerMode::Composite,
        short_period: 3,
        mid_period: 5,
        long_period: 10,
        lookback_period: 20,
        min_volume: 500,
        ..Default::default()
    };
    let res = state
        .screen_composite_range(
            &query_start,
            &query_end,
            params_composite,
            |_done, _total, _msg, _fail| {},
        )
        .await
        .unwrap();

    let symbols: Vec<&str> = res.results.iter().map(|r| r.symbol.as_str()).collect();
    assert!(symbols.contains(&"2330"), "STOCK_A 應符合複合選股條件");
    assert!(!symbols.contains(&"2317"), "STOCK_B 不應入選");
    assert!(
        !symbols.contains(&"2454"),
        "STOCK_C 未創新高不應入選複合條件"
    );

    let a_result = res.results.iter().find(|r| r.symbol == "2330").unwrap();
    assert_eq!(
        a_result.tags,
        vec!["多頭排列".to_string(), "創20日新高".to_string()]
    );

    // 2. 測試僅均線多頭排列
    let params_ma = ScreenerParams {
        mode: ScreenerMode::MaAlignment,
        short_period: 3,
        mid_period: 5,
        long_period: 10,
        min_volume: 500,
        ..Default::default()
    };
    let res_ma = state
        .screen_composite_range(
            &query_start,
            &query_end,
            params_ma,
            |_done, _total, _msg, _fail| {},
        )
        .await
        .unwrap();

    let ma_symbols: Vec<&str> = res_ma.results.iter().map(|r| r.symbol.as_str()).collect();
    assert!(ma_symbols.contains(&"2330"));
    assert!(ma_symbols.contains(&"2454"), "STOCK_C 應符合均線多頭排列");
    assert!(!ma_symbols.contains(&"2317"));

    let c_result = res_ma.results.iter().find(|r| r.symbol == "2454").unwrap();
    assert_eq!(c_result.tags, vec!["多頭排列".to_string()]);
}

#[gpui_kit::test]
fn test_screener_view_mode_switching_headless(cx: &mut gpui_kit::TestAppContext) {
    use gpui_kit::AppContext;
    use gpui_kit::test::TestWindowExt;
    use gpui_kit::{Bounds, Point, TitlebarOptions, WindowBounds, WindowOptions, px, size};
    use stock_gpui::views::screener::ScreenerView;

    cx.update(gpui_kit::init);

    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();
    let core_state = CoreState::new_with_db_path(db_path).unwrap();

    let (window_handle, screener_entity) = cx.update(|cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(1280.0), px(800.0)),
            })),
            titlebar: Some(TitlebarOptions {
                title: Some("選股器模式測試".into()),
                ..Default::default()
            }),
            window_min_size: Some(size(px(960.0), px(640.0))),
            ..Default::default()
        };

        gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| ScreenerView::new(core_state, window, cx))
        })
        .expect("開啟視窗失敗")
    });

    cx.update_window(window_handle, |_, window, cx| {
        window.draw(cx).clear(cx);
        let btn_bollinger = window.find("btn-mode-bollinger");
        let btn_ma = window.find("btn-mode-ma");
        let btn_nh = window.find("btn-mode-new-high");
        let btn_composite = window.find("btn-mode-composite");
        assert!(btn_bollinger.visible());
        assert!(btn_ma.visible());
        assert!(btn_nh.visible());
        assert!(btn_composite.visible());
    })
    .unwrap();

    // 切換為複合選股模式
    screener_entity.update(cx, |view, cx| {
        view.mode = ScreenerMode::Composite;
        cx.notify();
    });

    cx.update_window(window_handle, |_, window, cx| {
        window.draw(cx).clear(cx);
        let short_ma = window.find("screen-short-ma");
        let lookback = window.find("screen-lookback");
        assert!(short_ma.visible(), "複合模式下短均輸入框應可見");
        assert!(lookback.visible(), "複合模式下波段突破天數輸入框應可見");
    })
    .unwrap();

    // 驗證表格欄位
    screener_entity.update(cx, |view, cx| {
        let table = view.table_state.read(cx);
        let delegate = table.delegate();
        assert_eq!(delegate.results.len(), 0);
    });
}
