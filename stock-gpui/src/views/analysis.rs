use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::chart::{BarChart, CandlestickChart, LineChart};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AppContext, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _, Render,
    Styled as _, Window, div, hsla, px, rgb,
};

use crate::state::{AnalysisData, CoreState};
use crate::views::chart_viewport::{ChartViewport, compute_price_scales, compute_volume_scales};

#[derive(Clone, Debug)]
pub struct IndicatorPoint {
    pub date: String,
    pub val: f64,
}

pub struct AnalysisView {
    pub core_state: CoreState,
    pub symbol_input: Entity<InputState>,
    pub current_data: Option<AnalysisData>,
    pub is_loading: bool,
    pub error_msg: Option<String>,
    pub chart_viewport: ChartViewport,
}

impl AnalysisView {
    pub fn new(core_state: CoreState, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let symbol_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value("2330")
                .placeholder("輸入股票代號...")
        });

        cx.subscribe(
            &symbol_input,
            |this: &mut Self, input, event: &gpui_kit::component::input::InputEvent, cx| {
                if matches!(
                    event,
                    gpui_kit::component::input::InputEvent::PressEnter { .. }
                ) {
                    let sym = input.read(cx).value().trim().to_string();
                    if !sym.is_empty() {
                        this.fetch_symbol(sym, cx);
                    }
                }
            },
        )
        .detach();

        let mut view = Self {
            core_state,
            symbol_input,
            current_data: None,
            is_loading: false,
            error_msg: None,
            chart_viewport: ChartViewport::default(),
        };
        view.fetch_symbol("2330".into(), cx);
        view
    }

    pub fn fetch_symbol(&mut self, symbol: String, cx: &mut Context<Self>) {
        self.is_loading = true;
        self.error_msg = None;
        cx.notify();

        let core = self.core_state.clone();
        cx.spawn(async move |this, cx| {
            let res = core
                .get_analysis(&symbol, &[5, 10, 20], Some(14), Some(12), Some(26), Some(9))
                .await;

            let _ = this.update(cx, |view, cx| {
                view.is_loading = false;
                match res {
                    Ok(data) => {
                        view.current_data = Some(data);
                        view.error_msg = None;
                    }
                    Err(e) => {
                        view.current_data = None;
                        view.error_msg = Some(e);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for AnalysisView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let header = div()
            .flex()
            .items_center()
            .gap_3()
            .p_3()
            .border_b_1()
            .border_color(rgb(0x313244))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(div().text_sm().text_color(rgb(0xa6adc8)).child("股票代號:"))
                    .child(
                        div()
                            .w(px(160.))
                            .child(Input::new(&self.symbol_input).id("analysis-symbol-input")),
                    ),
            )
            .child(
                Button::new("btn-fetch-analysis")
                    .label("查詢個股分析")
                    .primary()
                    .on_click(cx.listener(|this, _, _, cx| {
                        let sym = this.symbol_input.read(cx).value().trim().to_string();
                        if !sym.is_empty() {
                            this.fetch_symbol(sym, cx);
                        }
                    })),
            )
            .when_some(self.current_data.as_ref(), |this, data| {
                this.child(
                    div()
                        .flex()
                        .gap_3()
                        .items_center()
                        .child(
                            div()
                                .font_weight(gpui_kit::FontWeight::BOLD)
                                .text_color(rgb(0x89b4fa))
                                .child(data.symbol.clone()),
                        )
                        .child(
                            div()
                                .text_color(rgb(0xa6adc8))
                                .child(data.name.clone().unwrap_or_default()),
                        ),
                )
            });

        let content = if self.is_loading {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(0x89b4fa))
                .child("載入分析數據中...")
        } else if let Some(err) = &self.error_msg {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(0xf38ba8))
                .child(format!("查詢錯誤：{}", err))
        } else if let Some(data) = &self.current_data {
            let ohlcv = data.ohlcv.clone();
            let total_candles = ohlcv.len();
            let (start_idx, end_idx) = self.chart_viewport.slice_range(total_candles);
            let visible_ohlcv = ohlcv[start_idx..end_idx].to_vec();

            let price_scales = compute_price_scales(&visible_ohlcv);
            let vol_scales = compute_volume_scales(&visible_ohlcv);

            let kline = CandlestickChart::new(visible_ohlcv.clone())
                .id("analysis-candlestick")
                .x(|d| d.trade_date.clone())
                .open(|d| d.open_price.unwrap_or(0.0))
                .high(|d| d.high_price.unwrap_or(0.0))
                .low(|d| d.low_price.unwrap_or(0.0))
                .close(|d| d.close_price.unwrap_or(0.0))
                .body_width_ratio(0.85)
                .max_band_width(px(60.0))
                .bullish(hsla(0.0, 0.8, 0.5, 1.0))
                .bearish(hsla(0.33, 0.8, 0.4, 1.0))
                .interactive(true);

            let vol_chart = BarChart::new(visible_ohlcv.clone())
                .id("analysis-volume")
                .band(|d| d.trade_date.clone())
                .value(|d| d.volume_shares.unwrap_or(0) as f64)
                .interactive(true);

            let rsi_points: Vec<IndicatorPoint> = ohlcv[start_idx..end_idx]
                .iter()
                .zip(data.rsi[start_idx..end_idx].iter())
                .filter_map(|(rec, val)| {
                    val.map(|v| IndicatorPoint {
                        date: rec.trade_date.clone(),
                        val: v,
                    })
                })
                .collect();

            let rsi_chart = LineChart::new(rsi_points)
                .id("analysis-rsi")
                .x(|p: &IndicatorPoint| p.date.clone())
                .y(|p: &IndicatorPoint| p.val)
                .stroke(hsla(0.15, 0.9, 0.6, 1.0))
                .interactive(true);

            // 縱軸價格刻度
            let price_axis = div()
                .w(px(70.))
                .h_full()
                .flex()
                .flex_col()
                .justify_between()
                .items_end()
                .py_2()
                .pr_1()
                .border_l_1()
                .border_color(rgb(0xe2e8f0))
                .children(
                    price_scales
                        .map(|scales| {
                            scales
                                .into_iter()
                                .map(|p| {
                                    div()
                                        .text_xs()
                                        .font_weight(gpui_kit::FontWeight::BOLD)
                                        .text_color(rgb(0x475569))
                                        .child(format!("{:.2}", p))
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default(),
                );

            // 縱軸成交量刻度
            let vol_axis = div()
                .w(px(70.))
                .h_full()
                .flex()
                .flex_col()
                .justify_between()
                .items_end()
                .py_1()
                .pr_1()
                .border_l_1()
                .border_color(rgb(0xe2e8f0))
                .children(
                    vol_scales
                        .map(|(max_v, half_v, _)| {
                            vec![
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x475569))
                                    .child(format!("{}張", max_v / 1000)),
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x475569))
                                    .child(format!("{}張", half_v / 1000)),
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x475569))
                                    .child("0張".to_string()),
                            ]
                        })
                        .unwrap_or_default(),
                );

            let date_range_label =
                if let (Some(first), Some(last)) = (visible_ohlcv.first(), visible_ohlcv.last()) {
                    format!(
                        "顯示 {} / {} 根（{} ~ {}）",
                        visible_ohlcv.len(),
                        total_candles,
                        first.trade_date,
                        last.trade_date
                    )
                } else {
                    format!("顯示 {} / {} 根", visible_ohlcv.len(), total_candles)
                };

            let chart_toolbar = div()
                .flex()
                .items_center()
                .justify_between()
                .px_2()
                .py_1()
                .bg(rgb(0xf1f5f9))
                .rounded_md()
                .border_1()
                .border_color(rgb(0xe2e8f0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            Button::new("btn-chart-zoom-in")
                                .label("🔍 放大 (+)")
                                .secondary()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.chart_viewport.zoom_in();
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("btn-chart-zoom-out")
                                .label("🔍 縮小 (-)")
                                .secondary()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.chart_viewport.zoom_out(total_candles);
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("btn-chart-pan-left")
                                .label("◀ 向左平移")
                                .secondary()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.chart_viewport.pan_left(total_candles);
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("btn-chart-pan-right")
                                .label("向右平移 ▶")
                                .secondary()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.chart_viewport.pan_right();
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("btn-chart-reset")
                                .label("⟲ 回最新")
                                .secondary()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.chart_viewport.reset();
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui_kit::FontWeight::BOLD)
                        .text_color(rgb(0x334155))
                        .child(date_range_label),
                );

            let kline_with_axis = div()
                .size_full()
                .flex()
                .flex_row()
                .child(div().flex_1().h_full().child(kline))
                .child(price_axis);

            let vol_with_axis = div()
                .size_full()
                .flex()
                .flex_row()
                .child(div().flex_1().h_full().child(vol_chart))
                .child(vol_axis);

            div()
                .size_full()
                .flex()
                .flex_col()
                .p_3()
                .gap_2()
                .child(chart_toolbar)
                .child(
                    div()
                        .flex_1()
                        .border_1()
                        .border_color(rgb(0x313244))
                        .rounded_md()
                        .p_2()
                        .on_scroll_wheel(cx.listener(
                            move |this, event: &gpui_kit::ScrollWheelEvent, _, cx| {
                                if this
                                    .chart_viewport
                                    .handle_scroll_wheel(&event.delta, total_candles)
                                {
                                    cx.notify();
                                }
                            },
                        ))
                        .on_mouse_down(
                            gpui_kit::MouseButton::Left,
                            cx.listener(|this, event: &gpui_kit::MouseDownEvent, _, _| {
                                this.chart_viewport.drag_start_x = Some(event.position.x.as_f32());
                            }),
                        )
                        .on_mouse_move(cx.listener(
                            move |this, event: &gpui_kit::MouseMoveEvent, _, cx| {
                                if event.dragging()
                                    && this
                                        .chart_viewport
                                        .handle_mouse_move(event.position.x.as_f32(), total_candles)
                                {
                                    cx.notify();
                                }
                            },
                        ))
                        .on_mouse_up(
                            gpui_kit::MouseButton::Left,
                            cx.listener(|this, _, _, _| {
                                this.chart_viewport.drag_start_x = None;
                            }),
                        )
                        .child(kline_with_axis),
                )
                .child(
                    div()
                        .h(px(140.))
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .flex_1()
                                .border_1()
                                .border_color(rgb(0x313244))
                                .rounded_md()
                                .p_2()
                                .child(vol_with_axis),
                        )
                        .child(
                            div()
                                .flex_1()
                                .border_1()
                                .border_color(rgb(0x313244))
                                .rounded_md()
                                .p_2()
                                .child(rsi_chart),
                        ),
                )
        } else {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(0x6c7086))
                .child("請輸入股票代號（例如 2330）並點擊「查詢個股分析」")
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(header)
            .child(content)
    }
}
