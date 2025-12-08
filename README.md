# GPUI Table

[![Build Status](https://github.com/stayhydated/gpui-table/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/gpui-table/actions/workflows/ci.yml)
[![Docs](https://docs.rs/gpui-table/badge.svg)](https://docs.rs/gpui-table/)
[![Crates.io](https://img.shields.io/crates/v/gpui-table.svg)](https://crates.io/crates/gpui-table)

`gpui-table` ...tbd

## Compatibility

| `gpui-table` | `gpui-component` |
| :----------- | :--------------- |
| `0.5.x`      | `0.5.x`          |

## Showcase

declaring:

```rs
#[derive(Clone, Debug, Dummy, EsFluentKv, GpuiTable)]
#[fluent_kv(this, keys = ["description", "label"])]
#[gpui_table(load_more = "Self::load_more_data")]
#[gpui_table(load_more_threshold = 30)]
#[gpui_table(fluent = "label")]
pub struct InfiniteRow {
    #[dummy(faker = "1..10000")]
    #[gpui_table(width = 80., resizable = false, movable = false)]
    pub id: u64,

    #[dummy(faker = "Name()")]
    #[gpui_table(sortable, ascending)]
    pub name: String,

    #[dummy(faker = "Sentence(3..6)")]
    #[gpui_table(width = 300.)]
    pub description: String,
}

impl InfiniteRowTableDelegate {
    pub fn load_more_data(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        if self.loading || self.eof {
            return;
        }

        self.loading = true;
        cx.notify();

        cx.spawn(async move |view, cx| {
            // Simulate network delay
            cx.background_executor()
                .timer(Duration::from_millis(500))
                .await;

            let new_rows: Vec<InfiniteRow> = (0..50).map(|_| Faker.fake()).collect();

            _ = cx.update(|cx| {
                view.update(cx, |table, cx| {
                    let delegate = table.delegate_mut();
                    delegate.rows.extend(new_rows);
                    delegate.loading = false;

                    // Stop after 500 rows
                    if delegate.rows.len() >= 500 {
                        delegate.eof = true;
                    }

                    cx.notify();
                })
                .unwrap();
            });
        })
        .detach();
    }
}
```

this would expand to a structure we normally would have to declare ourselves, reducing boilerplate

```rs
pub enum InfiniteRowTableColumn {
    Id,
    Name,
    Description,
}
impl From<usize> for InfiniteRowTableColumn {
    fn from(ix: usize) -> Self {
        match ix {
            0usize => InfiniteRowTableColumn::Id,
            1usize => InfiniteRowTableColumn::Name,
            2usize => InfiniteRowTableColumn::Description,
            _ => {
                ::core::panicking::panic_fmt(format_args!("Invalid column index: {0}", ix));
            },
        }
    }
}
impl From<InfiniteRowTableColumn> for usize {
    fn from(col: InfiniteRowTableColumn) -> Self {
        match col {
            InfiniteRowTableColumn::Id => 0usize,
            InfiniteRowTableColumn::Name => 1usize,
            InfiniteRowTableColumn::Description => 2usize,
        }
    }
}
impl gpui_table::TableRowMeta for InfiniteRow {
    const TABLE_ID: &'static str = "InfiniteRow";
    const TABLE_TITLE: &'static str = "InfiniteRow";
    fn table_title() -> String {
        InfiniteRowLabelKvFtl::this_ftl()
    }
    fn table_columns() -> &'static [gpui_component::table::Column] {
        static COLUMNS: std::sync::OnceLock<Vec<gpui_component::table::Column>> =
            std::sync::OnceLock::new();
        COLUMNS.get_or_init(|| {
            <[_]>::into_vec(::alloc::boxed::box_new([
                gpui_component::table::Column::new("id", {
                    use es_fluent::ToFluentString as _;
                    InfiniteRowLabelKvFtl::Id.to_fluent_string()
                })
                .width(80f32)
                .resizable(false)
                .movable(false),
                gpui_component::table::Column::new("name", {
                    use es_fluent::ToFluentString as _;
                    InfiniteRowLabelKvFtl::Name.to_fluent_string()
                })
                .width(100f32)
                .ascending(),
                gpui_component::table::Column::new("description", {
                    use es_fluent::ToFluentString as _;
                    InfiniteRowLabelKvFtl::Description.to_fluent_string()
                })
                .width(300f32),
            ]))
        })
    }
    fn cell_value(&self, col_ix: usize) -> Box<dyn gpui_table::TableCell + '_> {
        match col_ix {
            0usize => Box::new(self.id.clone()),
            1usize => Box::new(self.name.clone()),
            2usize => Box::new(self.description.clone()),
            _ => Box::new(String::new()),
        }
    }
}
impl gpui_table::TableRowStyle for InfiniteRow {
    type ColumnId = InfiniteRowTableColumn;
    fn render_table_cell(
        &self,
        col: Self::ColumnId,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> gpui::AnyElement {
        use gpui::IntoElement;
        gpui_table::default_render_cell(self, col.into(), window, cx).into_any_element()
    }
}
pub struct InfiniteRowTableDelegate {
    pub rows: Vec<InfiniteRow>,
    #[new(default)]
    pub visible_rows: std::ops::Range<usize>,
    #[new(default)]
    pub visible_cols: std::ops::Range<usize>,
    #[new(default)]
    pub eof: bool,
    #[new(default)]
    pub loading: bool,
    #[new(default)]
    pub full_loading: bool,
}
impl InfiniteRowTableDelegate {
    ///Constructs a new `InfiniteRowTableDelegate`.
    pub fn new(rows: Vec<InfiniteRow>) -> Self {
        InfiniteRowTableDelegate {
            rows,
            visible_rows: ::core::default::Default::default(),
            visible_cols: ::core::default::Default::default(),
            eof: ::core::default::Default::default(),
            loading: ::core::default::Default::default(),
            full_loading: ::core::default::Default::default(),
        }
    }
}
impl gpui_component::table::TableDelegate for InfiniteRowTableDelegate {
    fn columns_count(&self, _: &gpui::App) -> usize {
        <InfiniteRow as gpui_table::TableRowMeta>::table_columns().len()
    }
    fn rows_count(&self, _: &gpui::App) -> usize {
        self.rows.len()
    }
    fn column(&self, col_ix: usize, _: &gpui::App) -> &gpui_component::table::Column {
        &<InfiniteRow as gpui_table::TableRowMeta>::table_columns()[col_ix]
    }
    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
    ) -> impl gpui::IntoElement {
        use gpui_table::TableRowStyle;
        self.rows[row_ix].render_table_cell(InfiniteRowTableColumn::from(col_ix), window, cx)
    }
    fn visible_rows_changed(
        &mut self,
        visible_range: std::ops::Range<usize>,
        _: &mut gpui::Window,
        _: &mut gpui::Context<gpui_component::table::TableState<Self>>,
    ) {
        self.visible_rows = visible_range;
    }
    fn visible_columns_changed(
        &mut self,
        visible_range: std::ops::Range<usize>,
        _: &mut gpui::Window,
        _: &mut gpui::Context<gpui_component::table::TableState<Self>>,
    ) {
        self.visible_cols = visible_range;
    }
    fn loading(&self, _: &gpui::App) -> bool {
        self.full_loading
    }
    fn is_eof(&self, _: &gpui::App) -> bool {
        if self.loading {
            return false;
        }
        !self.eof
    }
    fn load_more(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
    ) {
        Self::load_more_data(self, window, cx);
    }
    fn load_more_threshold(&self) -> usize {
        30usize
    }
    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: gpui_component::table::ColumnSort,
        _: &mut gpui::Window,
        _: &mut gpui::Context<gpui_component::table::TableState<Self>>,
    ) {
        match col_ix {
            1usize => {
                self.rows.sort_by(|a, b| {
                    let a_val = &a.name;
                    let b_val = &b.name;
                    match sort {
                        gpui_component::table::ColumnSort::Ascending => a_val
                            .partial_cmp(b_val)
                            .unwrap_or(std::cmp::Ordering::Equal),
                        gpui_component::table::ColumnSort::Descending => b_val
                            .partial_cmp(a_val)
                            .unwrap_or(std::cmp::Ordering::Equal),
                        _ => std::cmp::Ordering::Equal,
                    }
                });
            },
            _ => {},
        }
    }
}
```

## Bonus

There's also a prototyping tool which you can customize to your needs (except the [gpui-table-prototyping-core](crates/gpui-table-prototyping-core))

see examples's [README.md](examples/README.md)
