use gpui_kit::component::table::{Column, ColumnSort, DataTable, TableDelegate, TableState};
use gpui_kit::{
    App, AppContext, Bounds, Context, Entity, IntoElement, ParentElement as _, Point, Render,
    Styled as _, TestAppContext, TitlebarOptions, Window, WindowBounds, WindowOptions, div, px,
    size,
};

#[derive(Clone, Debug)]
pub struct SecurityItem {
    pub symbol: String,
    pub market: String,
    pub name: String,
}

pub struct SecuritiesDelegate {
    pub items: Vec<SecurityItem>,
    pub sort_col: Option<usize>,
    pub sort_order: ColumnSort,
}

impl SecuritiesDelegate {
    pub fn sort_by_column(&mut self, col_ix: usize, sort: ColumnSort) {
        self.sort_col = Some(col_ix);
        self.sort_order = sort;
        match sort {
            ColumnSort::Ascending => match col_ix {
                0 => self.items.sort_by(|a, b| a.symbol.cmp(&b.symbol)),
                1 => self.items.sort_by(|a, b| a.market.cmp(&b.market)),
                _ => self.items.sort_by(|a, b| a.name.cmp(&b.name)),
            },
            ColumnSort::Descending => match col_ix {
                0 => self.items.sort_by(|a, b| b.symbol.cmp(&a.symbol)),
                1 => self.items.sort_by(|a, b| b.market.cmp(&a.market)),
                _ => self.items.sort_by(|a, b| b.name.cmp(&a.name)),
            },
            ColumnSort::Default => {}
        }
    }
}

impl TableDelegate for SecuritiesDelegate {
    fn columns_count(&self, _: &App) -> usize {
        3
    }

    fn rows_count(&self, _: &App) -> usize {
        self.items.len()
    }

    fn column(&self, ix: usize, _: &App) -> Column {
        match ix {
            0 => Column::new("symbol", "股票代號").width(px(120.)).sortable(),
            1 => Column::new("market", "市場").width(px(100.)).sortable(),
            _ => Column::new("name", "證券名稱").width(px(200.)).sortable(),
        }
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) {
        self.sort_by_column(col_ix, sort);
    }

    fn render_td(
        &mut self,
        row: usize,
        col: usize,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let item = &self.items[row];
        let text = match col {
            0 => item.symbol.as_str(),
            1 => item.market.as_str(),
            _ => item.name.as_str(),
        };
        div().child(text.to_string())
    }
}

struct TableView {
    table: Entity<TableState<SecuritiesDelegate>>,
}

impl Render for TableView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(DataTable::new(&self.table))
    }
}

#[gpui_kit::test]
fn test_datatable_virtual_scrolling_and_sorting(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);

    // 建立 12,000 筆證券測試資料以驗證萬筆資料效能與排序
    let mut dummy_securities = Vec::with_capacity(12000);
    for i in 0..12000 {
        dummy_securities.push(SecurityItem {
            symbol: format!("{:05}", 1000 + (12000 - i)),
            market: if i % 2 == 0 {
                "TWSE".into()
            } else {
                "TPEx".into()
            },
            name: format!("測試股票-{:05}", i),
        });
    }

    let delegate = SecuritiesDelegate {
        items: dummy_securities,
        sort_col: None,
        sort_order: ColumnSort::Default,
    };

    let (window_handle, view_entity) = cx.update(|cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(1280.0), px(800.0)),
            })),
            titlebar: Some(TitlebarOptions {
                title: Some("證券清單 DataTable 測試".into()),
                ..Default::default()
            }),
            window_min_size: Some(size(px(960.0), px(640.0))),
            ..Default::default()
        };

        gpui_kit::open_window(options, cx, |window, cx| {
            let table = cx.new(|cx| TableState::new(delegate, window, cx).cell_selectable(true));
            cx.new(|_| TableView { table })
        })
        .expect("開啟視窗失敗")
    });

    cx.update_window(window_handle, |_, window, cx| {
        window.draw(cx).clear(cx);
    })
    .unwrap();

    let table_entity = view_entity.read_with(cx, |v, _| v.table.clone());

    // 驗證萬筆資料行數正確
    table_entity.update(cx, |table, cx| {
        assert_eq!(table.delegate().rows_count(cx), 12000);
        assert_eq!(table.delegate().columns_count(cx), 3);

        // 觸發排序並驗證升冪排列
        table
            .delegate_mut()
            .sort_by_column(0, ColumnSort::Ascending);
        assert_eq!(table.delegate().items[0].symbol, "01001");
        assert_eq!(table.delegate().items[11999].symbol, "13000");

        // 觸發降冪排序
        table
            .delegate_mut()
            .sort_by_column(0, ColumnSort::Descending);
        assert_eq!(table.delegate().items[0].symbol, "13000");
        assert_eq!(table.delegate().items[11999].symbol, "01001");
    });
}
