use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::chart::{BarChart, CandlestickChart};
use gpui_kit::test::TestWindowExt;
use gpui_kit::{
    AppContext, Bounds, Context, InteractiveElement as _, IntoElement, ParentElement as _, Point,
    Render, Styled as _, TestAppContext, TitlebarOptions, Window, WindowBounds, WindowOptions, div,
    hsla, px, size,
};

#[derive(Clone, Debug)]
pub struct StockCandle {
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

struct ChartView {
    data: Vec<StockCandle>,
}

impl Render for ChartView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let kline = CandlestickChart::new(self.data.clone())
            .id("candlestick-chart")
            .x(|d: &StockCandle| d.date.clone())
            .open(|d: &StockCandle| d.open)
            .high(|d: &StockCandle| d.high)
            .low(|d: &StockCandle| d.low)
            .close(|d: &StockCandle| d.close)
            .bullish(hsla(0.0, 0.8, 0.5, 1.0)) // 台股紅漲
            .bearish(hsla(0.33, 0.8, 0.4, 1.0)) // 台股綠跌
            .interactive(true);

        let volume_chart = BarChart::new(self.data.clone())
            .id("volume-chart")
            .band(|d: &StockCandle| d.date.clone())
            .value(|d: &StockCandle| d.volume)
            .interactive(true);

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .id("chart-container")
                    .test_support()
                    .size_full()
                    .child(kline),
            )
            .child(div().h(px(150.)).child(volume_chart))
    }
}

#[gpui_kit::test]
fn test_candlestick_and_volume_chart_rendering(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);

    // 建立 30 筆日 K 線資料
    let mut candles = Vec::new();
    for i in 1..=30 {
        let base = 500.0 + (i as f64) * 2.0;
        candles.push(StockCandle {
            date: format!("2026-03-{:02}", i),
            open: base,
            high: base + 8.0,
            low: base - 5.0,
            close: base + 4.0,
            volume: 10000.0 + (i as f64) * 500.0,
        });
    }

    let (window_handle, _view_entity) = cx.update(|cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(1280.0), px(800.0)),
            })),
            titlebar: Some(TitlebarOptions {
                title: Some("日 K 線圖表測試".into()),
                ..Default::default()
            }),
            window_min_size: Some(size(px(960.0), px(640.0))),
            ..Default::default()
        };

        gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| ChartView { data: candles }))
            .expect("開啟視窗失敗")
    });

    cx.update_window(window_handle, |_, window, cx| {
        window.draw(cx).clear(cx);
        let container = window.find("chart-container");
        assert!(container.visible());
    })
    .unwrap();
}

#[gpui_kit::test]
fn test_bollinger_canvas_overlay(cx: &mut TestAppContext) {
    use gpui_kit::canvas;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    cx.update(gpui_kit::init);

    let painted_flag = Arc::new(AtomicBool::new(false));
    let painted_clone = painted_flag.clone();

    struct OverlayTestView {
        flag: Arc<AtomicBool>,
    }

    impl Render for OverlayTestView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let flag = self.flag.clone();
            let c = canvas(
                move |_, _, _| {},
                move |bounds, _, _, _| {
                    if bounds.size.width.as_f32() > 10.0 && bounds.size.height.as_f32() > 10.0 {
                        flag.store(true, Ordering::SeqCst);
                    }
                },
            )
            .size_full();

            div()
                .size_full()
                .relative()
                .child(div().absolute().inset_0().size_full().child(c))
        }
    }

    let (window_handle, _) = cx.update(|cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(800.0), px(600.0)),
            })),
            ..Default::default()
        };

        gpui_kit::open_window(options, cx, |_, cx| {
            cx.new(|_| OverlayTestView {
                flag: painted_clone,
            })
        })
        .unwrap()
    });

    cx.update_window(window_handle, |_, window, cx| {
        window.draw(cx).clear(cx);
    })
    .unwrap();

    let was_painted = painted_flag.load(Ordering::SeqCst);
    assert!(was_painted, "Canvas 沒有被正確繪製！");
}

#[test]
fn test_chart_viewport_candle_index_at_x() {
    let mut vp = stock_gpui::views::chart_viewport::ChartViewport {
        visible_count: 10,
        offset: 0,
        ..Default::default()
    };
    // 總共 30 根 K 棒，slice_range(30) -> start: 20, end: 30 (10 根)
    assert_eq!(vp.slice_range(30), (20, 30));

    // 圖表寬度 1000px，每根 K 棒寬約 100px
    // x = 50.0 命中第 0 欄，絕對索引為 20
    assert_eq!(vp.candle_index_at_x(50.0, 1000.0, 30), Some((0, 20)));
    // x = 950.0 命中第 9 欄，絕對索引為 29
    assert_eq!(vp.candle_index_at_x(950.0, 1000.0, 30), Some((9, 29)));
    // x < 0 超出左邊界
    assert_eq!(vp.candle_index_at_x(-5.0, 1000.0, 30), None);
    // x > width 超出右邊界
    assert_eq!(vp.candle_index_at_x(1050.0, 1000.0, 30), None);
    // total = 0
    assert_eq!(vp.candle_index_at_x(50.0, 1000.0, 0), None);

    // 測試向左平移 5 根
    vp.offset = 5;
    // slice_range(30) -> start: 15, end: 25
    assert_eq!(vp.slice_range(30), (15, 25));
    assert_eq!(vp.candle_index_at_x(50.0, 1000.0, 30), Some((0, 15)));
    assert_eq!(vp.candle_index_at_x(950.0, 1000.0, 30), Some((9, 24)));
}

#[gpui_kit::test]
fn test_analysis_crosshair_and_hud(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);

    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();
    let core_state = stock_gpui::CoreState::new_with_db_path(db_path).unwrap();

    let (window_handle, analysis_entity) = cx.update(|cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(1280.0), px(800.0)),
            })),
            ..Default::default()
        };

        gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| stock_gpui::views::AnalysisView::new(core_state, window, cx))
        })
        .expect("開啟視窗失敗")
    });

    analysis_entity.update(cx, |view, cx| {
        // 初始狀態：十字游標未懸停
        assert!(view.hovered_candle_idx.is_none());
        assert!(view.hover_pos.is_none());

        // 模擬注入分析數據
        let mut ohlcv = Vec::new();
        for i in 1..=20 {
            ohlcv.push(stock_core::models::OhlcvRecord {
                trade_date: format!("2026-03-{:02}", i),
                open_price: Some(100.0 + i as f64),
                high_price: Some(105.0 + i as f64),
                low_price: Some(99.0 + i as f64),
                close_price: Some(104.0 + i as f64),
                volume_shares: Some(500000),
            });
        }

        view.current_data = Some(stock_gpui::state::AnalysisData {
            symbol: "2330".into(),
            name: Some("台積電".into()),
            ohlcv,
            bollinger_upper: vec![Some(120.0); 20],
            bollinger_middle: vec![Some(110.0); 20],
            bollinger_lower: vec![Some(100.0); 20],
            ma5: vec![Some(108.0); 20],
            ma10: vec![Some(106.0); 20],
            ma20: vec![Some(104.0); 20],
            rsi: vec![Some(60.0); 20],
            macd_line: vec![Some(2.5); 20],
            macd_signal: vec![Some(1.8); 20],
            macd_histogram: vec![Some(0.7); 20],
            institutional: vec![None; 20],
        });

        // 模擬滑鼠移動至第 5 根 K 棒
        view.hovered_candle_idx = Some(4);
        view.hover_pos = Some(Point {
            x: px(200.0),
            y: px(150.0),
        });
        cx.notify();
    });

    cx.update_window(window_handle, |_, window, cx| {
        window.draw(cx).clear(cx);
    })
    .unwrap();

    analysis_entity.update(cx, |view, cx| {
        assert_eq!(view.hovered_candle_idx, Some(4));
        assert!(view.hover_pos.is_some());

        // 模擬滑鼠移出還原
        view.hovered_candle_idx = None;
        view.hover_pos = None;
        cx.notify();
    });

    cx.update_window(window_handle, |_, window, cx| {
        window.draw(cx).clear(cx);
    })
    .unwrap();

    analysis_entity.update(cx, |view, _| {
        assert!(view.hovered_candle_idx.is_none());
        assert!(view.hover_pos.is_none());
    });
}
