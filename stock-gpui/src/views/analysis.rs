use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::chart::{BarChart, CandlestickChart, LineChart};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AppContext, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _, Window, div,
    hsla, px, rgb,
};

use crate::state::{AnalysisData, CoreState};

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
}

impl AnalysisView {
    pub fn new(core_state: CoreState, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let symbol_input = cx.new(|cx| InputState::new(window, cx).default_value("2330"));
        let mut view = Self {
            core_state,
            symbol_input,
            current_data: None,
            is_loading: false,
            error_msg: None,
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
                    .w(px(160.))
                    .child(Input::new(&self.symbol_input).id("analysis-symbol-input")),
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

            let kline = CandlestickChart::new(ohlcv.clone())
                .id("analysis-candlestick")
                .x(|d| d.trade_date.clone())
                .open(|d| d.open_price.unwrap_or(0.0))
                .high(|d| d.high_price.unwrap_or(0.0))
                .low(|d| d.low_price.unwrap_or(0.0))
                .close(|d| d.close_price.unwrap_or(0.0))
                .bullish(hsla(0.0, 0.8, 0.5, 1.0))
                .bearish(hsla(0.33, 0.8, 0.4, 1.0))
                .interactive(true);

            let vol_chart = BarChart::new(ohlcv.clone())
                .id("analysis-volume")
                .band(|d| d.trade_date.clone())
                .value(|d| d.volume_shares.unwrap_or(0) as f64)
                .interactive(true);

            let rsi_points: Vec<IndicatorPoint> = ohlcv
                .iter()
                .zip(data.rsi.iter())
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

            div()
                .size_full()
                .flex()
                .flex_col()
                .p_3()
                .gap_3()
                .child(
                    div()
                        .flex_1()
                        .border_1()
                        .border_color(rgb(0x313244))
                        .rounded_md()
                        .p_2()
                        .child(kline),
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
                                .child(vol_chart),
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
