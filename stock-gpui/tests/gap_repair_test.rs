use stock_gpui::state::CoreState;

#[tokio::test]
async fn test_core_state_diagnose_gaps() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    let state = CoreState::new_with_db_path(db_path).unwrap();

    // 診斷 2026-07-06(週一) ~ 2026-07-12(週日)
    let report = state
        .diagnose_gaps("2026-07-06", "2026-07-12")
        .await
        .unwrap();

    assert_eq!(report.start_date, "2026-07-06");
    assert_eq!(report.end_date, "2026-07-12");
    assert_eq!(report.total_workdays, 5);
    assert_eq!(report.recorded_dates, 0);
    assert_eq!(report.missing_dates.len(), 5);
    assert_eq!(report.missing_dates[0], "2026-07-06");
    assert_eq!(report.missing_dates[4], "2026-07-10");
}

#[tokio::test]
async fn test_core_state_repair_gaps_empty_list() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    let state = CoreState::new_with_db_path(db_path).unwrap();

    let progress_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let progress_called_clone = progress_called.clone();
    let res = state
        .repair_gaps(&[], move |_done, _total, _date| {
            progress_called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        })
        .await
        .unwrap();

    assert!(res.contains("沒有缺漏交易日需要修補"));
    assert!(!progress_called.load(std::sync::atomic::Ordering::SeqCst));
}

#[tokio::test]
async fn test_core_state_repair_gaps_cancellation() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    let state = CoreState::new_with_db_path(db_path).unwrap();
    let state_cancel = state.clone();

    let missing = vec!["2026-07-06".to_string(), "2026-07-07".to_string()];
    let res = state
        .repair_gaps(&missing, move |_done, _total, _date| {
            state_cancel.cancel();
        })
        .await;

    assert!(res.is_err());
    assert!(res.unwrap_err().contains("取消"));
}

#[gpui_kit::test]
fn test_download_view_gap_ui_headless(cx: &mut gpui_kit::TestAppContext) {
    use gpui_kit::AppContext;
    use gpui_kit::test::TestWindowExt;
    use gpui_kit::{Bounds, Point, TitlebarOptions, WindowBounds, WindowOptions, px, size};
    use stock_gpui::views::download::DownloadView;

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
                title: Some("下載與診斷測試".into()),
                ..Default::default()
            }),
            window_min_size: Some(size(px(960.0), px(640.0))),
            ..Default::default()
        };

        gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| DownloadView::new(core_state, window, cx))
        })
        .expect("開啟視窗失敗")
    });

    cx.update_window(window_handle, |_, window, cx| {
        window.draw(cx).clear(cx);
        let btn_diagnose = window.find("btn-diagnose-gaps");
        let btn_repair = window.find("btn-repair-gaps");
        assert!(btn_diagnose.visible());
        assert!(btn_repair.visible());
    })
    .unwrap();
}
