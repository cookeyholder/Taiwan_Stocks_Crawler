use gpui_kit::base::Disableable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::{
    AppContext, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _, Window,
    div, px, rgb,
};
use stock_core::models::BacktestReport;

use crate::state::CoreState;

pub struct BacktestView {
    pub core_state: CoreState,
    pub start_date_input: Entity<InputState>,
    pub end_date_input: Entity<InputState>,
    pub capital_input: Entity<InputState>,
    pub report: Option<BacktestReport>,
    pub is_running: bool,
    pub status_text: String,
}

impl BacktestView {
    pub fn new(core_state: CoreState, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let start_date_input = cx.new(|cx| InputState::new(window, cx).default_value("2026-01-01"));
        let end_date_input = cx.new(|cx| InputState::new(window, cx).default_value("2026-03-20"));
        let capital_input = cx.new(|cx| InputState::new(window, cx).default_value("1000000"));

        Self {
            core_state,
            start_date_input,
            end_date_input,
            capital_input,
            report: None,
            is_running: false,
            status_text: "就緒".into(),
        }
    }

    pub fn run_backtest(&mut self, cx: &mut Context<Self>) {
        let start_date = self.start_date_input.read(cx).value().trim().to_string();
        let end_date = self.end_date_input.read(cx).value().trim().to_string();
        let capital: f64 = self
            .capital_input
            .read(cx)
            .value()
            .trim()
            .parse()
            .unwrap_or(1_000_000.0);

        self.is_running = true;
        self.status_text = "回測執行中...".into();
        cx.notify();

        let core = self.core_state.clone();
        cx.spawn(async move |this, cx| {
            let res = core
                .run_backtest(
                    "bollinger_squeeze",
                    &[
                        ("bandwidth_limit".to_string(), 0.08),
                        ("consecutive_days".to_string(), 5.0),
                    ],
                    &start_date,
                    &end_date,
                    capital,
                    None,
                )
                .await;

            let _ = this.update(cx, |view, cx| {
                view.is_running = false;
                match res {
                    Ok(report) => {
                        view.status_text = format!(
                            "回測完成：總報酬率 {:.2}%，交易次數 {}",
                            report.metrics.total_return_pct, report.metrics.total_trades
                        );
                        view.report = Some(report);
                    }
                    Err(e) => {
                        view.status_text = format!("回測失敗：{}", e);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for BacktestView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_running = self.is_running;

        let toolbar = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_3()
            .p_3()
            .border_b_1()
            .border_color(rgb(0x313244))
            .child(
                div()
                    .w(px(120.))
                    .child(Input::new(&self.start_date_input).id("bt-start-date")),
            )
            .child(
                div()
                    .w(px(120.))
                    .child(Input::new(&self.end_date_input).id("bt-end-date")),
            )
            .child(
                div()
                    .w(px(120.))
                    .child(Input::new(&self.capital_input).id("bt-capital")),
            )
            .child(
                Button::new("btn-run-backtest")
                    .label(if is_running { "計算中..." } else { "執行回測" })
                    .primary()
                    .disabled(is_running)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.run_backtest(cx);
                    })),
            )
            .child(
                div()
                    .text_color(rgb(0xa6adc8))
                    .text_sm()
                    .child(self.status_text.clone()),
            );

        let kpi_area = if let Some(rep) = &self.report {
            let m = &rep.metrics;
            div()
                .p_4()
                .flex()
                .flex_wrap()
                .gap_4()
                .child(kpi_card("總報酬率", &format!("{:.2}%", m.total_return_pct), m.total_return_pct >= 0.0))
                .child(kpi_card("年化報酬率", &format!("{:.2}%", m.annualized_return_pct), m.annualized_return_pct >= 0.0))
                .child(kpi_card("勝率", &format!("{:.1}%", m.win_rate * 100.0), true))
                .child(kpi_card("總交易筆數", &format!("{}", m.total_trades), true))
                .child(kpi_card("夏普比率", &format!("{:.2}", m.sharpe_ratio), m.sharpe_ratio >= 1.0))
                .child(kpi_card("最大回撤", &format!("{:.2}%", m.max_drawdown_pct), false))
                .child(kpi_card("期末淨值", &format!("{:.0}", m.final_value), m.final_value >= 1_000_000.0))
        } else {
            div()
                .p_4()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(0x6c7086))
                .child("設定日期區間與初始資金後點擊「執行回測」以檢視策略 KPI 績效卡片")
        };

        div().size_full().flex().flex_col().child(toolbar).child(kpi_area)
    }
}

fn kpi_card(title: &str, value: &str, positive: bool) -> impl IntoElement {
    let color = if positive { rgb(0xa6e3a1) } else { rgb(0xf38ba8) };
    div()
        .w(px(160.))
        .p_3()
        .border_1()
        .border_color(rgb(0x313244))
        .rounded_md()
        .bg(rgb(0x181825))
        .flex()
        .flex_col()
        .gap_1()
        .child(div().text_sm().text_color(rgb(0xa6adc8)).child(title.to_string()))
        .child(div().font_weight(gpui_kit::FontWeight::BOLD).text_color(color).child(value.to_string()))
}
