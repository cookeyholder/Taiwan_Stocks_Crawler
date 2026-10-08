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
            .child(div().id("chart-container").test_support().size_full().child(kline))
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

        gpui_kit::open_window(options, cx, |_, cx| {
            cx.new(|_| ChartView { data: candles })
        })
        .expect("開啟視窗失敗")
    });

    cx.update_window(window_handle.into(), |_, window, cx| {
        window.draw(cx).clear(cx);
        let container = window.find("chart-container");
        assert!(container.visible());
    })
    .unwrap();
}
