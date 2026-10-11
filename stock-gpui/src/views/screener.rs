use gpui_kit::base::Disableable as _;
use gpui_kit::component::Sizable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::chart::{BarChart, CandlestickChart};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::plot::scale::{Scale, ScaleBand, ScaleLinear};
use gpui_kit::component::resizable::{h_resizable, resizable_panel};
use gpui_kit::component::table::{
    Column, ColumnSort, DataTable, TableDelegate, TableEvent, TableState,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    App, AppContext, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    PathBuilder, Pixels, Render, Styled as _, Window, canvas, div, hsla, point, px, rgb,
};
use stock_core::models::ScreenResult;

use crate::app::FontSizeLevel;
use crate::state::{AnalysisData, CoreState, ScreenerMode, ScreenerParams, default_db_path};
use crate::views::chart_viewport::{ChartViewport, compute_price_scales, compute_volume_scales};
use stock_core::db::MarketRepository;

fn get_screener_default_dates() -> (String, String) {
    let db_path = default_db_path();
    if let Ok(repo) = MarketRepository::open(db_path.to_str().unwrap_or(""))
        && let Ok(Some(latest)) = repo.get_latest_trade_date()
    {
        return (latest.clone(), latest);
    }
    let tz_offset = chrono::FixedOffset::east_opt(8 * 3600).unwrap();
    let today = chrono::Utc::now().with_timezone(&tz_offset).date_naive();
    let today_str = today.format("%Y-%m-%d").to_string();
    (today_str.clone(), today_str)
}

pub struct ScreenerDelegate {
    pub results: Vec<ScreenResult>,
    pub sort_col: Option<usize>,
    pub sort_order: ColumnSort,
    pub font_size: FontSizeLevel,
}

impl ScreenerDelegate {
    pub fn sort_by_col(&mut self, col: usize, sort: ColumnSort) {
        self.sort_col = Some(col);
        self.sort_order = sort;
        match sort {
            ColumnSort::Ascending => match col {
                0 => self.results.sort_by(|a, b| a.symbol.cmp(&b.symbol)),
                1 => self.results.sort_by(|a, b| a.name.cmp(&b.name)),
                2 => self.results.sort_by(|a, b| a.trade_date.cmp(&b.trade_date)),
                3 => self.results.sort_by(|a, b| {
                    a.latest_close
                        .unwrap_or(0.0)
                        .partial_cmp(&b.latest_close.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                4 => self.results.sort_by(|a, b| {
                    a.latest_volume
                        .unwrap_or(0)
                        .cmp(&b.latest_volume.unwrap_or(0))
                }),
                5 => self.results.sort_by_key(|a| a.tags.join(",")),
                6 => self.results.sort_by(|a, b| {
                    a.max_bandwidth
                        .partial_cmp(&b.max_bandwidth)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                7 => self.results.sort_by(|a, b| {
                    a.latest_bandwidth
                        .partial_cmp(&b.latest_bandwidth)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                _ => self.results.sort_by_key(|a| a.break_upper),
            },
            ColumnSort::Descending => match col {
                0 => self.results.sort_by(|a, b| b.symbol.cmp(&a.symbol)),
                1 => self.results.sort_by(|a, b| b.name.cmp(&a.name)),
                2 => self.results.sort_by(|a, b| b.trade_date.cmp(&a.trade_date)),
                3 => self.results.sort_by(|a, b| {
                    b.latest_close
                        .unwrap_or(0.0)
                        .partial_cmp(&a.latest_close.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                4 => self.results.sort_by(|a, b| {
                    b.latest_volume
                        .unwrap_or(0)
                        .cmp(&a.latest_volume.unwrap_or(0))
                }),
                5 => self
                    .results
                    .sort_by_key(|b| std::cmp::Reverse(b.tags.join(","))),
                6 => self.results.sort_by(|a, b| {
                    b.max_bandwidth
                        .partial_cmp(&a.max_bandwidth)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                7 => self.results.sort_by(|a, b| {
                    b.latest_bandwidth
                        .partial_cmp(&a.latest_bandwidth)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                _ => self
                    .results
                    .sort_by_key(|b| std::cmp::Reverse(b.break_upper)),
            },
            ColumnSort::Default => {
                self.results.sort_by(|a, b| a.symbol.cmp(&b.symbol));
            }
        }
    }
}

impl TableDelegate for ScreenerDelegate {
    fn columns_count(&self, _: &App) -> usize {
        9
    }

    fn rows_count(&self, _: &App) -> usize {
        self.results.len()
    }

    fn column(&self, ix: usize, _: &App) -> Column {
        let mut col = match ix {
            0 => Column::new("symbol", "股票代號").width(px(85.)).sortable(),
            1 => Column::new("name", "證券名稱").width(px(95.)).sortable(),
            2 => Column::new("date", "訊號日期").width(px(95.)).sortable(),
            3 => Column::new("close", "收盤價").width(px(85.)).sortable(),
            4 => Column::new("volume", "成交量").width(px(90.)).sortable(),
            5 => Column::new("tags", "策略標籤").width(px(140.)).sortable(),
            6 => Column::new("max_bw", "最大頻寬").width(px(80.)).sortable(),
            7 => Column::new("latest_bw", "觸發頻寬")
                .width(px(80.))
                .sortable(),
            _ => Column::new("break_upper", "突破上軌")
                .width(px(75.))
                .sortable(),
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
        let r = &self.results[row];
        let content: gpui_kit::AnyElement = match col {
            0 => div().child(r.symbol.clone()).into_any_element(),
            1 => div()
                .child(r.name.clone().unwrap_or_default())
                .into_any_element(),
            2 => div().child(r.trade_date.clone()).into_any_element(),
            3 => div()
                .child(format!("{:.2}", r.latest_close.unwrap_or(0.0)))
                .into_any_element(),
            4 => div()
                .child(format!("{}", r.latest_volume.unwrap_or(0)))
                .into_any_element(),
            5 => {
                if r.tags.is_empty() {
                    div()
                        .text_color(rgb(0x94a3b8))
                        .child("-")
                        .into_any_element()
                } else {
                    let tag_elements = r.tags.iter().map(|tag| {
                        div()
                            .px_1p5()
                            .py_0p5()
                            .rounded_sm()
                            .bg(rgb(0x1e293b))
                            .border_1()
                            .border_color(rgb(0x38bdf8))
                            .text_color(rgb(0x38bdf8))
                            .text_xs()
                            .child(tag.clone())
                    });
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .children(tag_elements)
                        .into_any_element()
                }
            }
            6 => div()
                .child(format!("{:.4}", r.max_bandwidth))
                .into_any_element(),
            7 => div()
                .child(format!("{:.4}", r.latest_bandwidth))
                .into_any_element(),
            _ => {
                if r.break_upper {
                    div()
                        .text_color(rgb(0x10b981))
                        .font_weight(gpui_kit::FontWeight::BOLD)
                        .child("✓ 是")
                        .into_any_element()
                } else {
                    div()
                        .text_color(rgb(0x94a3b8))
                        .child("-")
                        .into_any_element()
                }
            }
        };
        div()
            .text_size(self.font_size.size_px())
            .cursor_pointer()
            .child(content)
    }
}

pub struct ScreenerView {
    pub core_state: CoreState,
    pub mode: ScreenerMode,
    pub start_date_input: Entity<InputState>,
    pub end_date_input: Entity<InputState>,
    pub bandwidth_input: Entity<InputState>,
    pub min_days_input: Entity<InputState>,
    pub min_volume_input: Entity<InputState>,
    pub short_ma_input: Entity<InputState>,
    pub mid_ma_input: Entity<InputState>,
    pub long_ma_input: Entity<InputState>,
    pub lookback_input: Entity<InputState>,
    pub include_etf: bool,
    pub break_upper_only: bool,
    pub show_bollinger: bool,
    pub table_state: Entity<TableState<ScreenerDelegate>>,
    pub is_running: bool,
    pub status_text: String,
    pub selected_symbol: Option<String>,
    pub selected_name: Option<String>,
    pub selected_signal_date: Option<String>,
    pub chart_data: Option<AnalysisData>,
    pub is_chart_loading: bool,
    pub chart_viewport: ChartViewport,
    pub font_size: FontSizeLevel,
}

impl ScreenerView {
    pub fn new(core_state: CoreState, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (default_start, default_end) = get_screener_default_dates();
        let start_date_input =
            cx.new(|cx| InputState::new(window, cx).default_value(&default_start));
        let end_date_input = cx.new(|cx| InputState::new(window, cx).default_value(&default_end));
        let bandwidth_input = cx.new(|cx| InputState::new(window, cx).default_value("0.10"));
        let min_days_input = cx.new(|cx| InputState::new(window, cx).default_value("3"));
        let min_volume_input = cx.new(|cx| InputState::new(window, cx).default_value("500"));
        let short_ma_input = cx.new(|cx| InputState::new(window, cx).default_value("5"));
        let mid_ma_input = cx.new(|cx| InputState::new(window, cx).default_value("10"));
        let long_ma_input = cx.new(|cx| InputState::new(window, cx).default_value("20"));
        let lookback_input = cx.new(|cx| InputState::new(window, cx).default_value("20"));

        let delegate = ScreenerDelegate {
            results: Vec::new(),
            sort_col: None,
            sort_order: ColumnSort::Default,
            font_size: FontSizeLevel::Medium,
        };
        let table_state = cx.new(|cx| {
            TableState::new(delegate, window, cx)
                .cell_selectable(true)
                .row_selectable(true)
        });

        for input_field in [
            &start_date_input,
            &end_date_input,
            &bandwidth_input,
            &min_days_input,
            &min_volume_input,
            &short_ma_input,
            &mid_ma_input,
            &long_ma_input,
            &lookback_input,
        ] {
            cx.subscribe(
                input_field,
                |this: &mut Self, _, event: &gpui_kit::component::input::InputEvent, cx| {
                    if matches!(
                        event,
                        gpui_kit::component::input::InputEvent::PressEnter { .. }
                    ) {
                        this.start_screen(cx);
                    }
                },
            )
            .detach();
        }

        cx.subscribe(
            &table_state,
            |this: &mut Self, table, event: &TableEvent, cx| match event {
                TableEvent::SelectRow(r) | TableEvent::DoubleClickedRow(r) => {
                    let item_opt = {
                        let d = table.read(cx).delegate();
                        d.results.get(*r).cloned()
                    };
                    if let Some(item) = item_opt {
                        this.select_and_load_chart(
                            &item.symbol,
                            item.name,
                            Some(item.trade_date),
                            cx,
                        );
                    }
                }
                TableEvent::SelectCell(r, _) | TableEvent::DoubleClickedCell(r, _) => {
                    let item_opt = {
                        let d = table.read(cx).delegate();
                        d.results.get(*r).cloned()
                    };
                    if let Some(item) = item_opt {
                        this.select_and_load_chart(
                            &item.symbol,
                            item.name,
                            Some(item.trade_date),
                            cx,
                        );
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
                            match *col {
                                0 | 5 | 6 | 7 => ColumnSort::Ascending,
                                _ => ColumnSort::Descending,
                            }
                        };
                        t.delegate_mut().sort_by_col(*col, next_sort);
                        t.refresh(cx);
                    });
                    cx.notify();
                }
                _ => {}
            },
        )
        .detach();

        Self {
            core_state,
            mode: ScreenerMode::Bollinger,
            start_date_input,
            end_date_input,
            bandwidth_input,
            min_days_input,
            min_volume_input,
            short_ma_input,
            mid_ma_input,
            long_ma_input,
            lookback_input,
            include_etf: false,
            break_upper_only: false,
            show_bollinger: true,
            table_state,
            is_running: false,
            status_text: "就緒".into(),
            selected_symbol: None,
            selected_name: None,
            selected_signal_date: None,
            chart_data: None,
            is_chart_loading: false,
            chart_viewport: ChartViewport::default(),
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

    pub fn select_and_load_chart(
        &mut self,
        symbol: &str,
        name: Option<String>,
        signal_date: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.selected_signal_date = signal_date;
        if self.selected_symbol.as_deref() == Some(symbol) && self.chart_data.is_some() {
            cx.notify();
            return;
        }
        self.selected_symbol = Some(symbol.to_string());
        self.selected_name = name;
        self.is_chart_loading = true;
        self.chart_viewport.reset();
        cx.notify();

        let sym = symbol.to_string();
        let core = self.core_state.clone();
        cx.spawn(async move |this, cx| {
            let res = core
                .get_analysis(&sym, &[5, 10, 20], Some(14), Some(12), Some(26), Some(9))
                .await;

            let _ = this.update(cx, |view, cx| {
                view.is_chart_loading = false;
                match res {
                    Ok(data) => {
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

    pub fn start_screen(&mut self, cx: &mut Context<Self>) {
        let start_date = self.start_date_input.read(cx).value().trim().to_string();
        let end_date = self.end_date_input.read(cx).value().trim().to_string();

        if start_date.is_empty() || end_date.is_empty() {
            self.status_text = "請輸入有效的起始與結束日期".into();
            cx.notify();
            return;
        }

        let max_bw: f64 = self
            .bandwidth_input
            .read(cx)
            .value()
            .trim()
            .parse()
            .unwrap_or(0.10);
        let min_days: i32 = match self.min_days_input.read(cx).value().trim().parse() {
            Ok(d) if d >= 1 => d,
            _ => {
                self.status_text = "連續天數必須為大於等於 1 的整數".into();
                cx.notify();
                return;
            }
        };

        let min_vol_lots: i64 = self
            .min_volume_input
            .read(cx)
            .value()
            .trim()
            .parse()
            .unwrap_or(500);
        let min_volume = min_vol_lots.max(0) * 1000;

        let short_period: usize = self
            .short_ma_input
            .read(cx)
            .value()
            .trim()
            .parse()
            .unwrap_or(5);
        let mid_period: usize = self
            .mid_ma_input
            .read(cx)
            .value()
            .trim()
            .parse()
            .unwrap_or(10);
        let long_period: usize = self
            .long_ma_input
            .read(cx)
            .value()
            .trim()
            .parse()
            .unwrap_or(20);
        let lookback_period: usize = self
            .lookback_input
            .read(cx)
            .value()
            .trim()
            .parse()
            .unwrap_or(20);

        let instrument_types = if self.include_etf {
            Some(vec!["stock".to_string(), "etf".to_string()])
        } else {
            Some(vec!["stock".to_string()])
        };

        let break_upper_only = self.break_upper_only;

        let params = ScreenerParams {
            mode: self.mode,
            max_bandwidth: max_bw,
            min_days,
            require_break_upper: break_upper_only,
            short_period,
            mid_period,
            long_period,
            lookback_period,
            use_high: true,
            min_volume,
            instrument_types,
        };

        // 開始新篩選時重設舊有選股與圖表結果
        self.selected_symbol = None;
        self.selected_name = None;
        self.selected_signal_date = None;
        self.chart_data = None;
        self.table_state.update(cx, |table, cx| {
            table.delegate_mut().results.clear();
            table.refresh(cx);
        });

        self.is_running = true;
        self.status_text = "開始執行篩選...".into();
        cx.notify();

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(usize, usize, String)>();
        let core = self.core_state.clone();

        cx.spawn(async move |this, cx| {
            let mut screen_task = crate::state::TOKIO_RUNTIME.spawn(async move {
                core.screen_composite_range(
                    &start_date,
                    &end_date,
                    params,
                    move |curr, tot, date, _cached| {
                        let _ = tx.send((curr, tot, date));
                    },
                )
                .await
            });

            loop {
                tokio::select! {
                    msg = rx.recv() => {
                        if let Some((curr, tot, msg_text)) = msg {
                            let _ = this.update(cx, |view, cx| {
                                view.status_text = format!("{}（進度: {}/{}）", msg_text, curr, tot);
                                cx.notify();
                            });
                        }
                    }
                    res = &mut screen_task => {
                        let final_res = match res {
                            Ok(inner) => inner,
                            Err(e) => Err(format!("背景任務異常：{}", e)),
                        };
                        let _ = this.update(cx, |view, cx| {
                            view.is_running = false;
                            match final_res {
                                Ok(data) => {
                                    let count = data.results.len();
                                    view.status_text = format!("篩選完成：共符合 {} 檔標的", count);
                                    if let Some(first) = data.results.first() {
                                        let sym = first.symbol.clone();
                                        let name = first.name.clone();
                                        let sig_date = Some(first.trade_date.clone());
                                        view.select_and_load_chart(&sym, name, sig_date, cx);
                                    } else {
                                        view.chart_data = None;
                                        view.selected_symbol = None;
                                    }
                                    view.table_state.update(cx, |table, cx| {
                                        let d = table.delegate_mut();
                                        d.results = data.results;
                                        if let Some(col) = d.sort_col {
                                            d.sort_by_col(col, d.sort_order);
                                        }
                                        table.refresh(cx);
                                    });
                                }
                                Err(e) => {
                                    view.status_text = format!("篩選失敗：{}", e);
                                    view.chart_data = None;
                                    view.selected_symbol = None;
                                }
                            }
                            cx.notify();
                        });
                        break;
                    }
                }
            }
        })
        .detach();
    }

    pub fn cancel_screen(&mut self, cx: &mut Context<Self>) {
        self.core_state.cancel();
        self.status_text = "已送出取消篩選請求...".into();
        cx.notify();
    }

    pub fn clear_cache(&mut self, cx: &mut Context<Self>) {
        let core = self.core_state.clone();
        cx.spawn(async move |this, cx| {
            let res = core.clear_screener_cache().await;
            let _ = this.update(cx, |view, cx| {
                match res {
                    Ok(count) => {
                        view.status_text = format!("已清除選股快取，共刪除 {} 筆", count);
                    }
                    Err(e) => {
                        view.status_text = format!("清除快取失敗：{}", e);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for ScreenerView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_running = self.is_running;

        let mode_buttons = div()
            .flex()
            .items_center()
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui_kit::FontWeight::BOLD)
                    .text_color(rgb(0xcdd6f4))
                    .child("策略模式:"),
            )
            .child(
                Button::new("btn-mode-bollinger")
                    .label("布林通道擠壓")
                    .when(self.mode == ScreenerMode::Bollinger, |b| b.primary())
                    .when(self.mode != ScreenerMode::Bollinger, |b| b.secondary())
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.mode = ScreenerMode::Bollinger;
                        cx.notify();
                    })),
            )
            .child(
                Button::new("btn-mode-ma")
                    .label("均線多頭排列")
                    .when(self.mode == ScreenerMode::MaAlignment, |b| b.primary())
                    .when(self.mode != ScreenerMode::MaAlignment, |b| b.secondary())
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.mode = ScreenerMode::MaAlignment;
                        cx.notify();
                    })),
            )
            .child(
                Button::new("btn-mode-new-high")
                    .label("創波段新高")
                    .when(self.mode == ScreenerMode::NewHigh, |b| b.primary())
                    .when(self.mode != ScreenerMode::NewHigh, |b| b.secondary())
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.mode = ScreenerMode::NewHigh;
                        cx.notify();
                    })),
            )
            .child(
                Button::new("btn-mode-composite")
                    .label("複合選股 (多頭+創新高)")
                    .when(self.mode == ScreenerMode::Composite, |b| b.primary())
                    .when(self.mode != ScreenerMode::Composite, |b| b.secondary())
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.mode = ScreenerMode::Composite;
                        cx.notify();
                    })),
            );

        let toolbar = div()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .border_b_1()
            .border_color(rgb(0x313244))
            .child(mode_buttons)
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(div().text_sm().text_color(rgb(0xa6adc8)).child("起始:"))
                            .child(
                                div().w(px(120.)).child(
                                    Input::new(&self.start_date_input).id("screen-start-date"),
                                ),
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
                                    .child(Input::new(&self.end_date_input).id("screen-end-date")),
                            ),
                    )
                    .when(self.mode == ScreenerMode::Bollinger, |d| {
                        d.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(div().text_sm().text_color(rgb(0xa6adc8)).child("頻寬:"))
                                .child(div().w(px(80.)).child(
                                    Input::new(&self.bandwidth_input).id("screen-bandwidth"),
                                )),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(div().text_sm().text_color(rgb(0xa6adc8)).child("天數:"))
                                .child(
                                    div().w(px(60.)).child(
                                        Input::new(&self.min_days_input).id("screen-min-days"),
                                    ),
                                ),
                        )
                    })
                    .when(
                        self.mode == ScreenerMode::MaAlignment
                            || self.mode == ScreenerMode::Composite,
                        |d| {
                            d.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(div().text_sm().text_color(rgb(0xa6adc8)).child("短均:"))
                                    .child(div().w(px(50.)).child(
                                        Input::new(&self.short_ma_input).id("screen-short-ma"),
                                    )),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(div().text_sm().text_color(rgb(0xa6adc8)).child("中均:"))
                                    .child(
                                        div().w(px(50.)).child(
                                            Input::new(&self.mid_ma_input).id("screen-mid-ma"),
                                        ),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(div().text_sm().text_color(rgb(0xa6adc8)).child("長均:"))
                                    .child(div().w(px(50.)).child(
                                        Input::new(&self.long_ma_input).id("screen-long-ma"),
                                    )),
                            )
                        },
                    )
                    .when(
                        self.mode == ScreenerMode::NewHigh || self.mode == ScreenerMode::Composite,
                        |d| {
                            d.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0xa6adc8))
                                            .child("突破天數:"),
                                    )
                                    .child(div().w(px(60.)).child(
                                        Input::new(&self.lookback_input).id("screen-lookback"),
                                    )),
                            )
                        },
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(div().text_sm().text_color(rgb(0xa6adc8)).child("量(張):"))
                            .child(
                                div().w(px(70.)).child(
                                    Input::new(&self.min_volume_input).id("screen-min-volume"),
                                ),
                            ),
                    )
                    .child(
                        Checkbox::new("screener-include-etf")
                            .label("包含 ETF")
                            .checked(self.include_etf)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.include_etf = *checked;
                                cx.notify();
                            })),
                    )
                    .when(self.mode == ScreenerMode::Bollinger, |d| {
                        d.child(
                            Checkbox::new("screener-break-upper-only")
                                .label("僅突破上軌")
                                .checked(self.break_upper_only)
                                .on_click(cx.listener(|this, checked, _, cx| {
                                    this.break_upper_only = *checked;
                                    cx.notify();
                                })),
                        )
                    })
                    .child(
                        Button::new("btn-start-screen")
                            .label(if is_running {
                                "篩選中..."
                            } else {
                                "開始選股"
                            })
                            .primary()
                            .disabled(is_running)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.start_screen(cx);
                            })),
                    )
                    .when(is_running, |this| {
                        this.child(
                            Button::new("btn-cancel-screen")
                                .label("取消篩選")
                                .danger()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.cancel_screen(cx);
                                })),
                        )
                    })
                    .when(!is_running, |this| {
                        this.child(
                            Button::new("btn-clear-screen-cache")
                                .label("清除快取")
                                .secondary()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.clear_cache(cx);
                                })),
                        )
                    })
                    .child(
                        div()
                            .text_color(rgb(0xa6adc8))
                            .text_sm()
                            .child(self.status_text.clone()),
                    ),
            );

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
                .id("screener-candlestick")
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

                    let build_band_path =
                        |series: &[Option<f64>]| -> Option<gpui_kit::Path<Pixels>> {
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
                            if started { builder.build().ok() } else { None }
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

            let vol_chart = BarChart::new(visible_ohlcv.clone())
                .id("screener-volume")
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
                    if let Some(sig_date) = &self.selected_signal_date {
                        format!(
                            "顯示 {} / {} 根（{} ~ {}） | ⚡ 訊號日: {}",
                            visible_ohlcv.len(),
                            total_candles,
                            first.trade_date,
                            last.trade_date,
                            sig_date
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
                } else if let Some(sig_date) = &self.selected_signal_date {
                    format!(
                        "顯示 {} / {} 根 | ⚡ 訊號日: {}",
                        visible_ohlcv.len(),
                        total_candles,
                        sig_date
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
                            Button::new("btn-screener-zoom-in")
                                .label("🔍 放大 (+)")
                                .secondary()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.chart_viewport.zoom_in();
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("btn-screener-zoom-out")
                                .label("🔍 縮小 (-)")
                                .secondary()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.chart_viewport.zoom_out(total_candles);
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("btn-screener-pan-left")
                                .label("◀ 向左平移")
                                .secondary()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.chart_viewport.pan_left(total_candles);
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("btn-screener-pan-right")
                                .label("向右平移 ▶")
                                .secondary()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.chart_viewport.pan_right();
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("btn-screener-reset")
                                .label("⟲ 回最新")
                                .secondary()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.chart_viewport.reset();
                                    cx.notify();
                                })),
                        )
                        .child(div().w(px(1.)).h(px(18.)).bg(rgb(0xcbd5e1)))
                        .child(
                            Checkbox::new("screener-show-bollinger")
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
                .child(div().relative().flex_1().h_full().child(kline).when(
                    self.show_bollinger,
                    |this| {
                        this.child(
                            div()
                                .absolute()
                                .inset_0()
                                .size_full()
                                .child(bollinger_canvas),
                        )
                    },
                ))
                .child(price_axis);

            let vol_with_axis = div()
                .size_full()
                .flex()
                .flex_row()
                .child(div().flex_1().h_full().child(vol_chart))
                .child(vol_axis);

            div()
                .size_full()
                .border_1()
                .border_color(rgb(0x313244))
                .rounded_md()
                .p_3()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
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
                .child("點擊左側股票列以即時檢視 K 線與走勢圖")
        };

        let content =
            h_resizable("screener-resizable-layout")
                .child(resizable_panel().child(div().size_full().p_3().child(
                    DataTable::new(&self.table_state).with_size(self.font_size.table_size()),
                )))
                .child(
                    resizable_panel()
                        .size(px(520.))
                        .size_range(px(320.)..px(1200.))
                        .child(div().size_full().p_3().child(chart_section)),
                );

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(toolbar)
            .child(div().flex_1().size_full().child(content))
    }
}
