use gpui_kit::base::Disableable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::table::{Column, ColumnSort, DataTable, TableDelegate, TableState};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _, Window,
    div, px, rgb,
};
use stock_core::models::ScreenResult;

use crate::state::CoreState;

pub struct ScreenerDelegate {
    pub results: Vec<ScreenResult>,
    pub sort_col: Option<usize>,
    pub sort_order: ColumnSort,
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
            ColumnSort::Default => {}
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
        match ix {
            0 => Column::new("symbol", "股票代號").width(px(100.)).sortable(),
            1 => Column::new("name", "證券名稱").width(px(120.)).sortable(),
            2 => Column::new("close", "收盤價").width(px(100.)).sortable(),
            3 => Column::new("volume", "成交量").width(px(120.)).sortable(),
            4 => Column::new("max_bw", "最大頻寬").width(px(100.)).sortable(),
            _ => Column::new("latest_bw", "最新頻寬")
                .width(px(100.))
                .sortable(),
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
        let r = &self.results[row];
        let text = match col {
            0 => r.symbol.clone(),
            1 => r.name.clone().unwrap_or_default(),
            2 => format!("{:.2}", r.latest_close.unwrap_or(0.0)),
            3 => format!("{}", r.latest_volume.unwrap_or(0)),
            4 => format!("{:.4}", r.max_bandwidth),
            _ => format!("{:.4}", r.latest_bandwidth),
        };
        div().child(text)
    }
}

pub struct ScreenerView {
    pub core_state: CoreState,
    pub start_date_input: Entity<InputState>,
    pub end_date_input: Entity<InputState>,
    pub bandwidth_input: Entity<InputState>,
    pub min_days_input: Entity<InputState>,
    pub table_state: Entity<TableState<ScreenerDelegate>>,
    pub is_running: bool,
    pub status_text: String,
}

impl ScreenerView {
    pub fn new(core_state: CoreState, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let start_date_input = cx.new(|cx| InputState::new(window, cx).default_value("2026-03-01"));
        let end_date_input = cx.new(|cx| InputState::new(window, cx).default_value("2026-03-20"));
        let bandwidth_input = cx.new(|cx| InputState::new(window, cx).default_value("0.10"));
        let min_days_input = cx.new(|cx| InputState::new(window, cx).default_value("3"));

        let delegate = ScreenerDelegate {
            results: Vec::new(),
            sort_col: None,
            sort_order: ColumnSort::Default,
        };
        let table_state = cx.new(|cx| TableState::new(delegate, window, cx).cell_selectable(true));

        Self {
            core_state,
            start_date_input,
            end_date_input,
            bandwidth_input,
            min_days_input,
            table_state,
            is_running: false,
            status_text: "就緒".into(),
        }
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
                    None,
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
                        if let Some((curr, tot, date)) = msg {
                            let _ = this.update(cx, |view, cx| {
                                view.status_text = format!("正在篩選 {}（進度: {}/{}）", date, curr, tot);
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
                                    view.table_state.update(cx, |table, _| {
                                        table.delegate_mut().results = data.results;
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

        let table = div()
            .flex_1()
            .size_full()
            .p_3()
            .child(DataTable::new(&self.table_state));

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(toolbar)
            .child(table)
    }
}
