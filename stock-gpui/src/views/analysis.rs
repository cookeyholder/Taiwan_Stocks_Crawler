use gpui_kit::component::Sizable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::chart::{BarChart, CandlestickChart, LineChart};
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

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::app::FontSizeLevel;
use crate::state::{AnalysisData, CoreState, SecurityItem};
use crate::views::chart_viewport::{ChartViewport, compute_price_scales, compute_volume_scales};
use stock_core::models::InstitutionalRecord;

fn format_shares_as_lots(shares: i64) -> String {
    if shares == 0 {
        "0張".to_string()
    } else {
        format!("{:+.3}張", shares as f64 / 1000.0)
    }
}

#[derive(Clone, Debug)]
pub struct IndicatorPoint {
    pub date: String,
    pub val: f64,
}

pub struct AnalysisSecuritiesDelegate {
    pub all_items: Vec<SecurityItem>,
    pub filtered_items: Vec<SecurityItem>,
    pub sort_col: Option<usize>,
    pub sort_order: ColumnSort,
    pub include_etf: bool,
    pub show_watchlist_only: bool,
    pub watchlist: HashSet<String>,
    pub search_query: String,
    pub font_size: FontSizeLevel,
}

impl Default for AnalysisSecuritiesDelegate {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisSecuritiesDelegate {
    pub fn new() -> Self {
        Self {
            all_items: Vec::new(),
            filtered_items: Vec::new(),
            sort_col: None,
            sort_order: ColumnSort::Default,
            include_etf: false,
            show_watchlist_only: false,
            watchlist: HashSet::new(),
            search_query: String::new(),
            font_size: FontSizeLevel::Medium,
        }
    }

    pub fn apply_filter(&mut self) {
        let q = self.search_query.trim().to_lowercase();
        self.filtered_items = self
            .all_items
            .iter()
            .filter(|s| {
                if self.show_watchlist_only && !self.watchlist.contains(&s.symbol) {
                    return false;
                }
                let is_etf = s.symbol.starts_with("00") || s.symbol.len() > 4;
                if !self.include_etf && is_etf {
                    return false;
                }
                if q.is_empty() {
                    true
                } else {
                    s.symbol.to_lowercase().contains(&q)
                        || s.name.to_lowercase().contains(&q)
                        || s.market.to_lowercase().contains(&q)
                }
            })
            .cloned()
            .collect();

        if let Some(col) = self.sort_col {
            self.sort_by_col(col, self.sort_order);
        } else {
            self.sort_by_col(0, ColumnSort::Default);
        }
    }

    pub fn sort_by_col(&mut self, col: usize, sort: ColumnSort) {
        self.sort_col = Some(col);
        self.sort_order = sort;
        match sort {
            ColumnSort::Ascending => match col {
                0 => self.filtered_items.sort_by(|a, b| a.symbol.cmp(&b.symbol)),
                1 => self.filtered_items.sort_by(|a, b| a.market.cmp(&b.market)),
                _ => self.filtered_items.sort_by(|a, b| a.name.cmp(&b.name)),
            },
            ColumnSort::Descending => match col {
                0 => self.filtered_items.sort_by(|a, b| b.symbol.cmp(&a.symbol)),
                1 => self.filtered_items.sort_by(|a, b| b.market.cmp(&a.market)),
                _ => self.filtered_items.sort_by(|a, b| b.name.cmp(&a.name)),
            },
            ColumnSort::Default => {
                // 股票在前段，ETF在後段
                self.filtered_items.sort_by(|a, b| {
                    let a_is_etf = a.symbol.starts_with("00") || a.symbol.len() > 4;
                    let b_is_etf = b.symbol.starts_with("00") || b.symbol.len() > 4;
                    match (a_is_etf, b_is_etf) {
                        (false, true) => std::cmp::Ordering::Less,
                        (true, false) => std::cmp::Ordering::Greater,
                        _ => a.symbol.cmp(&b.symbol),
                    }
                });
            }
        }
    }
}

impl TableDelegate for AnalysisSecuritiesDelegate {
    fn columns_count(&self, _: &App) -> usize {
        3
    }

    fn rows_count(&self, _: &App) -> usize {
        self.filtered_items.len()
    }

    fn column(&self, ix: usize, _: &App) -> Column {
        let mut col = match ix {
            0 => Column::new("symbol", "代號").width(px(80.)).sortable(),
            1 => Column::new("market", "市場").width(px(60.)).sortable(),
            _ => Column::new("name", "證券名稱").width(px(140.)).sortable(),
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
        let item = &self.filtered_items[row];
        let is_fav = self.watchlist.contains(&item.symbol);
        let text = match col {
            0 => {
                if is_fav {
                    format!("★ {}", item.symbol)
                } else {
                    item.symbol.clone()
                }
            }
            1 => item.market.clone(),
            _ => item.name.clone(),
        };
        div()
            .text_size(self.font_size.size_px())
            .cursor_pointer()
            .when(col == 0 && is_fav, |this| this.text_color(rgb(0xeab308)))
            .child(text)
    }
}

pub struct AnalysisView {
    pub core_state: CoreState,
    pub search_input: Entity<InputState>,
    pub include_etf: bool,
    pub table_state: Entity<TableState<AnalysisSecuritiesDelegate>>,
    pub current_symbol: String,
    pub current_data: Option<AnalysisData>,
    pub is_loading: bool,
    pub error_msg: Option<String>,
    pub chart_viewport: ChartViewport,
    pub font_size: FontSizeLevel,
    pub show_bollinger: bool,
    pub show_ma: bool,
    pub show_macd: bool,
    pub show_institutional: bool,
    pub show_watchlist_only: bool,
    pub is_dark: bool,
    pub hovered_candle_idx: Option<usize>,
    pub hover_pos: Option<gpui_kit::Point<Pixels>>,
    pub main_chart_bounds: Arc<Mutex<Option<gpui_kit::Bounds<Pixels>>>>,
}

impl AnalysisView {
    pub fn new(core_state: CoreState, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("搜尋股票代號或名稱..."));

        let delegate = AnalysisSecuritiesDelegate::new();
        let table_state = cx.new(|cx| {
            TableState::new(delegate, window, cx)
                .cell_selectable(true)
                .row_selectable(true)
        });

        // 搜尋輸入框事件
        cx.subscribe(
            &search_input,
            |this: &mut Self, input, event: &gpui_kit::component::input::InputEvent, cx| {
                if matches!(
                    event,
                    gpui_kit::component::input::InputEvent::Change
                        | gpui_kit::component::input::InputEvent::PressEnter { .. }
                ) {
                    let q = input.read(cx).value().to_string();
                    this.table_state.update(cx, |table, cx| {
                        let d = table.delegate_mut();
                        d.search_query = q;
                        d.apply_filter();
                        table.refresh(cx);
                    });
                    cx.notify();
                }
            },
        )
        .detach();

        // 點擊左側列表選取證券
        cx.subscribe(
            &table_state,
            |this: &mut Self, table, event: &TableEvent, cx| {
                let row_opt = match event {
                    TableEvent::SelectRow(r) | TableEvent::DoubleClickedRow(r) => Some(*r),
                    TableEvent::SelectCell(r, _) | TableEvent::DoubleClickedCell(r, _) => Some(*r),
                    _ => None,
                };
                if let Some(row) = row_opt {
                    let sym_opt = {
                        let d = table.read(cx).delegate();
                        d.filtered_items.get(row).map(|item| item.symbol.clone())
                    };
                    if let Some(sym) = sym_opt {
                        this.fetch_symbol(sym, cx);
                    }
                }
            },
        )
        .detach();

        let mut view = Self {
            core_state,
            search_input,
            include_etf: false,
            table_state,
            current_symbol: "2330".into(),
            current_data: None,
            is_loading: false,
            error_msg: None,
            chart_viewport: ChartViewport::default(),
            font_size: FontSizeLevel::Medium,
            show_bollinger: true,
            show_ma: true,
            show_macd: true,
            show_institutional: true,
            show_watchlist_only: false,
            is_dark: false,
            hovered_candle_idx: None,
            hover_pos: None,
            main_chart_bounds: Arc::new(Mutex::new(None)),
        };

        view.load_securities(cx);
        view.fetch_symbol("2330".into(), cx);
        view
    }

    pub fn set_dark(&mut self, is_dark: bool, cx: &mut Context<Self>) {
        self.is_dark = is_dark;
        cx.notify();
    }

    pub fn set_font_size(&mut self, font_size: FontSizeLevel, cx: &mut Context<Self>) {
        self.font_size = font_size;
        self.table_state.update(cx, |table, cx| {
            table.delegate_mut().font_size = font_size;
            table.refresh(cx);
        });
        cx.notify();
    }

    pub fn load_securities(&mut self, cx: &mut Context<Self>) {
        let core = self.core_state.clone();
        cx.spawn(async move |this, cx| {
            let res = core.get_securities().await;
            let watchlist = core.get_watchlist_symbols();
            let _ = this.update(cx, |view, cx| {
                if let Ok(items) = res {
                    view.table_state.update(cx, |table, cx| {
                        let d = table.delegate_mut();
                        d.all_items = items;
                        d.watchlist = watchlist;
                        d.apply_filter();
                        table.refresh(cx);
                    });
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn fetch_symbol(&mut self, symbol: String, cx: &mut Context<Self>) {
        if self.current_symbol == symbol && self.current_data.is_some() {
            return;
        }
        self.current_symbol = symbol.clone();
        self.is_loading = true;
        self.error_msg = None;
        self.chart_viewport.reset();
        self.hovered_candle_idx = None;
        self.hover_pos = None;
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
        let count = self.table_state.read(cx).delegate().filtered_items.len();

        let (
            panel_bg,
            border_col,
            toolbar_bg,
            header_bg,
            text_primary,
            text_secondary,
            text_title,
            price_text,
            divider_col,
        ) = if self.is_dark {
            (
                rgb(0x181825),
                rgb(0x313244),
                rgb(0x1e1e2e),
                rgb(0x181825),
                rgb(0xcdd6f4),
                rgb(0xa6adc8),
                rgb(0x89b4fa),
                rgb(0xcdd6f4),
                rgb(0x45475a),
            )
        } else {
            (
                rgb(0xf8fafc),
                rgb(0xe2e8f0),
                rgb(0xf1f5f9),
                rgb(0xf8fafc),
                rgb(0x1e293b),
                rgb(0x64748b),
                rgb(0x1d4ed8),
                rgb(0x0f172a),
                rgb(0xcbd5e1),
            )
        };

        // 左側工具列
        let total_fav_count = self.core_state.get_watchlist_symbols().len();
        let total_all_count = self.table_state.read(cx).delegate().all_items.len();
        let watchlist_tabs = div()
            .flex()
            .items_center()
            .gap_2()
            .child(
                Button::new("tab-filter-all")
                    .label(format!("全部 ({})", total_all_count))
                    .when(!self.show_watchlist_only, |btn| btn.primary())
                    .when(self.show_watchlist_only, |btn| btn.secondary())
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.show_watchlist_only = false;
                        this.table_state.update(cx, |table, cx| {
                            let d = table.delegate_mut();
                            d.show_watchlist_only = false;
                            d.apply_filter();
                            table.refresh(cx);
                        });
                        cx.notify();
                    })),
            )
            .child(
                Button::new("tab-filter-watchlist")
                    .label(format!("★ 自選股 ({})", total_fav_count))
                    .when(self.show_watchlist_only, |btn| btn.primary())
                    .when(!self.show_watchlist_only, |btn| btn.secondary())
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.show_watchlist_only = true;
                        let watchlist = this.core_state.get_watchlist_symbols();
                        this.table_state.update(cx, |table, cx| {
                            let d = table.delegate_mut();
                            d.show_watchlist_only = true;
                            d.watchlist = watchlist;
                            d.apply_filter();
                            table.refresh(cx);
                        });
                        cx.notify();
                    })),
            );

        let left_toolbar = div()
            .flex()
            .flex_col()
            .gap_2()
            .p_2()
            .border_b_1()
            .border_color(border_col)
            .child(watchlist_tabs)
            .child(
                div()
                    .w_full()
                    .child(Input::new(&self.search_input).id("analysis-search-input")),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        Checkbox::new("analysis-include-etf")
                            .label("包含 ETF")
                            .checked(self.include_etf)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.include_etf = *checked;
                                this.table_state.update(cx, |table, cx| {
                                    let d = table.delegate_mut();
                                    d.include_etf = *checked;
                                    d.apply_filter();
                                    table.refresh(cx);
                                });
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui_kit::FontWeight::MEDIUM)
                            .text_color(text_secondary)
                            .child(format!("共 {} 檔", count)),
                    ),
            );

        // 左側證券清單面板
        let left_panel =
            div()
                .size_full()
                .flex()
                .flex_col()
                .bg(panel_bg)
                .border_r_1()
                .border_color(border_col)
                .child(left_toolbar)
                .child(div().flex_1().size_full().p_1().child(
                    DataTable::new(&self.table_state).with_size(self.font_size.table_size()),
                ));

        // 右側圖表與指標內容
        let right_content = if self.is_loading {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(0x2563eb))
                .child("載入分析數據中...")
        } else if let Some(err) = &self.error_msg {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(0xdc2626))
                .child(format!("查詢錯誤：{}", err))
        } else if let Some(data) = &self.current_data {
            let ohlcv = data.ohlcv.clone();
            let total_candles = ohlcv.len();
            let (start_idx, end_idx) = self.chart_viewport.slice_range(total_candles);
            let visible_ohlcv = ohlcv[start_idx..end_idx].to_vec();

            let price_scales = compute_price_scales(&visible_ohlcv);
            let vol_scales = compute_volume_scales(&visible_ohlcv);

            let max_band_w = self.font_size.kline_band_width();
            let axis_font_size = self.font_size.axis_font_size();
            let axis_w = self.font_size.axis_width();

            let kline = CandlestickChart::new(visible_ohlcv.clone())
                .id("analysis-candlestick")
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

            let vol_chart = BarChart::new(visible_ohlcv.clone())
                .id("analysis-volume")
                .band(|d| d.trade_date.clone())
                .value(|d| d.volume_shares.unwrap_or(0) as f64)
                .interactive(true);

            let safe_slice = |vec: &[Option<f64>]| -> Vec<Option<f64>> {
                (start_idx..end_idx)
                    .map(|i| vec.get(i).copied().flatten())
                    .collect()
            };

            let rsi_slice = safe_slice(&data.rsi);
            let rsi_points: Vec<IndicatorPoint> = visible_ohlcv
                .iter()
                .zip(rsi_slice.iter())
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

            // 布林通道畫布
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

            // 均線 (MA5, MA10, MA20) 畫布
            let visible_ma5 = safe_slice(&data.ma5);
            let visible_ma10 = safe_slice(&data.ma10);
            let visible_ma20 = safe_slice(&data.ma20);
            let visible_ohlcv_ma = visible_ohlcv.clone();

            let ma_canvas = canvas(
                move |_, _, _| {},
                move |bounds, _, window, _cx| {
                    let height = bounds.size.height.as_f32() - 18.0;
                    if height <= 0.0
                        || bounds.size.width.as_f32() <= 0.0
                        || visible_ohlcv_ma.is_empty()
                    {
                        return;
                    }

                    let x_scale = ScaleBand::new(
                        visible_ohlcv_ma.iter().map(|d| d.trade_date.clone()),
                        [0., bounds.size.width.as_f32()],
                    )
                    .max_band_width(max_band_w.as_f32())
                    .padding_inner(0.4)
                    .padding_outer(0.2);

                    let all_values: Vec<f64> = visible_ohlcv_ma
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

                    let build_line_path =
                        |series: &[Option<f64>]| -> Option<gpui_kit::Path<Pixels>> {
                            let mut builder = PathBuilder::stroke(px(1.5));
                            let mut started = false;
                            for (d, val_opt) in visible_ohlcv_ma.iter().zip(series.iter()) {
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
                        // MA5: 橙黃色
                        if let Some(p) = build_line_path(&visible_ma5) {
                            window.paint_path(p, hsla(0.12, 0.95, 0.55, 1.0));
                        }
                        // MA10: 天藍色
                        if let Some(p) = build_line_path(&visible_ma10) {
                            window.paint_path(p, hsla(0.58, 0.90, 0.55, 1.0));
                        }
                        // MA20: 洋紅色
                        if let Some(p) = build_line_path(&visible_ma20) {
                            window.paint_path(p, hsla(0.85, 0.85, 0.60, 1.0));
                        }
                    });
                },
            )
            .size_full();

            // 十字游標畫布
            let visible_ohlcv_crosshair = visible_ohlcv.clone();
            let hovered_idx = self.hovered_candle_idx;
            let hover_pos = self.hover_pos;
            let bounds_ref = self.main_chart_bounds.clone();

            let crosshair_canvas = canvas(
                move |_, _, _| {},
                move |bounds, _, window, _cx| {
                    bounds_ref.lock().unwrap().replace(bounds);

                    let height = bounds.size.height.as_f32() - 18.0;
                    if height <= 0.0
                        || bounds.size.width.as_f32() <= 0.0
                        || visible_ohlcv_crosshair.is_empty()
                    {
                        return;
                    }

                    let (Some(target_abs_idx), Some(hover_pt)) = (hovered_idx, hover_pos) else {
                        return;
                    };

                    if hover_pt.x < bounds.origin.x
                        || hover_pt.x > bounds.origin.x + bounds.size.width
                        || hover_pt.y < bounds.origin.y
                        || hover_pt.y > bounds.origin.y + bounds.size.height
                    {
                        return;
                    }

                    if target_abs_idx < start_idx || target_abs_idx >= end_idx {
                        return;
                    }

                    let x_scale = ScaleBand::new(
                        visible_ohlcv_crosshair.iter().map(|d| d.trade_date.clone()),
                        [0., bounds.size.width.as_f32()],
                    )
                    .max_band_width(max_band_w.as_f32())
                    .padding_inner(0.4)
                    .padding_outer(0.2);

                    let col_idx = target_abs_idx - start_idx;
                    if let Some(candle) = visible_ohlcv_crosshair.get(col_idx)
                        && let Some(xt) = x_scale.tick(&candle.trade_date)
                    {
                        let half_band = x_scale.band_width() / 2.0;
                        let line_x = bounds.origin.x + px(xt + half_band);

                        window.with_content_mask(
                            Some(gpui_kit::ContentMask { bounds }),
                            |window| {
                                // 垂直十字線（微透明灰色，對齊 K 棒中心）
                                let mut v_line = PathBuilder::stroke(px(1.0));
                                v_line.move_to(point(line_x, bounds.origin.y));
                                v_line.line_to(point(line_x, bounds.origin.y + bounds.size.height));
                                if let Ok(path) = v_line.build() {
                                    window.paint_path(path, hsla(0.6, 0.2, 0.70, 0.75));
                                }

                                // 水平十字線（跟隨游標 Y 座標）
                                let mut h_line = PathBuilder::stroke(px(1.0));
                                h_line.move_to(point(bounds.origin.x, hover_pt.y));
                                h_line.line_to(point(
                                    bounds.origin.x + bounds.size.width,
                                    hover_pt.y,
                                ));
                                if let Ok(path) = h_line.build() {
                                    window.paint_path(path, hsla(0.6, 0.2, 0.70, 0.75));
                                }
                            },
                        );
                    }
                },
            )
            .size_full();

            // MACD 畫布
            let visible_macd_line = safe_slice(&data.macd_line);
            let visible_macd_signal = safe_slice(&data.macd_signal);
            let visible_macd_hist = safe_slice(&data.macd_histogram);
            let visible_ohlcv_macd = visible_ohlcv.clone();

            let mut macd_all_vals: Vec<f64> = Vec::new();
            for v in visible_macd_line
                .iter()
                .flatten()
                .chain(visible_macd_signal.iter().flatten())
                .chain(visible_macd_hist.iter().flatten())
            {
                macd_all_vals.push(*v);
            }
            macd_all_vals.push(0.0);

            let macd_canvas = canvas(
                move |_, _, _| {},
                move |bounds, _, window, _cx| {
                    let height = bounds.size.height.as_f32() - 18.0;
                    if height <= 0.0
                        || bounds.size.width.as_f32() <= 0.0
                        || visible_ohlcv_macd.is_empty()
                    {
                        return;
                    }

                    let x_scale = ScaleBand::new(
                        visible_ohlcv_macd.iter().map(|d| d.trade_date.clone()),
                        [0., bounds.size.width.as_f32()],
                    )
                    .max_band_width(max_band_w.as_f32())
                    .padding_inner(0.4)
                    .padding_outer(0.2);

                    if macd_all_vals.is_empty() {
                        return;
                    }

                    let y_scale = ScaleLinear::new(macd_all_vals.clone(), [height, 10.]);
                    let half_band = x_scale.band_width() / 2.0;

                    window.with_content_mask(Some(gpui_kit::ContentMask { bounds }), |window| {
                        // 零軸
                        if let Some(y_zero) = y_scale.tick(&0.0) {
                            let mut zero_line = PathBuilder::stroke(px(1.0));
                            zero_line.move_to(point(bounds.origin.x, bounds.origin.y + px(y_zero)));
                            zero_line.line_to(point(
                                bounds.origin.x + bounds.size.width,
                                bounds.origin.y + px(y_zero),
                            ));
                            if let Ok(path) = zero_line.build() {
                                window.paint_path(path, hsla(0.0, 0.0, 0.5, 0.4));
                            }
                        }

                        // 柱狀圖 (OSC)
                        let bar_w = (x_scale.band_width() * 0.7).max(1.0);
                        for (d, hist_opt) in visible_ohlcv_macd.iter().zip(visible_macd_hist.iter())
                        {
                            if let Some(hist) = hist_opt
                                && let Some(xt) = x_scale.tick(&d.trade_date)
                                && let (Some(y_val), Some(y_zero)) =
                                    (y_scale.tick(hist), y_scale.tick(&0.0))
                            {
                                let top = y_val.min(y_zero);
                                let bar_h = (y_val - y_zero).abs().max(1.0);
                                let bar_x = bounds.origin.x + px(xt + half_band);
                                let mut bar = PathBuilder::stroke(px(bar_w));
                                bar.move_to(point(bar_x, bounds.origin.y + px(top)));
                                bar.line_to(point(bar_x, bounds.origin.y + px(top + bar_h)));
                                let col = if *hist >= 0.0 {
                                    hsla(0.0, 0.8, 0.5, 0.8)
                                } else {
                                    hsla(0.33, 0.8, 0.4, 0.8)
                                };
                                if let Ok(p) = bar.build() {
                                    window.paint_path(p, col);
                                }
                            }
                        }

                        // DIF & Signal 折線
                        let build_line =
                            |series: &[Option<f64>]| -> Option<gpui_kit::Path<Pixels>> {
                                let mut builder = PathBuilder::stroke(px(1.5));
                                let mut started = false;
                                for (d, val_opt) in visible_ohlcv_macd.iter().zip(series.iter()) {
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

                        // DIF (快線): 橙黃色
                        if let Some(p) = build_line(&visible_macd_line) {
                            window.paint_path(p, hsla(0.12, 0.95, 0.55, 1.0));
                        }
                        // Signal (慢線): 紫色
                        if let Some(p) = build_line(&visible_macd_signal) {
                            window.paint_path(p, hsla(0.75, 0.85, 0.60, 1.0));
                        }
                    });
                },
            )
            .size_full();

            // 三大法人買賣超柱狀圖畫布
            let visible_inst: Vec<Option<InstitutionalRecord>> = (start_idx..end_idx)
                .map(|i| data.institutional.get(i).cloned().flatten())
                .collect();
            let visible_ohlcv_inst = visible_ohlcv.clone();

            let mut inst_all_vals: Vec<f64> = Vec::new();
            for r in visible_inst.iter().flatten() {
                inst_all_vals.push(r.total_net as f64 / 1000.0);
            }
            if inst_all_vals.is_empty() {
                inst_all_vals.push(10.0);
                inst_all_vals.push(-10.0);
            } else {
                inst_all_vals.push(0.0);
            }

            let institutional_canvas = canvas(
                move |_, _, _| {},
                move |bounds, _, window, _cx| {
                    let height = bounds.size.height.as_f32() - 18.0;
                    if height <= 0.0
                        || bounds.size.width.as_f32() <= 0.0
                        || visible_ohlcv_inst.is_empty()
                    {
                        return;
                    }

                    let x_scale = ScaleBand::new(
                        visible_ohlcv_inst.iter().map(|d| d.trade_date.clone()),
                        [0., bounds.size.width.as_f32()],
                    )
                    .max_band_width(max_band_w.as_f32())
                    .padding_inner(0.4)
                    .padding_outer(0.2);

                    let y_scale = ScaleLinear::new(inst_all_vals.clone(), [height, 10.]);
                    let half_band = x_scale.band_width() / 2.0;

                    window.with_content_mask(Some(gpui_kit::ContentMask { bounds }), |window| {
                        // 零軸
                        if let Some(y_zero) = y_scale.tick(&0.0) {
                            let mut zero_line = PathBuilder::stroke(px(1.0));
                            zero_line.move_to(point(bounds.origin.x, bounds.origin.y + px(y_zero)));
                            zero_line.line_to(point(
                                bounds.origin.x + bounds.size.width,
                                bounds.origin.y + px(y_zero),
                            ));
                            if let Ok(path) = zero_line.build() {
                                window.paint_path(path, hsla(0.0, 0.0, 0.5, 0.4));
                            }
                        }

                        // 柱狀圖（買超紅、賣超綠）
                        let bar_w = (x_scale.band_width() * 0.7).max(1.0);
                        for (d, inst_opt) in visible_ohlcv_inst.iter().zip(visible_inst.iter()) {
                            if let Some(inst) = inst_opt
                                && let Some(xt) = x_scale.tick(&d.trade_date)
                            {
                                let val = inst.total_net as f64 / 1000.0;
                                if let (Some(y_val), Some(y_zero)) =
                                    (y_scale.tick(&val), y_scale.tick(&0.0))
                                {
                                    let top = y_val.min(y_zero);
                                    let bar_h = (y_val - y_zero).abs().max(1.0);
                                    let bar_x = bounds.origin.x + px(xt + half_band);
                                    let mut bar = PathBuilder::stroke(px(bar_w));
                                    bar.move_to(point(bar_x, bounds.origin.y + px(top)));
                                    bar.line_to(point(bar_x, bounds.origin.y + px(top + bar_h)));
                                    let col = if val >= 0.0 {
                                        hsla(0.0, 0.8, 0.5, 0.8)
                                    } else {
                                        hsla(0.33, 0.8, 0.4, 0.8)
                                    };
                                    if let Ok(p) = bar.build() {
                                        window.paint_path(p, col);
                                    }
                                }
                            }
                        }
                    });
                },
            )
            .size_full();

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
                .border_color(border_col)
                .children(
                    price_scales
                        .map(|scales| {
                            scales
                                .into_iter()
                                .map(|p| {
                                    div()
                                        .text_size(axis_font_size)
                                        .font_weight(gpui_kit::FontWeight::BOLD)
                                        .text_color(text_secondary)
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
                .border_color(border_col)
                .children(
                    vol_scales
                        .map(|(max_v, half_v, _)| {
                            vec![
                                div()
                                    .text_size(axis_font_size)
                                    .text_color(text_secondary)
                                    .child(format!("{}張", max_v / 1000)),
                                div()
                                    .text_size(axis_font_size)
                                    .text_color(text_secondary)
                                    .child(format!("{}張", half_v / 1000)),
                                div()
                                    .text_size(axis_font_size)
                                    .text_color(text_secondary)
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
                .bg(toolbar_bg)
                .rounded_md()
                .border_1()
                .border_color(border_col)
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
                        )
                        .child(div().w(px(1.)).h(px(18.)).bg(divider_col))
                        .child(
                            Checkbox::new("analysis-show-bollinger")
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
                                    .text_color(text_secondary)
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
                        })
                        .child(div().w(px(1.)).h(px(18.)).bg(divider_col))
                        .child(
                            Checkbox::new("analysis-show-ma")
                                .label("均線")
                                .checked(self.show_ma)
                                .on_click(cx.listener(|this, checked, _, cx| {
                                    this.show_ma = *checked;
                                    cx.notify();
                                })),
                        )
                        .when(self.show_ma, |this| {
                            this.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .text_xs()
                                    .text_color(text_secondary)
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
                                                    .bg(hsla(0.12, 0.95, 0.55, 1.0)),
                                            )
                                            .child("MA5"),
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
                                                    .bg(hsla(0.58, 0.90, 0.55, 1.0)),
                                            )
                                            .child("MA10"),
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
                                                    .bg(hsla(0.85, 0.85, 0.60, 1.0)),
                                            )
                                            .child("MA20"),
                                    ),
                            )
                        })
                        .child(div().w(px(1.)).h(px(18.)).bg(divider_col))
                        .child(
                            Checkbox::new("analysis-show-macd")
                                .label("MACD")
                                .checked(self.show_macd)
                                .on_click(cx.listener(|this, checked, _, cx| {
                                    this.show_macd = *checked;
                                    cx.notify();
                                })),
                        )
                        .when(self.show_macd, |this| {
                            this.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .text_xs()
                                    .text_color(text_secondary)
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
                                                    .bg(hsla(0.12, 0.95, 0.55, 1.0)),
                                            )
                                            .child("DIF"),
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
                                                    .bg(hsla(0.75, 0.85, 0.60, 1.0)),
                                            )
                                            .child("MACD"),
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
                                                    .bg(hsla(0.0, 0.8, 0.5, 0.8)),
                                            )
                                            .child("OSC"),
                                    ),
                            )
                        })
                        .child(div().w(px(1.)).h(px(18.)).bg(divider_col))
                        .child(
                            Checkbox::new("analysis-show-institutional")
                                .label("三大法人")
                                .checked(self.show_institutional)
                                .on_click(cx.listener(|this, checked, _, cx| {
                                    this.show_institutional = *checked;
                                    cx.notify();
                                })),
                        )
                        .when(self.show_institutional, |this| {
                            this.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .text_xs()
                                    .text_color(text_secondary)
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .w(px(8.))
                                                    .h(px(8.))
                                                    .rounded_sm()
                                                    .bg(rgb(0xdc2626)),
                                            )
                                            .child("買超"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .w(px(8.))
                                                    .h(px(8.))
                                                    .rounded_sm()
                                                    .bg(rgb(0x16a34a)),
                                            )
                                            .child("賣超"),
                                    ),
                            )
                        }),
                )
                .child(
                    div()
                        .text_size(self.font_size.size_px())
                        .font_weight(gpui_kit::FontWeight::BOLD)
                        .text_color(text_primary)
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
                        .when(self.show_ma, |this| {
                            this.child(div().absolute().inset_0().size_full().child(ma_canvas))
                        })
                        .child(
                            div()
                                .absolute()
                                .inset_0()
                                .size_full()
                                .child(crosshair_canvas),
                        ),
                )
                .child(price_axis);

            let vol_with_axis = div()
                .size_full()
                .flex()
                .flex_row()
                .child(div().flex_1().h_full().child(vol_chart))
                .child(vol_axis);

            let active_abs_idx = self
                .hovered_candle_idx
                .unwrap_or_else(|| total_candles.saturating_sub(1));
            let is_hovered = self.hovered_candle_idx.is_some();

            let active_rec = ohlcv.get(active_abs_idx);
            let active_date = active_rec.map(|r| r.trade_date.as_str()).unwrap_or("");
            let active_open = active_rec.and_then(|r| r.open_price).unwrap_or(0.0);
            let active_high = active_rec.and_then(|r| r.high_price).unwrap_or(0.0);
            let active_low = active_rec.and_then(|r| r.low_price).unwrap_or(0.0);
            let active_close = active_rec.and_then(|r| r.close_price).unwrap_or(0.0);
            let active_vol = active_rec.and_then(|r| r.volume_shares).unwrap_or(0);

            let prev_close = if active_abs_idx > 0 {
                ohlcv.get(active_abs_idx - 1).and_then(|r| r.close_price)
            } else {
                None
            };

            let (diff_val, diff_pct) = match prev_close {
                Some(pc) if pc > 0.0 => (active_close - pc, ((active_close - pc) / pc) * 100.0),
                _ => (0.0, 0.0),
            };

            let (diff_sign, price_color) = if diff_val > 0.0001 {
                ("+", rgb(0xdc2626)) // 漲紅
            } else if diff_val < -0.0001 {
                ("", rgb(0x16a34a)) // 跌綠
            } else {
                ("", price_text)
            };

            let active_ma5 = data.ma5.get(active_abs_idx).copied().flatten();
            let active_ma10 = data.ma10.get(active_abs_idx).copied().flatten();
            let active_ma20 = data.ma20.get(active_abs_idx).copied().flatten();

            let active_bb_u = data.bollinger_upper.get(active_abs_idx).copied().flatten();
            let active_bb_m = data.bollinger_middle.get(active_abs_idx).copied().flatten();
            let active_bb_l = data.bollinger_lower.get(active_abs_idx).copied().flatten();
            let active_bb_bw = match (active_bb_u, active_bb_l, active_bb_m) {
                (Some(u), Some(l), Some(m)) if m > 0.0 => Some((u - l) / m * 100.0),
                _ => None,
            };

            let active_rsi = data.rsi.get(active_abs_idx).copied().flatten();
            let active_macd_dif = data.macd_line.get(active_abs_idx).copied().flatten();
            let active_macd_dea = data.macd_signal.get(active_abs_idx).copied().flatten();
            let active_macd_osc = data.macd_histogram.get(active_abs_idx).copied().flatten();

            let mode_badge = if is_hovered {
                div()
                    .px_1p5()
                    .py_0p5()
                    .rounded_sm()
                    .bg(hsla(0.6, 0.8, 0.5, 0.2))
                    .text_color(rgb(0x2563eb))
                    .text_xs()
                    .font_weight(gpui_kit::FontWeight::BOLD)
                    .child("游標檢視")
            } else {
                div()
                    .px_1p5()
                    .py_0p5()
                    .rounded_sm()
                    .bg(hsla(0.0, 0.0, 0.5, 0.15))
                    .text_color(text_secondary)
                    .text_xs()
                    .font_weight(gpui_kit::FontWeight::MEDIUM)
                    .child("最新收盤")
            };

            // 右側上方行情概況與技術指標即時 HUD 資訊列
            let stock_header = div()
                .flex()
                .flex_col()
                .gap_1()
                .px_3()
                .py_2()
                .bg(header_bg)
                .border_b_1()
                .border_color(border_col)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_size(self.font_size.size_px() * 1.4)
                                        .font_weight(gpui_kit::FontWeight::BOLD)
                                        .text_color(text_title)
                                        .child(data.symbol.clone()),
                                )
                                .child(
                                    div()
                                        .text_size(self.font_size.size_px() * 1.1)
                                        .font_weight(gpui_kit::FontWeight::BOLD)
                                        .text_color(text_primary)
                                        .child(data.name.clone().unwrap_or_default()),
                                )
                                .child({
                                    let is_fav = self.core_state.is_favorited(&data.symbol);
                                    Button::new("btn-toggle-favorite")
                                        .label(if is_fav {
                                            "★ 已收藏"
                                        } else {
                                            "☆ 加入自選"
                                        })
                                        .when(is_fav, |btn| btn.primary())
                                        .when(!is_fav, |btn| btn.secondary())
                                        .on_click(cx.listener({
                                            let sym = data.symbol.clone();
                                            move |this, _, _, cx| {
                                                let core = this.core_state.clone();
                                                let sym_clone = sym.clone();
                                                cx.spawn(async move |this, cx| {
                                                    let _ = core
                                                        .toggle_watchlist(&sym_clone, None)
                                                        .await;
                                                    let watchlist = core.get_watchlist_symbols();
                                                    let _ = this.update(cx, |view, cx| {
                                                        view.table_state.update(cx, |table, cx| {
                                                            let d = table.delegate_mut();
                                                            d.watchlist = watchlist;
                                                            d.apply_filter();
                                                            table.refresh(cx);
                                                        });
                                                        cx.notify();
                                                    });
                                                })
                                                .detach();
                                            }
                                        }))
                                })
                                .child(mode_badge)
                                .child(
                                    div()
                                        .text_size(self.font_size.size_px() * 0.9)
                                        .text_color(text_secondary)
                                        .child(active_date.to_string()),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_size(self.font_size.size_px() * 0.85)
                                                .text_color(text_secondary)
                                                .child("收盤:"),
                                        )
                                        .child(
                                            div()
                                                .text_size(self.font_size.size_px() * 1.15)
                                                .font_weight(gpui_kit::FontWeight::BOLD)
                                                .text_color(price_color)
                                                .child(format!(
                                                    "{:.2} ({}{:.2}, {}{:.2}%)",
                                                    active_close,
                                                    diff_sign,
                                                    diff_val,
                                                    diff_sign,
                                                    diff_pct
                                                )),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_size(self.font_size.size_px() * 0.85)
                                        .text_color(text_secondary)
                                        .child(format!(
                                            "開: {:.2}  高: {:.2}  低: {:.2}  量: {} 張",
                                            active_open,
                                            active_high,
                                            active_low,
                                            active_vol / 1000
                                        )),
                                ),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .text_xs()
                        .text_color(text_secondary)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_color(hsla(0.12, 0.95, 0.55, 1.0))
                                        .child(format!("MA5: {:.2}", active_ma5.unwrap_or(0.0))),
                                )
                                .child(
                                    div()
                                        .text_color(hsla(0.58, 0.90, 0.55, 1.0))
                                        .child(format!("MA10: {:.2}", active_ma10.unwrap_or(0.0))),
                                )
                                .child(
                                    div()
                                        .text_color(hsla(0.85, 0.85, 0.60, 1.0))
                                        .child(format!("MA20: {:.2}", active_ma20.unwrap_or(0.0))),
                                ),
                        )
                        .when(self.show_bollinger, |this| {
                            this.child(div().w(px(1.)).h(px(12.)).bg(border_col)).child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().text_color(hsla(0.80, 0.85, 0.60, 1.0)).child(
                                        format!("布林上: {:.2}", active_bb_u.unwrap_or(0.0)),
                                    ))
                                    .child(
                                        div().text_color(hsla(0.10, 0.95, 0.50, 1.0)).child(
                                            format!("中: {:.2}", active_bb_m.unwrap_or(0.0)),
                                        ),
                                    )
                                    .child(
                                        div().text_color(hsla(0.55, 0.85, 0.45, 1.0)).child(
                                            format!("下: {:.2}", active_bb_l.unwrap_or(0.0)),
                                        ),
                                    )
                                    .child(div().text_color(text_secondary).child(format!(
                                        "帶寬: {:.1}%",
                                        active_bb_bw.unwrap_or(0.0)
                                    ))),
                            )
                        })
                        .child(div().w(px(1.)).h(px(12.)).bg(border_col))
                        .child(
                            div()
                                .text_color(hsla(0.15, 0.9, 0.6, 1.0))
                                .child(format!("RSI(14): {:.1}", active_rsi.unwrap_or(0.0))),
                        )
                        .when(self.show_macd, |this| {
                            let osc_col = if active_macd_osc.unwrap_or(0.0) >= 0.0 {
                                rgb(0xdc2626)
                            } else {
                                rgb(0x16a34a)
                            };
                            this.child(div().w(px(1.)).h(px(12.)).bg(border_col)).child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().child(format!(
                                        "DIF: {:.2}",
                                        active_macd_dif.unwrap_or(0.0)
                                    )))
                                    .child(div().child(format!(
                                        "DEA: {:.2}",
                                        active_macd_dea.unwrap_or(0.0)
                                    )))
                                    .child(div().text_color(osc_col).child(format!(
                                        "OSC: {:+.2}",
                                        active_macd_osc.unwrap_or(0.0)
                                    ))),
                            )
                        })
                        .when(self.show_institutional, |this| {
                            let inst_opt =
                                data.institutional.get(active_abs_idx).cloned().flatten();
                            let format_net_shares = |val: i64| -> (String, gpui_kit::Rgba) {
                                if val > 0 {
                                    (format_shares_as_lots(val), rgb(0xdc2626))
                                } else if val < 0 {
                                    (format_shares_as_lots(val), rgb(0x16a34a))
                                } else {
                                    (format_shares_as_lots(val), text_secondary)
                                }
                            };

                            let (foreign_str, foreign_col) = inst_opt
                                .as_ref()
                                .map(|i| format_net_shares(i.foreign_net))
                                .unwrap_or_else(|| ("-".to_string(), text_secondary));
                            let (trust_str, trust_col) = inst_opt
                                .as_ref()
                                .map(|i| format_net_shares(i.trust_net))
                                .unwrap_or_else(|| ("-".to_string(), text_secondary));
                            let (dealer_str, dealer_col) = inst_opt
                                .as_ref()
                                .map(|i| format_net_shares(i.dealer_net))
                                .unwrap_or_else(|| ("-".to_string(), text_secondary));
                            let (total_str, total_col) = inst_opt
                                .as_ref()
                                .map(|i| format_net_shares(i.total_net))
                                .unwrap_or_else(|| ("-".to_string(), text_secondary));

                            this.child(div().w(px(1.)).h(px(12.)).bg(border_col)).child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div().flex().items_center().gap_1().child("外資:").child(
                                            div()
                                                .text_color(foreign_col)
                                                .font_weight(gpui_kit::FontWeight::BOLD)
                                                .child(foreign_str),
                                        ),
                                    )
                                    .child(
                                        div().flex().items_center().gap_1().child("投信:").child(
                                            div()
                                                .text_color(trust_col)
                                                .font_weight(gpui_kit::FontWeight::BOLD)
                                                .child(trust_str),
                                        ),
                                    )
                                    .child(
                                        div().flex().items_center().gap_1().child("自營:").child(
                                            div()
                                                .text_color(dealer_col)
                                                .font_weight(gpui_kit::FontWeight::BOLD)
                                                .child(dealer_str),
                                        ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .child("法人合計:")
                                            .child(
                                                div()
                                                    .text_color(total_col)
                                                    .font_weight(gpui_kit::FontWeight::BOLD)
                                                    .child(total_str),
                                            ),
                                    ),
                            )
                        }),
                );

            div()
                .size_full()
                .flex()
                .flex_col()
                .child(stock_header)
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .p_3()
                        .gap_2()
                        .child(chart_toolbar)
                        .child(
                            div()
                                .flex_1()
                                .border_1()
                                .border_color(border_col)
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
                                        this.chart_viewport.drag_start_x =
                                            Some(event.position.x.as_f32());
                                    }),
                                )
                                .on_mouse_move(cx.listener(
                                    move |this, event: &gpui_kit::MouseMoveEvent, _, cx| {
                                        if event.dragging() {
                                            if this.chart_viewport.handle_mouse_move(
                                                event.position.x.as_f32(),
                                                total_candles,
                                            ) {
                                                cx.notify();
                                            }
                                        } else if let Some(bounds) =
                                            *this.main_chart_bounds.lock().unwrap()
                                        {
                                            let px_x = event.position.x.as_f32();
                                            let px_y = event.position.y.as_f32();
                                            let b_x = bounds.origin.x.as_f32();
                                            let b_y = bounds.origin.y.as_f32();
                                            let w = bounds.size.width.as_f32();
                                            let h = bounds.size.height.as_f32();

                                            let rel_x = px_x - b_x;
                                            let rel_y = px_y - b_y;

                                            if rel_x >= 0.0
                                                && rel_x <= w
                                                && rel_y >= 0.0
                                                && rel_y <= h
                                            {
                                                this.hover_pos = Some(event.position);
                                                if let Some((_col, abs_idx)) = this
                                                    .chart_viewport
                                                    .candle_index_at_x(rel_x, w, total_candles)
                                                {
                                                    this.hovered_candle_idx = Some(abs_idx);
                                                    cx.notify();
                                                }
                                            } else if this.hovered_candle_idx.is_some()
                                                || this.hover_pos.is_some()
                                            {
                                                this.hovered_candle_idx = None;
                                                this.hover_pos = None;
                                                cx.notify();
                                            }
                                        }
                                    },
                                ))
                                .on_mouse_exit(cx.listener(|this, _, _, cx| {
                                    let had_hover_idx = this.hovered_candle_idx.take().is_some();
                                    let had_hover_pos = this.hover_pos.take().is_some();
                                    let had_hover = had_hover_idx || had_hover_pos;
                                    if had_hover {
                                        cx.notify();
                                    }
                                }))
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
                                        .border_color(border_col)
                                        .rounded_md()
                                        .p_2()
                                        .child(vol_with_axis),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .border_1()
                                        .border_color(border_col)
                                        .rounded_md()
                                        .p_2()
                                        .child(rsi_chart),
                                )
                                .when(self.show_macd, |this| {
                                    this.child(
                                        div()
                                            .flex_1()
                                            .border_1()
                                            .border_color(border_col)
                                            .rounded_md()
                                            .p_2()
                                            .child(macd_canvas),
                                    )
                                })
                                .when(self.show_institutional, |this| {
                                    this.child(
                                        div()
                                            .flex_1()
                                            .border_1()
                                            .border_color(border_col)
                                            .rounded_md()
                                            .p_2()
                                            .child(
                                                div()
                                                    .size_full()
                                                    .relative()
                                                    .child(
                                                        div()
                                                            .absolute()
                                                            .top_0()
                                                            .left_0()
                                                            .text_xs()
                                                            .text_color(text_secondary)
                                                            .child("三大法人 (張)"),
                                                    )
                                                    .child(institutional_canvas),
                                            ),
                                    )
                                }),
                        ),
                )
        } else {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(text_secondary)
                .child("請點擊左側證券清單以檢視 K 線圖與走勢分析")
        };

        // 左右分欄拖曳佈局
        h_resizable("analysis-resizable-layout")
            .child(
                resizable_panel()
                    .size(px(260.))
                    .size_range(px(200.)..px(500.))
                    .child(left_panel),
            )
            .child(resizable_panel().child(div().size_full().child(right_content)))
    }
}

#[cfg(test)]
mod tests {
    use super::format_shares_as_lots;

    #[test]
    fn institutional_lots_preserve_sub_lot_values_and_direction() {
        assert_eq!(format_shares_as_lots(999), "+0.999張");
        assert_eq!(format_shares_as_lots(-999), "-0.999張");
        assert_eq!(format_shares_as_lots(0), "0張");
    }
}
