use gpui_kit::base::TestSupportExt as _;
use gpui_kit::test::TestWindowExt;
use gpui_kit::{
    AppContext, Bounds, Context, InteractiveElement as _, IntoElement, ParentElement as _, Point,
    Render, StatefulInteractiveElement as _, Styled as _, TestAppContext, TitlebarOptions, Window,
    WindowBounds, WindowOptions, div, px, size,
};

struct DummyView;

impl Render for DummyView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("app-title")
            .test_support()
            .aria_label("台股布林通道擠壓選股系統")
            .size_full()
            .child("台股布林通道擠壓選股系統")
    }
}

#[gpui_kit::test]
fn test_window_open_and_render(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);

    let (window_handle, _content) = cx.update(|cx| {
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

        gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| DummyView)).expect("開啟視窗失敗")
    });

    cx.update_window(window_handle.into(), |_, window, cx| {
        window.draw(cx).clear(cx);
        let title_elem = window.find("app-title");
        assert!(title_elem.visible());
        assert_eq!(title_elem.label(), Some("台股布林通道擠壓選股系統"));
    })
    .unwrap();
}
