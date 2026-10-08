use gpui_kit::*;
use stock_gpui::{AppView, CoreState};

fn main() {
    let _tokio_guard = stock_gpui::state::TOKIO_RUNTIME.enter();
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);

        let core_state = CoreState::new().expect("初始化核心資料庫管線失敗");

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
        .expect("開啟視窗失敗");
    });
}
