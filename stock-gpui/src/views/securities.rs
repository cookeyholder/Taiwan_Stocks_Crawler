use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::table::{Column, ColumnSort, DataTable, TableDelegate, TableState};
use gpui_kit::component::Sizable as _;
use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _, Window,
    div, px, rgb,
};

use crate::app::FontSizeLevel;
use crate::state::{CoreState, SecurityItem};

pub struct SecuritiesListDelegate {
    pub all_items: Vec<SecurityItem>,
    pub filtered_items: Vec<SecurityItem>,
    pub sort_col: Option<usize>,
    pub sort_order: ColumnSort,
    pub font_size: FontSizeLevel,
}

impl SecuritiesListDelegate {
    pub fn filter(&mut self, query: &str) {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            self.filtered_items = self.all_items.clone();
        } else {
            self.filtered_items = self
                .all_items
                .iter()
                .filter(|s| {
                    s.symbol.to_lowercase().contains(&q)
                        || s.name.to_lowercase().contains(&q)
                        || s.market.to_lowercase().contains(&q)
                })
                .cloned()
                .collect();
        }
        if let Some(col) = self.sort_col {
            self.sort_by_col(col, self.sort_order);
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
            ColumnSort::Default => {}
        }
    }
}

impl TableDelegate for SecuritiesListDelegate {
    fn columns_count(&self, _: &App) -> usize {
        3
    }

    fn rows_count(&self, _: &App) -> usize {
        self.filtered_items.len()
    }

    fn column(&self, ix: usize, _: &App) -> Column {
        match ix {
            0 => Column::new("symbol", "股票代號").width(px(120.)).sortable(),
            1 => Column::new("market", "市場").width(px(100.)).sortable(),
            _ => Column::new("name", "證券名稱").width(px(220.)).sortable(),
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
        let item = &self.filtered_items[row];
        let text = match col {
            0 => item.symbol.as_str(),
            1 => item.market.as_str(),
            _ => item.name.as_str(),
        };
        div()
            .text_size(self.font_size.size_px())
            .child(text.to_string())
    }
}

pub struct SecuritiesView {
    pub core_state: CoreState,
    pub search_input: Entity<InputState>,
    pub table_state: Entity<TableState<SecuritiesListDelegate>>,
    pub is_loaded: bool,
    pub font_size: FontSizeLevel,
}

impl SecuritiesView {
    pub fn new(core_state: CoreState, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("搜尋股票代號或名稱..."));
        let delegate = SecuritiesListDelegate {
            all_items: Vec::new(),
            filtered_items: Vec::new(),
            sort_col: None,
            sort_order: ColumnSort::Default,
            font_size: FontSizeLevel::Medium,
        };
        let table_state = cx.new(|cx| TableState::new(delegate, window, cx).cell_selectable(true));

        cx.subscribe(
            &search_input,
            |this: &mut Self, input, event: &gpui_kit::component::input::InputEvent, cx| {
                if matches!(
                    event,
                    gpui_kit::component::input::InputEvent::Change
                        | gpui_kit::component::input::InputEvent::PressEnter { .. }
                ) {
                    let q = input.read(cx).value().to_string();
                    this.apply_filter(&q, cx);
                }
            },
        )
        .detach();

        let mut view = Self {
            core_state,
            search_input,
            table_state,
            is_loaded: false,
            font_size: FontSizeLevel::Medium,
        };
        view.load_securities(cx);
        view
    }

    pub fn set_font_size(&mut self, font_size: FontSizeLevel, cx: &mut Context<Self>) {
        self.font_size = font_size;
        self.table_state.update(cx, |table, cx| {
            table.delegate_mut().font_size = font_size;
            table.refresh(cx);
        });
        cx.notify();
    }

    pub fn apply_filter(&mut self, query: &str, cx: &mut Context<Self>) {
        self.table_state.update(cx, |table, _| {
            table.delegate_mut().filter(query);
        });
        cx.notify();
    }

    pub fn load_securities(&mut self, cx: &mut Context<Self>) {
        let core = self.core_state.clone();
        cx.spawn(async move |this, cx| {
            let res = core.get_securities().await;
            let _ = this.update(cx, |view, cx| {
                if let Ok(items) = res {
                    view.is_loaded = true;
                    view.table_state.update(cx, |table, _| {
                        let d = table.delegate_mut();
                        d.all_items = items.clone();
                        d.filtered_items = items;
                    });
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for SecuritiesView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.table_state.read(cx).delegate().filtered_items.len();

        let toolbar = div()
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
                    .child(
                        div()
                            .text_size(self.font_size.size_px())
                            .text_color(rgb(0xa6adc8))
                            .child("搜尋標的:"),
                    )
                    .child(
                        div()
                            .w(px(240.))
                            .child(Input::new(&self.search_input).id("securities-search-input")),
                    ),
            )
            .child(
                Button::new("btn-search-securities")
                    .label("篩選")
                    .primary()
                    .on_click(cx.listener(|this, _, _, cx| {
                        let q = this.search_input.read(cx).value().to_string();
                        this.apply_filter(&q, cx);
                    })),
            )
            .child(
                div()
                    .text_color(rgb(0xa6adc8))
                    .text_size(self.font_size.size_px())
                    .child(format!("共 {} 檔證券標的", count)),
            );

        let table = div()
            .flex_1()
            .size_full()
            .p_3()
            .child(
                DataTable::new(&self.table_state)
                    .with_size(self.font_size.table_size()),
            );

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(toolbar)
            .child(table)
    }
}
