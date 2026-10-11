use stock_gpui::state::CoreState;

#[tokio::test]
async fn test_core_state_watchlist_toggle_and_cache() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_watchlist.db");
    let db_path_str = db_path.to_str().unwrap();

    let state = CoreState::new_with_db_path(db_path_str).unwrap();

    // 初始狀態無自選股
    assert!(!state.is_favorited("2330"));
    assert_eq!(state.get_watchlist_symbols().len(), 0);

    // 加入自選股
    let added = state.toggle_watchlist("2330", None).await.unwrap();
    assert!(added);
    assert!(state.is_favorited("2330"));
    assert!(state.get_watchlist_symbols().contains("2330"));

    // 再次切換則移出自選股
    let removed = state.toggle_watchlist("2330", None).await.unwrap();
    assert!(!removed);
    assert!(!state.is_favorited("2330"));
    assert!(!state.get_watchlist_symbols().contains("2330"));

    // 加入自選股至特定群組
    let added_group = state
        .toggle_watchlist("2454", Some("半導體"))
        .await
        .unwrap();
    assert!(added_group);
    assert!(state.is_favorited("2454"));

    // 重新開啟資料庫，驗證持久化載入
    let state_reloaded = CoreState::new_with_db_path(db_path_str).unwrap();
    assert!(state_reloaded.is_favorited("2454"));
    assert!(!state_reloaded.is_favorited("2330"));
    assert_eq!(state_reloaded.get_watchlist_symbols().len(), 1);
}

#[test]
fn test_analysis_delegate_watchlist_filtering() {
    use stock_gpui::state::SecurityItem;
    use stock_gpui::views::analysis::AnalysisSecuritiesDelegate;

    let mut delegate = AnalysisSecuritiesDelegate::new();
    delegate.all_items = vec![
        SecurityItem {
            symbol: "2330".into(),
            market: "TWSE".into(),
            name: "台積電".into(),
        },
        SecurityItem {
            symbol: "2454".into(),
            market: "TWSE".into(),
            name: "聯發科".into(),
        },
        SecurityItem {
            symbol: "2317".into(),
            market: "TWSE".into(),
            name: "鴻海".into(),
        },
    ];

    delegate.apply_filter();
    assert_eq!(delegate.filtered_items.len(), 3);

    // 加入 2330 至自選股
    delegate.watchlist.insert("2330".into());
    delegate.show_watchlist_only = true;
    delegate.apply_filter();

    assert_eq!(delegate.filtered_items.len(), 1);
    assert_eq!(delegate.filtered_items[0].symbol, "2330");

    // 搜尋不在自選清單中的項目
    delegate.search_query = "聯發科".into();
    delegate.apply_filter();
    assert_eq!(delegate.filtered_items.len(), 0);

    // 切換回全部
    delegate.show_watchlist_only = false;
    delegate.apply_filter();
    assert_eq!(delegate.filtered_items.len(), 1);
    assert_eq!(delegate.filtered_items[0].symbol, "2454");
}

#[gpui_kit::test]
fn test_analysis_view_watchlist_headless(cx: &mut gpui_kit::TestAppContext) {
    use gpui_kit::AppContext;
    use gpui_kit::test::TestWindowExt;
    use gpui_kit::{Bounds, Point, TitlebarOptions, WindowBounds, WindowOptions, px, size};
    use stock_gpui::views::analysis::AnalysisView;

    cx.update(gpui_kit::init);

    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();
    let core_state = CoreState::new_with_db_path(db_path).unwrap();

    let (window_handle, _view) = cx.update(|cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(1280.0), px(800.0)),
            })),
            titlebar: Some(TitlebarOptions {
                title: Some("分析測試".into()),
                ..Default::default()
            }),
            window_min_size: Some(size(px(960.0), px(640.0))),
            ..Default::default()
        };

        gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| AnalysisView::new(core_state, window, cx))
        })
        .expect("開啟視窗失敗")
    });

    cx.update_window(window_handle, |_, window, cx| {
        window.draw(cx).clear(cx);
        let tab_all = window.find("tab-filter-all");
        let tab_watchlist = window.find("tab-filter-watchlist");
        assert!(tab_all.visible());
        assert!(tab_watchlist.visible());
    })
    .unwrap();
}
