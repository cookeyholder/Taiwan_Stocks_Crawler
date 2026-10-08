use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::test::TestWindowExt;
use gpui_kit::{
    AppContext, Bounds, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    Point, Render, StatefulInteractiveElement as _, Styled as _, TestAppContext, TitlebarOptions,
    Window, WindowBounds, WindowOptions, div, px, size,
};

struct CjkView {
    input: Entity<InputState>,
}

impl Render for CjkView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .id("cjk-title")
                    .test_support()
                    .aria_label("台股布林通道擠壓選股系統 測試 CJK 繁體中文渲染")
                    .child("台股布林通道擠壓選股系統 測試 CJK 繁體中文渲染"),
            )
            .child(Input::new(&self.input).id("stock-input").w(px(300.)))
    }
}

#[gpui_kit::test]
fn test_cjk_font_and_input(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);

    let (window_handle, view_entity) = cx.update(|cx| {
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
            let input = cx.new(|cx| InputState::new(window, cx).default_value("2330 台積電"));
            cx.new(|_| CjkView { input })
        })
        .expect("開啟視窗失敗")
    });

    cx.update_window(window_handle.into(), |_, window, cx| {
        window.draw(cx).clear(cx);

        // 驗證 CJK 繁體中文字元渲染
        let title = window.find("cjk-title");
        assert!(title.visible());
        assert_eq!(
            title.label(),
            Some("台股布林通道擠壓選股系統 測試 CJK 繁體中文渲染")
        );

        // 驗證中文與股票代號輸入法文字輸入
        window.click("stock-input", cx);
        window.input("聯發科 2454", cx);
    })
    .unwrap();

    let input_state = view_entity.read_with(cx, |v, _| v.input.clone());
    input_state.update(cx, |input, _| {
        assert_eq!(input.value(), "2330 台積電聯發科 2454");
    });
}
