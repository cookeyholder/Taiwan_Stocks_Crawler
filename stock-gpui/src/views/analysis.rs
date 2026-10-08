use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::chart::{BarChart, CandlestickChart, LineChart};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::resizable::{h_resizable, resizable_panel};
use gpui_kit::component::table::{
    Column, ColumnSort, DataTable, TableDelegate, TableEvent, TableState,
};
use gpui_kit::{
    App, AppContext, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    Render, Styled as _, Window, div, hsla, px, rgb,
};

use crate::state::{AnalysisData, CoreState, SecurityItem};
use crate::views::chart_viewport::{ChartViewport, compute_price_scales, compute_volume_scales};

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
    pub search_query: String,
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
            search_query: String::new(),
        }
    }

    pub fn apply_filter(&mut self) {
        let q = self.search_query.trim().to_lowercase();
        self.filtered_items = self
            .all_items
            .iter()
            .filter(|s| {
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
        let text = match col {
            0 => item.symbol.as_str(),
            1 => item.market.as_str(),
            _ => item.name.as_str(),
        };
        div().cursor_pointer().child(text.to_string())
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
        };

        view.load_securities(cx);
        view.fetch_symbol("2330".into(), cx);
        view
    }

    pub fn load_securities(&mut self, cx: &mut Context<Self>) {
        let core = self.core_state.clone();
        cx.spawn(async move |this, cx| {
            let res = core.get_securities().await;
            let _ = this.update(cx, |view, cx| {
                if let Ok(items) = res {
                    view.table_state.update(cx, |table, cx| {
                        let d = table.delegate_mut();
                        d.all_items = items;
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

        // 左側工具列
        let left_toolbar = div()
            .flex()
            .flex_col()
            .gap_2()
            .p_2()
            .border_b_1()
            .border_color(rgb(0xe2e8f0))
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
                            .text_color(rgb(0x64748b))
                            .child(format!("共 {} 檔", count)),
                    ),
            );

        // 左側證券清單面板
        let left_panel = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0xf8fafc))
            .border_r_1()
            .border_color(rgb(0xe2e8f0))
            .child(left_toolbar)
            .child(
                div()
                    .flex_1()
                    .size_full()
                    .p_1()
                    .child(DataTable::new(&self.table_state)),
            );

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

            let latest_rec = visible_ohlcv.last();
            let latest_close = latest_rec.and_then(|r| r.close_price).unwrap_or(0.0);
            let latest_vol = latest_rec.and_then(|r| r.volume_shares).unwrap_or(0);

            // 右側上方行情概況列
            let stock_header = div()
                .flex()
                .items_center()
                .justify_between()
                .px_3()
                .py_2()
                .bg(rgb(0xf8fafc))
                .border_b_1()
                .border_color(rgb(0xe2e8f0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .text_xl()
                                .font_weight(gpui_kit::FontWeight::BOLD)
                                .text_color(rgb(0x1d4ed8))
                                .child(data.symbol.clone()),
                        )
                        .child(
                            div()
                                .text_base()
                                .font_weight(gpui_kit::FontWeight::BOLD)
                                .text_color(rgb(0x1e293b))
                                .child(data.name.clone().unwrap_or_default()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(div().text_xs().text_color(rgb(0x64748b)).child("收盤價:"))
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(gpui_kit::FontWeight::BOLD)
                                        .text_color(rgb(0x0f172a))
                                        .child(format!("{:.2}", latest_close)),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(div().text_xs().text_color(rgb(0x64748b)).child("成交量:"))
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(gpui_kit::FontWeight::BOLD)
                                        .text_color(rgb(0x0f172a))
                                        .child(format!("{} 張", latest_vol / 1000)),
                                ),
                        ),
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
                                .border_color(rgb(0xe2e8f0))
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
                                        if event.dragging()
                                            && this.chart_viewport.handle_mouse_move(
                                                event.position.x.as_f32(),
                                                total_candles,
                                            )
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
                                        .border_color(rgb(0xe2e8f0))
                                        .rounded_md()
                                        .p_2()
                                        .child(vol_with_axis),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .border_1()
                                        .border_color(rgb(0xe2e8f0))
                                        .rounded_md()
                                        .p_2()
                                        .child(rsi_chart),
                                ),
                        ),
                )
        } else {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(0x64748b))
                .child("請點擊左側證券清單以檢視 K 線圖與走勢分析")
        };

        // 左右分欄拖曳佈局
        h_resizable("analysis-resizable-layout")
            .child(
                resizable_panel()
                    .size(px(320.))
                    .size_range(px(240.)..px(600.))
                    .child(left_panel),
            )
            .child(resizable_panel().child(div().size_full().child(right_content)))
    }
}
