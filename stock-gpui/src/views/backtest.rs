use gpui_kit::base::Disableable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::table::{Column, ColumnSort, DataTable, TableDelegate, TableState};
use gpui_kit::component::Sizable as _;
use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _, Window,
    div, px, rgb,
};
use stock_core::models::{BacktestReport, TradeRecord};

use crate::app::FontSizeLevel;
use crate::state::CoreState;

pub struct TradesDelegate {
    pub trades: Vec<TradeRecord>,
    pub sort_col: Option<usize>,
    pub sort_order: ColumnSort,
    pub font_size: FontSizeLevel,
}

impl TradesDelegate {
    pub fn sort_by_col(&mut self, col: usize, sort: ColumnSort) {
        self.sort_col = Some(col);
        self.sort_order = sort;
        match sort {
            ColumnSort::Ascending => match col {
                0 => self.trades.sort_by(|a, b| a.buy_date.cmp(&b.buy_date)),
                1 => self.trades.sort_by(|a, b| a.sell_date.cmp(&b.sell_date)),
                2 => self.trades.sort_by(|a, b| a.symbol.cmp(&b.symbol)),
                3 => self.trades.sort_by(|a, b| a.name.cmp(&b.name)),
                4 => self.trades.sort_by_key(|a| a.shares),
                5 => self.trades.sort_by(|a, b| {
                    a.buy_price
                        .partial_cmp(&b.buy_price)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                6 => self.trades.sort_by(|a, b| {
                    a.sell_price
                        .partial_cmp(&b.sell_price)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                7 => self.trades.sort_by(|a, b| {
                    a.pnl
                        .partial_cmp(&b.pnl)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                _ => self.trades.sort_by(|a, b| {
                    a.pnl_pct
                        .partial_cmp(&b.pnl_pct)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
            },
            ColumnSort::Descending => match col {
                0 => self.trades.sort_by(|a, b| b.buy_date.cmp(&a.buy_date)),
                1 => self.trades.sort_by(|a, b| b.sell_date.cmp(&a.sell_date)),
                2 => self.trades.sort_by(|a, b| b.symbol.cmp(&a.symbol)),
                3 => self.trades.sort_by(|a, b| b.name.cmp(&a.name)),
                4 => self.trades.sort_by_key(|b| std::cmp::Reverse(b.shares)),
                5 => self.trades.sort_by(|a, b| {
                    b.buy_price
                        .partial_cmp(&a.buy_price)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                6 => self.trades.sort_by(|a, b| {
                    b.sell_price
                        .partial_cmp(&a.sell_price)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                7 => self.trades.sort_by(|a, b| {
                    b.pnl
                        .partial_cmp(&a.pnl)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                _ => self.trades.sort_by(|a, b| {
                    b.pnl_pct
                        .partial_cmp(&a.pnl_pct)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
            },
            ColumnSort::Default => {}
        }
    }
}

impl TableDelegate for TradesDelegate {
    fn columns_count(&self, _: &App) -> usize {
        9
    }

    fn rows_count(&self, _: &App) -> usize {
        self.trades.len()
    }

    fn column(&self, ix: usize, _: &App) -> Column {
        match ix {
            0 => Column::new("buy_date", "買入日").width(px(95.)).sortable(),
            1 => Column::new("sell_date", "賣出日").width(px(95.)).sortable(),
            2 => Column::new("symbol", "代號").width(px(80.)).sortable(),
            3 => Column::new("name", "名稱").width(px(100.)).sortable(),
            4 => Column::new("shares", "股數").width(px(80.)).sortable(),
            5 => Column::new("buy_price", "買價").width(px(80.)).sortable(),
            6 => Column::new("sell_price", "賣價").width(px(80.)).sortable(),
            7 => Column::new("pnl", "損益").width(px(90.)).sortable(),
            _ => Column::new("pnl_pct", "報酬率").width(px(85.)).sortable(),
        }
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) {
        self.sort_by_col(col_ix, sort);
    }

    fn render_td(
        &mut self,
        row: usize,
        col: usize,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let t = &self.trades[row];
        let pnl_color = if t.pnl >= 0.0 {
            rgb(0xa6e3a1)
        } else {
            rgb(0xf38ba8)
        };

        let fs = self.font_size.size_px();
        match col {
            0 => div().text_size(fs).child(t.buy_date.clone()),
            1 => div()
                .text_size(fs)
                .child(t.sell_date.clone().unwrap_or_else(|| "持有中".into())),
            2 => div().text_size(fs).child(t.symbol.clone()),
            3 => div().text_size(fs).child(t.name.clone().unwrap_or_default()),
            4 => div().text_size(fs).child(format!("{}", t.shares)),
            5 => div().text_size(fs).child(format!("{:.2}", t.buy_price)),
            6 => div().text_size(fs).child(
                t.sell_price
                    .map(|p| format!("{:.2}", p))
                    .unwrap_or_else(|| "-".into()),
            ),
            7 => div()
                .text_size(fs)
                .text_color(pnl_color)
                .child(format!("{:+0.0}", t.pnl)),
            _ => div()
                .text_size(fs)
                .text_color(pnl_color)
                .child(format!("{:+.2}%", t.pnl_pct)),
        }
    }
}

fn get_today_str() -> String {
    let tz_offset = chrono::FixedOffset::east_opt(8 * 3600).unwrap();
    chrono::Utc::now()
        .with_timezone(&tz_offset)
        .format("%Y-%m-%d")
        .to_string()
}

pub struct BacktestView {
    pub core_state: CoreState,
    pub start_date_input: Entity<InputState>,
    pub end_date_input: Entity<InputState>,
    pub capital_input: Entity<InputState>,
    pub include_etf: bool,
    pub table_state: Entity<TableState<TradesDelegate>>,
    pub report: Option<BacktestReport>,
    pub is_running: bool,
    pub status_text: String,
    pub font_size: FontSizeLevel,
}

impl BacktestView {
    pub fn new(core_state: CoreState, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let today = get_today_str();
        let start_date_input = cx.new(|cx| InputState::new(window, cx).default_value("2026-01-01"));
        let end_date_input = cx.new(|cx| InputState::new(window, cx).default_value(&today));
        let capital_input = cx.new(|cx| InputState::new(window, cx).default_value("1000000"));

        for input_field in [&start_date_input, &end_date_input, &capital_input] {
            cx.subscribe(
                input_field,
                |this: &mut Self, _, event: &gpui_kit::component::input::InputEvent, cx| {
                    if matches!(
                        event,
                        gpui_kit::component::input::InputEvent::PressEnter { .. }
                    ) {
                        this.run_backtest(cx);
                    }
                },
            )
            .detach();
        }

        let delegate = TradesDelegate {
            trades: Vec::new(),
            sort_col: None,
            sort_order: ColumnSort::Default,
            font_size: FontSizeLevel::Medium,
        };
        let table_state = cx.new(|cx| TableState::new(delegate, window, cx).cell_selectable(true));

        Self {
            core_state,
            start_date_input,
            end_date_input,
            capital_input,
            include_etf: false,
            table_state,
            report: None,
            is_running: false,
            status_text: "就緒".into(),
            font_size: FontSizeLevel::Medium,
        }
    }

    pub fn set_font_size(&mut self, font_size: FontSizeLevel, cx: &mut Context<Self>) {
        self.font_size = font_size;
        self.table_state.update(cx, |table, cx| {
            table.delegate_mut().font_size = font_size;
            table.refresh(cx);
        });
        cx.notify();
    }

    pub fn run_backtest(&mut self, cx: &mut Context<Self>) {
        let start_date = self.start_date_input.read(cx).value().trim().to_string();
        let end_date = self.end_date_input.read(cx).value().trim().to_string();

        if start_date.is_empty() || end_date.is_empty() {
            self.status_text = "請輸入有效的起始與結束日期".into();
            cx.notify();
            return;
        }

        let capital: f64 = self
            .capital_input
            .read(cx)
            .value()
            .trim()
            .parse()
            .unwrap_or(1_000_000.0);

        let instrument_types = if self.include_etf {
            Some(vec!["stock".to_string(), "etf".to_string()])
        } else {
            Some(vec!["stock".to_string()])
        };

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
                    instrument_types,
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
                        view.table_state.update(cx, |table, _| {
                            table.delegate_mut().trades = report.trades.clone();
                        });
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
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(div().text_sm().text_color(rgb(0xa6adc8)).child("起始:"))
                    .child(
                        div()
                            .w(px(120.))
                            .child(Input::new(&self.start_date_input).id("bt-start-date")),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(div().text_sm().text_color(rgb(0xa6adc8)).child("結束:"))
                    .child(
                        div()
                            .w(px(120.))
                            .child(Input::new(&self.end_date_input).id("bt-end-date")),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(div().text_sm().text_color(rgb(0xa6adc8)).child("資金:"))
                    .child(
                        div()
                            .w(px(120.))
                            .child(Input::new(&self.capital_input).id("bt-capital")),
                    ),
            )
            .child(
                Checkbox::new("backtest-include-etf")
                    .label("包含 ETF")
                    .checked(self.include_etf)
                    .on_click(cx.listener(|this, checked, _, cx| {
                        this.include_etf = *checked;
                        cx.notify();
                    })),
            )
            .child(
                Button::new("btn-run-backtest")
                    .label(if is_running {
                        "計算中..."
                    } else {
                        "執行回測"
                    })
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
                .child(kpi_card(
                    "總報酬率",
                    &format!("{:.2}%", m.total_return_pct),
                    m.total_return_pct >= 0.0,
                ))
                .child(kpi_card(
                    "年化報酬率",
                    &format!("{:.2}%", m.annualized_return_pct),
                    m.annualized_return_pct >= 0.0,
                ))
                .child(kpi_card(
                    "勝率",
                    &format!("{:.1}%", m.win_rate * 100.0),
                    true,
                ))
                .child(kpi_card("總交易筆數", &format!("{}", m.total_trades), true))
                .child(kpi_card(
                    "夏普比率",
                    &format!("{:.2}", m.sharpe_ratio),
                    m.sharpe_ratio >= 1.0,
                ))
                .child(kpi_card(
                    "最大回撤",
                    &format!("{:.2}%", m.max_drawdown_pct),
                    false,
                ))
                .child(kpi_card(
                    "期末淨值",
                    &format!("{:.0}", m.final_value),
                    m.final_value >= 1_000_000.0,
                ))
        } else {
            div()
                .p_4()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(0x6c7086))
                .child("設定日期區間與初始資金後點擊「執行回測」以檢視策略 KPI 績效與交易明細")
        };

        let trades_count = self.table_state.read(cx).delegate().trades.len();
        let trades_area = if trades_count > 0 {
            div()
                .flex_1()
                .size_full()
                .p_4()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui_kit::FontWeight::BOLD)
                        .text_color(rgb(0x89b4fa))
                        .child(format!("交易明細（共 {} 筆）", trades_count)),
                )
                .child(
                    div()
                        .flex_1()
                        .size_full()
                        .child(
                            DataTable::new(&self.table_state)
                                .with_size(self.font_size.table_size()),
                        ),
                )
        } else {
            div()
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(toolbar)
            .child(kpi_area)
            .child(trades_area)
    }
}

fn kpi_card(title: &str, value: &str, positive: bool) -> impl IntoElement {
    let color = if positive {
        rgb(0xa6e3a1)
    } else {
        rgb(0xf38ba8)
    };
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
        .child(
            div()
                .text_sm()
                .text_color(rgb(0xa6adc8))
                .child(title.to_string()),
        )
        .child(
            div()
                .font_weight(gpui_kit::FontWeight::BOLD)
                .text_color(color)
                .child(value.to_string()),
        )
}
