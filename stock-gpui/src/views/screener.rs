use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::chart::{BarChart, CandlestickChart};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::resizable::{h_resizable, resizable_panel};
use gpui_kit::component::table::{
    Column, ColumnSort, DataTable, TableDelegate, TableEvent, TableState,
};
use gpui_kit::base::Disableable as _;
use gpui_kit::component::Sizable as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    App, AppContext, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    Render, Styled as _, Window, div, hsla, px, rgb,
};
use stock_core::models::ScreenResult;

use crate::app::FontSizeLevel;
use crate::state::{AnalysisData, CoreState};
use crate::views::chart_viewport::{ChartViewport, compute_price_scales, compute_volume_scales};

fn get_screener_default_dates() -> (String, String) {
    let tz_offset = chrono::FixedOffset::east_opt(8 * 3600).unwrap();
    let today = chrono::Utc::now().with_timezone(&tz_offset).date_naive();
    let start = today - chrono::Duration::days(60);
    (
        start.format("%Y-%m-%d").to_string(),
        today.format("%Y-%m-%d").to_string(),
    )
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
                2 => self.results.sort_by(|a, b| {
                    a.latest_close
                        .unwrap_or(0.0)
                        .partial_cmp(&b.latest_close.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                3 => self.results.sort_by(|a, b| {
                    a.latest_volume
                        .unwrap_or(0)
                        .cmp(&b.latest_volume.unwrap_or(0))
                }),
                4 => self.results.sort_by(|a, b| {
                    a.max_bandwidth
                        .partial_cmp(&b.max_bandwidth)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                _ => self.results.sort_by(|a, b| {
                    a.latest_bandwidth
                        .partial_cmp(&b.latest_bandwidth)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
            },
            ColumnSort::Descending => match col {
                0 => self.results.sort_by(|a, b| b.symbol.cmp(&a.symbol)),
                1 => self.results.sort_by(|a, b| b.name.cmp(&a.name)),
                2 => self.results.sort_by(|a, b| {
                    b.latest_close
                        .unwrap_or(0.0)
                        .partial_cmp(&a.latest_close.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                3 => self.results.sort_by(|a, b| {
                    b.latest_volume
                        .unwrap_or(0)
                        .cmp(&a.latest_volume.unwrap_or(0))
                }),
                4 => self.results.sort_by(|a, b| {
                    b.max_bandwidth
                        .partial_cmp(&a.max_bandwidth)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                _ => self.results.sort_by(|a, b| {
                    b.latest_bandwidth
                        .partial_cmp(&a.latest_bandwidth)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
            },
            ColumnSort::Default => {
                self.results.sort_by(|a, b| a.symbol.cmp(&b.symbol));
            }
        }
    }
}

impl TableDelegate for ScreenerDelegate {
    fn columns_count(&self, _: &App) -> usize {
        6
    }

    fn rows_count(&self, _: &App) -> usize {
        self.results.len()
    }

    fn column(&self, ix: usize, _: &App) -> Column {
        let mut col = match ix {
            0 => Column::new("symbol", "股票代號").width(px(90.)).sortable(),
            1 => Column::new("name", "證券名稱").width(px(100.)).sortable(),
            2 => Column::new("close", "收盤價").width(px(90.)).sortable(),
            3 => Column::new("volume", "成交量").width(px(100.)).sortable(),
            4 => Column::new("max_bw", "最大頻寬").width(px(90.)).sortable(),
            _ => Column::new("latest_bw", "最新頻寬")
                .width(px(90.))
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
        let r = &self.results[row];
        let text = match col {
            0 => r.symbol.clone(),
            1 => r.name.clone().unwrap_or_default(),
            2 => format!("{:.2}", r.latest_close.unwrap_or(0.0)),
            3 => format!("{}", r.latest_volume.unwrap_or(0)),
            4 => format!("{:.4}", r.max_bandwidth),
            _ => format!("{:.4}", r.latest_bandwidth),
        };
        div()
            .text_size(self.font_size.size_px())
            .cursor_pointer()
            .child(text)
    }
}

pub struct ScreenerView {
    pub core_state: CoreState,
    pub start_date_input: Entity<InputState>,
    pub end_date_input: Entity<InputState>,
    pub bandwidth_input: Entity<InputState>,
    pub min_days_input: Entity<InputState>,
    pub include_etf: bool,
    pub table_state: Entity<TableState<ScreenerDelegate>>,
    pub is_running: bool,
    pub status_text: String,
    pub selected_symbol: Option<String>,
    pub selected_name: Option<String>,
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
            |this: &mut Self, table, event: &TableEvent, cx| {
                let row_opt = match event {
                    TableEvent::SelectRow(r) | TableEvent::DoubleClickedRow(r) => Some(*r),
                    TableEvent::SelectCell(r, _) | TableEvent::DoubleClickedCell(r, _) => Some(*r),
                    _ => None,
                };
                if let Some(row) = row_opt {
                    let item_opt = {
                        let d = table.read(cx).delegate();
                        d.results.get(row).cloned()
                    };
                    if let Some(item) = item_opt {
                        this.select_and_load_chart(&item.symbol, item.name, cx);
                    }
                }
            },
        )
        .detach();

        Self {
            core_state,
            start_date_input,
            end_date_input,
            bandwidth_input,
            min_days_input,
            include_etf: false,
            table_state,
            is_running: false,
            status_text: "就緒".into(),
            selected_symbol: None,
            selected_name: None,
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
        cx: &mut Context<Self>,
    ) {
        if self.selected_symbol.as_deref() == Some(symbol) && self.chart_data.is_some() {
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
        let min_days: i32 = self
            .min_days_input
            .read(cx)
            .value()
            .trim()
            .parse()
            .unwrap_or(3);

        let instrument_types = if self.include_etf {
            Some(vec!["stock".to_string(), "etf".to_string()])
        } else {
            Some(vec!["stock".to_string()])
        };

        self.is_running = true;
        self.status_text = "開始執行篩選...".into();
        cx.notify();

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(usize, usize, String)>();
        let core = self.core_state.clone();

        cx.spawn(async move |this, cx| {
            let mut screen_task = crate::state::TOKIO_RUNTIME.spawn(async move {
                core.screen_bollinger_range(
                    &start_date,
                    &end_date,
                    max_bw,
                    min_days,
                    instrument_types,
                    500_000,
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
                                        view.select_and_load_chart(&sym, name, cx);
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
                            .child(Input::new(&self.start_date_input).id("screen-start-date")),
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
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(div().text_sm().text_color(rgb(0xa6adc8)).child("頻寬:"))
                    .child(
                        div()
                            .w(px(80.))
                            .child(Input::new(&self.bandwidth_input).id("screen-bandwidth")),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(div().text_sm().text_color(rgb(0xa6adc8)).child("天數:"))
                    .child(
                        div()
                            .w(px(60.))
                            .child(Input::new(&self.min_days_input).id("screen-min-days")),
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

            let kline = CandlestickChart::new(visible_ohlcv.clone())
                .id("screener-candlestick")
                .x(|d| d.trade_date.clone())
                .open(|d| d.open_price.unwrap_or(0.0))
                .high(|d| d.high_price.unwrap_or(0.0))
                .low(|d| d.low_price.unwrap_or(0.0))
                .close(|d| d.close_price.unwrap_or(0.0))
                .body_width_ratio(0.85)
                .max_band_width(self.font_size.kline_band_width())
                .bullish(hsla(0.0, 0.8, 0.5, 1.0))
                .bearish(hsla(0.33, 0.8, 0.4, 1.0))
                .interactive(true);

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

        let content = h_resizable("screener-resizable-layout")
            .child(
                resizable_panel().child(
                    div()
                        .size_full()
                        .p_3()
                        .child(
                            DataTable::new(&self.table_state)
                                .with_size(self.font_size.table_size()),
                        ),
                ),
            )
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
