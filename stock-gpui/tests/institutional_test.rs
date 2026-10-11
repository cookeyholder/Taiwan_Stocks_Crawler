use std::collections::HashMap;
use stock_core::models::{CanonicalQuote, InstitutionalRecord};
use stock_gpui::state::CoreState;

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
        high_price: Some(close + 10.0),
        low_price: Some(close - 5.0),
        close_price: Some(close),
        volume_shares: Some(10_000_000),
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
        ingested_at: "2026-07-06T00:00:00".to_string(),
        extension: HashMap::new(),
    }
}

#[tokio::test]
async fn test_core_state_institutional_data_alignment() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    let state = CoreState::new_with_db_path(db_path).unwrap();

    // 1. 寫入行情記錄 (3 天: 2026-07-06, 2026-07-07, 2026-07-08)
    let quotes = vec![
        make_quote("2330", "2026-07-06", 1000.0),
        make_quote("2330", "2026-07-07", 1010.0),
        make_quote("2330", "2026-07-08", 1020.0),
    ];

    {
        let p = state.pipeline.lock().await;
        p.repo.upsert_core_batch(&quotes).unwrap();
    }

    // 2. 寫入法人記錄 (僅 2026-07-06 與 2026-07-08，故意缺漏 2026-07-07)
    let inst_records = vec![
        InstitutionalRecord {
            trade_date: "2026-07-06".to_string(),
            symbol: "2330".to_string(),
            foreign_buy: 10_000_000,
            foreign_sell: 8_000_000,
            foreign_net: 2_000_000,
            trust_buy: 1_000_000,
            trust_sell: 500_000,
            trust_net: 500_000,
            dealer_buy: 500_000,
            dealer_sell: 100_000,
            dealer_net: 400_000,
            total_net: 2_900_000,
        },
        InstitutionalRecord {
            trade_date: "2026-07-08".to_string(),
            symbol: "2330".to_string(),
            foreign_buy: 5_000_000,
            foreign_sell: 7_000_000,
            foreign_net: -2_000_000,
            trust_buy: 2_000_000,
            trust_sell: 1_000_000,
            trust_net: 1_000_000,
            dealer_buy: 300_000,
            dealer_sell: 500_000,
            dealer_net: -200_000,
            total_net: -1_200_000,
        },
    ];

    {
        let p = state.pipeline.lock().await;
        p.repo.upsert_institutional_batch(&inst_records).unwrap();
    }

    // 3. 取得走勢分析資料
    let analysis = state
        .get_analysis("2330", &[5], None, None, None, None)
        .await
        .unwrap();

    assert_eq!(analysis.ohlcv.len(), 3);
    assert_eq!(analysis.institutional.len(), 3);

    // 驗證 2026-07-06 資料對齊
    let day1 = analysis.institutional[0]
        .as_ref()
        .expect("第一天應有法人記錄");
    assert_eq!(day1.trade_date, "2026-07-06");
    assert_eq!(day1.foreign_net, 2_000_000);
    assert_eq!(day1.total_net, 2_900_000);

    // 驗證 2026-07-07 資料缺漏對齊為 None
    assert!(analysis.institutional[1].is_none());

    // 驗證 2026-07-08 資料對齊
    let day3 = analysis.institutional[2]
        .as_ref()
        .expect("第三天應有法人記錄");
    assert_eq!(day3.trade_date, "2026-07-08");
    assert_eq!(day3.foreign_net, -2_000_000);
    assert_eq!(day3.total_net, -1_200_000);
}

#[gpui_kit::test]
fn test_analysis_view_institutional_ui_headless(cx: &mut gpui_kit::TestAppContext) {
    use gpui_kit::AppContext;
    use gpui_kit::{Bounds, Point, TitlebarOptions, WindowBounds, WindowOptions, px, size};
    use stock_gpui::views::analysis::AnalysisView;

    cx.update(gpui_kit::init);

    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();
    let core_state = CoreState::new_with_db_path(db_path).unwrap();

    let (_window_handle, view) = cx.update(|cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(1200.), px(800.)),
            })),
            titlebar: Some(TitlebarOptions {
                title: Some("法人分析副圖測試".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| AnalysisView::new(core_state, window, cx))
        })
        .expect("開啟視窗失敗")
    });

    // 驗證 show_institutional 預設為 true
    view.update(cx, |this, _cx| {
        assert!(this.show_institutional);
        this.show_institutional = false;
        assert!(!this.show_institutional);
        this.show_institutional = true;
        assert!(this.show_institutional);
    });
}
