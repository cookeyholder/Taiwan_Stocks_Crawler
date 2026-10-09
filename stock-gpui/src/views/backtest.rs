use gpui_kit::base::Disableable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::chart::{BarChart, CandlestickChart};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::plot::scale::{Scale, ScaleBand, ScaleLinear};
use gpui_kit::component::resizable::{h_resizable, resizable_panel};
use gpui_kit::component::table::{Column, ColumnSort, DataTable, TableDelegate, TableEvent, TableState};
use gpui_kit::component::Sizable as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    canvas, div, hsla, point, px, rgb, App, AppContext, Context, Entity, InteractiveElement as _,
    IntoElement, ParentElement as _, PathBuilder, Pixels, Render, Styled as _, Window,
};
use stock_core::models::{BacktestReport, TradeRecord};

use crate::app::FontSizeLevel;
use crate::state::{AnalysisData, CoreState};
use crate::views::chart_viewport::{ChartViewport, compute_price_scales, compute_volume_scales};

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
        let mut col = match ix {
            0 => Column::new("buy_date", "買入日").width(px(95.)).sortable(),
            1 => Column::new("sell_date", "賣出日").width(px(95.)).sortable(),
            2 => Column::new("symbol", "代號").width(px(80.)).sortable(),
            3 => Column::new("name", "名稱").width(px(100.)).sortable(),
            4 => Column::new("shares", "股數").width(px(80.)).sortable(),
            5 => Column::new("buy_price", "買價").width(px(80.)).sortable(),
            6 => Column::new("sell_price", "賣價").width(px(80.)).sortable(),
            7 => Column::new("pnl", "損益").width(px(90.)).sortable(),
            _ => Column::new("pnl_pct", "報酬率").width(px(85.)).sortable(),
        };
        if self.sort_col == Some(ix) {
            col = col.sort(self.sort_order);
        }
        col
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) {
        self.sort_by_col(col_ix, sort);
        cx.notify();
    }

    fn render_th(
        &mut self,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let name = self.column(col_ix, cx).name.clone();
        let indicator = if self.sort_col == Some(col_ix) {
            match self.sort_order {
                ColumnSort::Ascending => " ▲",
                ColumnSort::Descending => " ▼",
                ColumnSort::Default => "",
            }
        } else {
            ""
        };
        div()
            .size_full()
            .cursor_pointer()
            .text_size(self.font_size.size_px())
            .font_weight(gpui_kit::FontWeight::BOLD)
            .flex()
            .items_center()
            .child(format!("{}{}", name, indicator))
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
        let content: gpui_kit::AnyElement = match col {
            0 => div().text_size(fs).child(t.buy_date.clone()).into_any_element(),
            1 => div()
                .text_size(fs)
                .child(t.sell_date.clone().unwrap_or_else(|| "持有中".into()))
                .into_any_element(),
            2 => div().text_size(fs).child(t.symbol.clone()).into_any_element(),
            3 => div().text_size(fs).child(t.name.clone().unwrap_or_default()).into_any_element(),
            4 => div().text_size(fs).child(format!("{}", t.shares)).into_any_element(),
            5 => div().text_size(fs).child(format!("{:.2}", t.buy_price)).into_any_element(),
            6 => div().text_size(fs).child(
                t.sell_price
                    .map(|p| format!("{:.2}", p))
                    .unwrap_or_else(|| "-".into()),
            ).into_any_element(),
            7 => div()
                .text_size(fs)
                .text_color(pnl_color)
                .child(format!("{:+0.0}", t.pnl))
                .into_any_element(),
            _ => div()
                .text_size(fs)
                .text_color(pnl_color)
                .child(format!("{:+.2}%", t.pnl_pct))
                .into_any_element(),
        };

        div().cursor_pointer().child(content)
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
    pub selected_trade: Option<TradeRecord>,
    pub chart_data: Option<AnalysisData>,
    pub is_chart_loading: bool,
    pub chart_viewport: ChartViewport,
    pub show_bollinger: bool,
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
        let table_state = cx.new(|cx| {
            TableState::new(delegate, window, cx)
                .cell_selectable(true)
                .row_selectable(true)
        });

        cx.subscribe(
            &table_state,
            |this: &mut Self, table, event: &TableEvent, cx| {
                match event {
                    TableEvent::SelectRow(r) | TableEvent::DoubleClickedRow(r) => {
                        let trade_opt = {
                            let d = table.read(cx).delegate();
                            d.trades.get(*r).cloned()
                        };
                        if let Some(trade) = trade_opt {
                            this.select_trade(trade, cx);
                        }
                    }
                    TableEvent::SelectCell(r, _) | TableEvent::DoubleClickedCell(r, _) => {
                        let trade_opt = {
                            let d = table.read(cx).delegate();
                            d.trades.get(*r).cloned()
                        };
                        if let Some(trade) = trade_opt {
                            this.select_trade(trade, cx);
                        }
                    }
                    TableEvent::SelectColumn(col) => {
                        table.update(cx, |t, cx| {
                            let current_col = t.delegate().sort_col;
                            let current_sort = t.delegate().sort_order;
                            let next_sort = if current_col == Some(*col) {
                                match current_sort {
                                    ColumnSort::Ascending => ColumnSort::Descending,
                                    ColumnSort::Descending => ColumnSort::Ascending,
                                    ColumnSort::Default => ColumnSort::Ascending,
                                }
                            } else {
                                ColumnSort::Descending
                            };
                            t.delegate_mut().sort_by_col(*col, next_sort);
                            t.refresh(cx);
                        });
                        cx.notify();
                    }
                    _ => {}
                }
            },
        )
        .detach();

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
            selected_trade: None,
            chart_data: None,
            is_chart_loading: false,
            chart_viewport: ChartViewport::default(),
            show_bollinger: true,
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

    pub fn select_trade(&mut self, trade: TradeRecord, cx: &mut Context<Self>) {
        if self.selected_trade.as_ref().map(|t| (&t.symbol, &t.buy_date))
            == Some((&trade.symbol, &trade.buy_date))
            && self.chart_data.is_some()
        {
            self.selected_trade = Some(trade);
            cx.notify();
            return;
        }
        self.selected_trade = Some(trade.clone());
        self.is_chart_loading = true;
        cx.notify();

        let sym = trade.symbol.clone();
        let buy_date = trade.buy_date.clone();
        let core = self.core_state.clone();

        cx.spawn(async move |this, cx| {
            let res = core
                .get_analysis(&sym, &[5, 10, 20], Some(14), Some(12), Some(26), Some(9))
                .await;

            let _ = this.update(cx, |view, cx| {
                view.is_chart_loading = false;
                match res {
                    Ok(data) => {
                        let total = data.ohlcv.len();
                        if let Some(pos) = data.ohlcv.iter().position(|r| r.trade_date == buy_date) {
                            let vis = 40.min(total).max(15);
                            view.chart_viewport.visible_count = vis;
                            let target_end = (pos + vis / 2).min(total);
                            view.chart_viewport.offset = total.saturating_sub(target_end);
                        } else {
                            view.chart_viewport.reset();
                        }
                        view.chart_data = Some(data);
                    }
                    Err(e) => {
                        eprintln!("載入股票 {} K 線圖失敗: {}", sym, e);
                        view.chart_data = None;
                    }
                }
                cx.notify();
            });
        })
        .detach();
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
                        if let Some(first_trade) = report.trades.first().cloned() {
                            view.select_trade(first_trade, cx);
                        }
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

        let chart_section = if self.is_chart_loading {
            div()
                .size_full()
                .border_1()
                .border_color(rgb(0x313244))
                .rounded_md()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(0x89b4fa))
                .child("載入 K 線圖中...")
        } else if let Some(data) = &self.chart_data {
            let ohlcv = data.ohlcv.clone();
            let total_candles = ohlcv.len();
            let (start_idx, end_idx) = self.chart_viewport.slice_range(total_candles);
            let visible_ohlcv = ohlcv[start_idx..end_idx].to_vec();

            let price_scales = compute_price_scales(&visible_ohlcv);
            let vol_scales = compute_volume_scales(&visible_ohlcv);
            let max_band_w = self.font_size.kline_band_width();

            let kline = CandlestickChart::new(visible_ohlcv.clone())
                .id("backtest-candlestick")
                .x(|d| d.trade_date.clone())
                .open(|d| d.open_price.unwrap_or(0.0))
                .high(|d| d.high_price.unwrap_or(0.0))
                .low(|d| d.low_price.unwrap_or(0.0))
                .close(|d| d.close_price.unwrap_or(0.0))
                .body_width_ratio(0.85)
                .max_band_width(max_band_w)
                .bullish(hsla(0.0, 0.8, 0.5, 1.0))
                .bearish(hsla(0.33, 0.8, 0.4, 1.0))
                .interactive(true);

            // 布林通道畫布
            let safe_slice = |vec: &[Option<f64>]| -> Vec<Option<f64>> {
                (start_idx..end_idx)
                    .map(|i| vec.get(i).copied().flatten())
                    .collect()
            };

            let visible_upper = safe_slice(&data.bollinger_upper);
            let visible_middle = safe_slice(&data.bollinger_middle);
            let visible_lower = safe_slice(&data.bollinger_lower);
            let visible_ohlcv_bb = visible_ohlcv.clone();

            let bollinger_canvas = canvas(
                move |_, _, _| {},
                move |bounds, _, window, _cx| {
                    let height = bounds.size.height.as_f32() - 18.0;
                    if height <= 0.0
                        || bounds.size.width.as_f32() <= 0.0
                        || visible_ohlcv_bb.is_empty()
                    {
                        return;
                    }

                    let x_scale = ScaleBand::new(
                        visible_ohlcv_bb.iter().map(|d| d.trade_date.clone()),
                        [0., bounds.size.width.as_f32()],
                    )
                    .max_band_width(max_band_w.as_f32())
                    .padding_inner(0.4)
                    .padding_outer(0.2);

                    let all_values: Vec<f64> = visible_ohlcv_bb
                        .iter()
                        .flat_map(|d| {
                            vec![
                                d.high_price.unwrap_or(0.0),
                                d.low_price.unwrap_or(0.0),
                                d.open_price.unwrap_or(0.0),
                                d.close_price.unwrap_or(0.0),
                            ]
                        })
                        .collect();

                    if all_values.is_empty() {
                        return;
                    }

                    let y_scale = ScaleLinear::new(all_values, [height, 10.]);
                    let half_band = x_scale.band_width() / 2.0;

                    let build_band_path = |series: &[Option<f64>]| -> Option<gpui_kit::Path<Pixels>> {
                        let mut builder = PathBuilder::stroke(px(2.0));
                        let mut started = false;
                        for (d, val_opt) in visible_ohlcv_bb.iter().zip(series.iter()) {
                            if let Some(val) = val_opt
                                && let (Some(xt), Some(yt)) =
                                    (x_scale.tick(&d.trade_date), y_scale.tick(val))
                            {
                                let pt = point(
                                    bounds.origin.x + px(xt + half_band),
                                    bounds.origin.y + px(yt),
                                );
                                if !started {
                                    builder.move_to(pt);
                                    started = true;
                                } else {
                                    builder.line_to(pt);
                                }
                            }
                        }
                        if started {
                            builder.build().ok()
                        } else {
                            None
                        }
                    };

                    window.with_content_mask(Some(gpui_kit::ContentMask { bounds }), |window| {
                        // 上軌：紫色 (Violet)
                        if let Some(upper_path) = build_band_path(&visible_upper) {
                            window.paint_path(upper_path, hsla(0.80, 0.85, 0.60, 1.0));
                        }
                        // 中軌：橙黃色 (20MA)
                        if let Some(middle_path) = build_band_path(&visible_middle) {
                            window.paint_path(middle_path, hsla(0.10, 0.95, 0.50, 1.0));
                        }
                        // 下軌：青藍色 (Cyan)
                        if let Some(lower_path) = build_band_path(&visible_lower) {
                            window.paint_path(lower_path, hsla(0.55, 0.85, 0.45, 1.0));
                        }
                    });
                },
            )
            .size_full();

            // 買進、賣出日 K 棒標記畫布（箭頭指示、引線與買賣連線）
            let selected_trade_clone = self.selected_trade.clone();
            let visible_ohlcv_trade = visible_ohlcv.clone();

            let trade_overlay_canvas = canvas(
                move |_, _, _| {},
                move |bounds, _, window, _cx| {
                    let height = bounds.size.height.as_f32() - 18.0;
                    if height <= 0.0
                        || bounds.size.width.as_f32() <= 0.0
                        || visible_ohlcv_trade.is_empty()
                    {
                        return;
                    }
                    let Some(trade) = &selected_trade_clone else {
                        return;
                    };

                    let x_scale = ScaleBand::new(
                        visible_ohlcv_trade.iter().map(|d| d.trade_date.clone()),
                        [0., bounds.size.width.as_f32()],
                    )
                    .max_band_width(max_band_w.as_f32())
                    .padding_inner(0.4)
                    .padding_outer(0.2);

                    let all_values: Vec<f64> = visible_ohlcv_trade
                        .iter()
                        .flat_map(|d| {
                            vec![
                                d.high_price.unwrap_or(0.0),
                                d.low_price.unwrap_or(0.0),
                                d.open_price.unwrap_or(0.0),
                                d.close_price.unwrap_or(0.0),
                            ]
                        })
                        .collect();

                    if all_values.is_empty() {
                        return;
                    }

                    let y_scale = ScaleLinear::new(all_values, [height, 10.]);
                    let half_band = x_scale.band_width() / 2.0;

                    let mut buy_point: Option<gpui_kit::Point<Pixels>> = None;
                    let mut sell_point: Option<gpui_kit::Point<Pixels>> = None;

                    window.with_content_mask(Some(gpui_kit::ContentMask { bounds }), |window| {
                        // 1. 買進標記 (紅色向上箭頭 ▲ 與引線)
                        for d in &visible_ohlcv_trade {
                            if d.trade_date == trade.buy_date {
                                if let (Some(xt), Some(low)) = (x_scale.tick(&d.trade_date), d.low_price)
                                    && let Some(yt) = y_scale.tick(&low)
                                {
                                    let center_x = bounds.origin.x + px(xt + half_band);
                                    let low_y = bounds.origin.y + px(yt);

                                    let buy_y = y_scale
                                        .tick(&trade.buy_price)
                                        .map(|p_yt| bounds.origin.y + px(p_yt))
                                        .unwrap_or(low_y);
                                    buy_point = Some(point(center_x, buy_y));

                                    // 指向 K 棒底部的引線
                                    let mut guide = PathBuilder::stroke(px(2.0));
                                    guide.move_to(point(center_x, low_y + px(2.0)));
                                    guide.line_to(point(center_x, low_y + px(8.0)));
                                    if let Ok(p) = guide.build() {
                                        window.paint_path(p, hsla(0.0, 0.85, 0.55, 1.0));
                                    }

                                    // 向上指示箭頭 ▲
                                    let top = point(center_x, low_y + px(8.0));
                                    let left = point(center_x - px(7.0), low_y + px(22.0));
                                    let right = point(center_x + px(7.0), low_y + px(22.0));

                                    let mut arrow = PathBuilder::stroke(px(2.5));
                                    arrow.move_to(top);
                                    arrow.line_to(left);
                                    arrow.line_to(right);
                                    arrow.line_to(top);
                                    arrow.move_to(top);
                                    arrow.line_to(point(center_x, low_y + px(22.0)));
                                    if let Ok(p) = arrow.build() {
                                        window.paint_path(p, hsla(0.0, 0.90, 0.55, 1.0));
                                    }
                                }
                                break;
                            }
                        }

                        // 2. 賣出標記 (綠色向下箭頭 ▼ 與引線)
                        if let Some(sell_date) = &trade.sell_date {
                            for d in &visible_ohlcv_trade {
                                if &d.trade_date == sell_date {
                                    if let (Some(xt), Some(high)) = (x_scale.tick(&d.trade_date), d.high_price)
                                        && let Some(yt) = y_scale.tick(&high)
                                    {
                                        let center_x = bounds.origin.x + px(xt + half_band);
                                        let high_y = bounds.origin.y + px(yt);

                                        let sell_y = trade
                                            .sell_price
                                            .as_ref()
                                            .and_then(|sp| y_scale.tick(sp))
                                            .map(|p_yt| bounds.origin.y + px(p_yt))
                                            .unwrap_or(high_y);
                                        sell_point = Some(point(center_x, sell_y));

                                        // 指向 K 棒頂部的引線
                                        let mut guide = PathBuilder::stroke(px(2.0));
                                        guide.move_to(point(center_x, high_y - px(2.0)));
                                        guide.line_to(point(center_x, high_y - px(8.0)));
                                        if let Ok(p) = guide.build() {
                                            window.paint_path(p, hsla(0.33, 0.85, 0.42, 1.0));
                                        }

                                        // 向下指示箭頭 ▼
                                        let bottom = point(center_x, high_y - px(8.0));
                                        let left = point(center_x - px(7.0), high_y - px(22.0));
                                        let right = point(center_x + px(7.0), high_y - px(22.0));

                                        let mut arrow = PathBuilder::stroke(px(2.5));
                                        arrow.move_to(bottom);
                                        arrow.line_to(left);
                                        arrow.line_to(right);
                                        arrow.line_to(bottom);
                                        arrow.move_to(bottom);
                                        arrow.line_to(point(center_x, high_y - px(22.0)));
                                        if let Ok(p) = arrow.build() {
                                            window.paint_path(p, hsla(0.33, 0.85, 0.42, 1.0));
                                        }
                                    }
                                    break;
                                }
                            }
                        }

                        // 3. 買賣走勢連線
                        if let (Some(bp), Some(sp)) = (buy_point, sell_point) {
                            let mut line = PathBuilder::stroke(px(2.0));
                            line.move_to(bp);
                            line.line_to(sp);
                            if let Ok(p) = line.build() {
                                let line_color = if trade.pnl >= 0.0 {
                                    hsla(0.0, 0.80, 0.55, 0.75)
                                } else {
                                    hsla(0.33, 0.80, 0.42, 0.75)
                                };
                                window.paint_path(p, line_color);
                            }
                        }
                    });
                },
            )
            .size_full();

            let vol_chart = BarChart::new(visible_ohlcv.clone())
                .id("backtest-volume")
                .band(|d| d.trade_date.clone())
                .value(|d| d.volume_shares.unwrap_or(0) as f64)
                .interactive(true);

            let axis_font_size = self.font_size.axis_font_size();
            let axis_w = self.font_size.axis_width();

            // 縱軸價格刻度
            let price_axis = div()
                .w(axis_w)
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
                                        .text_size(axis_font_size)
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
                .w(axis_w)
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
                                    .text_size(axis_font_size)
                                    .text_color(rgb(0x475569))
                                    .child(format!("{}張", max_v / 1000)),
                                div()
                                    .text_size(axis_font_size)
                                    .text_color(rgb(0x475569))
                                    .child(format!("{}張", half_v / 1000)),
                                div()
                                    .text_size(axis_font_size)
                                    .text_color(rgb(0x475569))
                                    .child("0張".to_string()),
                            ]
                        })
                        .unwrap_or_default(),
                );

            let date_range_label =
                if let (Some(first), Some(last)) = (visible_ohlcv.first(), visible_ohlcv.last()) {
                    if let Some(trade) = &self.selected_trade {
                        format!(
                            "顯示 {} / {} 根（{} ~ {}） | 買: {} 賣: {}",
                            visible_ohlcv.len(),
                            total_candles,
                            first.trade_date,
                            last.trade_date,
                            trade.buy_date,
                            trade.sell_date.as_deref().unwrap_or("持倉中")
                        )
                    } else {
                        format!(
                            "顯示 {} / {} 根（{} ~ {}）",
                            visible_ohlcv.len(),
                            total_candles,
                            first.trade_date,
                            last.trade_date
                        )
                    }
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
                            Button::new("btn-backtest-zoom-in")
                                .label("🔍 放大 (+)")
                                .secondary()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.chart_viewport.zoom_in();
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("btn-backtest-zoom-out")
                                .label("🔍 縮小 (-)")
                                .secondary()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.chart_viewport.zoom_out(total_candles);
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("btn-backtest-pan-left")
                                .label("◀ 向左平移")
                                .secondary()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.chart_viewport.pan_left(total_candles);
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("btn-backtest-pan-right")
                                .label("向右平移 ▶")
                                .secondary()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.chart_viewport.pan_right();
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("btn-backtest-focus-trade")
                                .label("🎯 聚焦交易")
                                .secondary()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    if let (Some(data), Some(trade)) =
                                        (&this.chart_data, &this.selected_trade)
                                    {
                                        let total = data.ohlcv.len();
                                        if let Some(pos) = data
                                            .ohlcv
                                            .iter()
                                            .position(|r| r.trade_date == trade.buy_date)
                                        {
                                            let vis = 40.min(total).max(15);
                                            this.chart_viewport.visible_count = vis;
                                            let target_end = (pos + vis / 2).min(total);
                                            this.chart_viewport.offset =
                                                total.saturating_sub(target_end);
                                            cx.notify();
                                        }
                                    }
                                })),
                        )
                        .child(
                            Button::new("btn-backtest-reset")
                                .label("⟲ 回最新")
                                .secondary()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.chart_viewport.reset();
                                    cx.notify();
                                })),
                        )
                        .child(div().w(px(1.)).h(px(18.)).bg(rgb(0xcbd5e1)))
                        .child(
                            Checkbox::new("backtest-show-bollinger")
                                .label("布林通道")
                                .checked(self.show_bollinger)
                                .on_click(cx.listener(|this, checked, _, cx| {
                                    this.show_bollinger = *checked;
                                    cx.notify();
                                })),
                        )
                        .when(self.show_bollinger, |this| {
                            this.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .text_xs()
                                    .text_color(rgb(0x64748b))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .w(px(8.))
                                                    .h(px(3.))
                                                    .rounded_sm()
                                                    .bg(hsla(0.80, 0.85, 0.60, 1.0)),
                                            )
                                            .child("上軌"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .w(px(8.))
                                                    .h(px(3.))
                                                    .rounded_sm()
                                                    .bg(hsla(0.10, 0.95, 0.50, 1.0)),
                                            )
                                            .child("中軌"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .w(px(8.))
                                                    .h(px(3.))
                                                    .rounded_sm()
                                                    .bg(hsla(0.55, 0.85, 0.45, 1.0)),
                                            )
                                            .child("下軌"),
                                    ),
                            )
                        }),
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
                .child(
                    div()
                        .relative()
                        .flex_1()
                        .h_full()
                        .child(kline)
                        .when(self.show_bollinger, |this| {
                            this.child(
                                div()
                                    .absolute()
                                    .inset_0()
                                    .size_full()
                                    .child(bollinger_canvas),
                            )
                        })
                        .child(
                            div()
                                .absolute()
                                .inset_0()
                                .size_full()
                                .child(trade_overlay_canvas),
                        ),
                )
                .child(price_axis);

            let vol_with_axis = div()
                .size_full()
                .flex()
                .flex_row()
                .child(div().flex_1().h_full().child(vol_chart))
                .child(vol_axis);

            let trade_info_bar = if let Some(trade) = &self.selected_trade {
                let pnl_color = if trade.pnl >= 0.0 {
                    rgb(0xf38ba8)
                } else {
                    rgb(0xa6e3a1)
                };
                let sell_info = match (&trade.sell_date, trade.sell_price) {
                    (Some(sd), Some(sp)) => format!("▼ 賣出: {} (${:.2})", sd, sp),
                    (Some(sd), None) => format!("▼ 賣出: {}", sd),
                    _ => "持倉中".to_string(),
                };

                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_2()
                    .bg(rgb(0x1e1e2e))
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(0x313244))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .font_weight(gpui_kit::FontWeight::BOLD)
                                    .text_color(rgb(0x89b4fa))
                                    .child(format!(
                                        "{} {}",
                                        trade.symbol,
                                        trade.name.clone().unwrap_or_default()
                                    )),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xf38ba8))
                                    .child(format!(
                                        "▲ 買進: {} (${:.2})",
                                        trade.buy_date, trade.buy_price
                                    )),
                            )
                            .child(div().text_xs().text_color(rgb(0xa6e3a1)).child(sell_info))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xa6adc8))
                                    .child(format!("{} 股", trade.shares)),
                            ),
                    )
                    .child(
                        div().flex().items_center().gap_2().child(
                            div()
                                .text_xs()
                                .font_weight(gpui_kit::FontWeight::BOLD)
                                .text_color(pnl_color)
                                .child(format!(
                                    "損益: {:+.0} ({:+.2}%)",
                                    trade.pnl, trade.pnl_pct
                                )),
                        ),
                    )
            } else {
                div()
            };

            div()
                .size_full()
                .border_1()
                .border_color(rgb(0x313244))
                .rounded_md()
                .p_3()
                .flex()
                .flex_col()
                .gap_2()
                .child(trade_info_bar)
                .child(chart_toolbar)
                .child(
                    div()
                        .flex_1()
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
                .child(div().h(px(130.)).child(vol_with_axis))
        } else {
            div()
                .size_full()
                .border_1()
                .border_color(rgb(0x313244))
                .rounded_md()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(0x6c7086))
                .child("點擊左側交易紀錄以檢視 K 線與買賣點標記")
        };

        let content = if trades_count > 0 {
            div().size_full().child(
                h_resizable("backtest-resizable-layout")
                    .child(
                        resizable_panel().child(
                            div()
                                .size_full()
                                .p_3()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(gpui_kit::FontWeight::BOLD)
                                        .text_color(rgb(0x89b4fa))
                                        .child(format!(
                                            "交易明細（共 {} 筆，點擊檢視 K 線圖）",
                                            trades_count
                                        )),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .size_full()
                                        .child(
                                            DataTable::new(&self.table_state)
                                                .with_size(self.font_size.table_size()),
                                        ),
                                ),
                        ),
                    )
                    .child(
                        resizable_panel()
                            .size(px(560.))
                            .size_range(px(320.)..px(1200.))
                            .child(div().size_full().p_3().child(chart_section)),
                    ),
            )
        } else {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(0x6c7086))
                .child("尚未產生交易明細，請先設定區間並點擊「執行回測」")
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(toolbar)
            .child(kpi_area)
            .child(div().flex_1().size_full().child(content))
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
