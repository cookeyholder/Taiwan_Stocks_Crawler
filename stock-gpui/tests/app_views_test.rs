use gpui_kit::AppContext;
use gpui_kit::test::TestWindowExt;
use gpui_kit::{
    Bounds, Point, TestAppContext, TitlebarOptions, WindowBounds, WindowOptions, px, size,
};
use stock_gpui::{AppView, CoreState, NavTab};

#[gpui_kit::test]
fn test_app_view_tabs_and_theme_switching(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);

    // 建立臨時記憶體資料庫供測試
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    let core_state = CoreState::new_with_db_path(db_path).unwrap();

    let (window_handle, app_entity) = cx.update(|cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(1280.0), px(800.0)),
            })),
            titlebar: Some(TitlebarOptions {
                title: Some("台股布林通道擠壓選股系統".into()),
                ..Default::default()
            }),
            window_min_size: Some(size(px(960.0), px(640.0))),
            ..Default::default()
        };

        gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| AppView::new(core_state, window, cx))
        })
        .expect("開啟視窗失敗")
    });

    cx.update_window(window_handle.into(), |_, window, cx| {
        window.draw(cx).clear(cx);
        let title = window.find("app-title");
        assert!(title.visible());
    })
    .unwrap();

    // 驗證分頁切換
    app_entity.update(cx, |app, cx| {
        assert_eq!(app.active_tab, NavTab::Analysis);

        app.set_tab(NavTab::Screener, cx);
        assert_eq!(app.active_tab, NavTab::Screener);

        app.set_tab(NavTab::Backtest, cx);
        assert_eq!(app.active_tab, NavTab::Backtest);

        app.set_tab(NavTab::Download, cx);
        assert_eq!(app.active_tab, NavTab::Download);

        app.set_tab(NavTab::Securities, cx);
        assert_eq!(app.active_tab, NavTab::Securities);

        // 驗證主題切換（預設為淺色模式）
        assert!(!app.is_dark);
        app.toggle_theme(cx);
        assert!(app.is_dark);
        app.toggle_theme(cx);
        assert!(!app.is_dark);

        // 驗證字體大小調整
        assert_eq!(app.font_size, stock_gpui::app::FontSizeLevel::Medium);
        app.cycle_font_size(cx);
        assert_eq!(app.font_size, stock_gpui::app::FontSizeLevel::Large);

        // 驗證優雅退出
        app.shutdown();
        assert!(
            app.core_state
                .cancel_flag
                .load(std::sync::atomic::Ordering::SeqCst)
        );
    });
}
