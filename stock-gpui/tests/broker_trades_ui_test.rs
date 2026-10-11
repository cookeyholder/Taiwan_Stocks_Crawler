use gpui_kit::AppContext;
use gpui_kit::test::TestWindowExt;
use gpui_kit::{
    Bounds, Point, TestAppContext, TitlebarOptions, WindowBounds, WindowOptions, px, size,
};
use stock_gpui::state::CoreState;
use stock_gpui::views::download::DownloadView;

#[gpui_kit::test]
fn test_download_view_broker_trades_ui_elements(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);

    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    let core_state = CoreState::new_with_db_path(db_path).unwrap();

    let (window_handle, download_entity) = cx.update(|cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(1280.0), px(800.0)),
            })),
            titlebar: Some(TitlebarOptions {
                title: Some("下載管理測試".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| DownloadView::new(core_state, window, cx))
        })
        .expect("開啟測試視窗失敗")
    });

    cx.update_window(window_handle, |_, window, cx| {
        window.draw(cx).clear(cx);

        // 驗證存在券商日報表交易日輸入框
        let date_input = window.find("broker-dl-date");
        assert!(date_input.visible(), "券商日報表日期輸入框應可見");

        // 驗證存在「開始 / 繼續下載」按鈕
        let btn_download = window.find("btn-start-broker-download");
        assert!(btn_download.visible(), "開始下載按鈕應可見");
    })
    .unwrap();

    // 驗證實體內部初始狀態
    download_entity.update(cx, |view, _| {
        assert!(!view.is_broker_downloading);
        assert_eq!(view.broker_progress_ratio, 0.0);
        assert!(view.broker_status_text.contains("就緒"));
    });
}
