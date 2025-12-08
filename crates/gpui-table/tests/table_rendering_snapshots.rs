use gpui::TextAlign;
use gpui_component::table::{Column, ColumnFixed, ColumnSort};
use gpui_table::{GpuiTable, TableRowMeta};
use serde::Serialize;

#[derive(GpuiTable)]
struct BasicRow {
    name: String,
    age: u8,
    active: bool,
}

#[derive(GpuiTable)]
#[gpui_table(id = "custom-row", title = "Custom Row Table")]
struct StyledRow {
    #[gpui_table(width = 120., sortable)]
    name: String,

    #[gpui_table(width = 80., text_right, descending, fixed = "left", resizable = false)]
    score: u8,

    #[gpui_table(width = 180., ascending, title = "Email Address", movable = false)]
    email: String,

    #[gpui_table(skip)]
    #[allow(dead_code)]
    internal: String,
}

#[derive(Serialize)]
struct ColumnSnapshot {
    key: String,
    title: String,
    align: &'static str,
    sort: Option<&'static str>,
    width: f32,
    fixed: Option<&'static str>,
    resizable: bool,
    movable: bool,
    selectable: bool,
}

#[derive(Serialize)]
struct TableSnapshot {
    table_id: &'static str,
    title: String,
    columns: Vec<ColumnSnapshot>,
}

fn to_column_snapshot(column: &Column) -> ColumnSnapshot {
    let width: f32 = (&column.width).into();

    ColumnSnapshot {
        key: column.key.to_string(),
        title: column.name.to_string(),
        align: match column.align {
            TextAlign::Left => "left",
            TextAlign::Center => "center",
            TextAlign::Right => "right",
        },
        sort: column.sort.as_ref().map(|sort| match sort {
            ColumnSort::Default => "default",
            ColumnSort::Ascending => "ascending",
            ColumnSort::Descending => "descending",
        }),
        width,
        fixed: column.fixed.map(|fixed| match fixed {
            ColumnFixed::Left => "left",
        }),
        resizable: column.resizable,
        movable: column.movable,
        selectable: column.selectable,
    }
}

fn table_snapshot<T: TableRowMeta>() -> TableSnapshot {
    TableSnapshot {
        table_id: T::TABLE_ID,
        title: T::table_title(),
        columns: T::table_columns().iter().map(to_column_snapshot).collect(),
    }
}

#[test]
fn basic_table_rendering_snapshot() {
    insta::assert_yaml_snapshot!("basic_table_rendering", table_snapshot::<BasicRow>());
}

#[test]
fn styled_table_rendering_snapshot() {
    insta::assert_yaml_snapshot!("styled_table_rendering", table_snapshot::<StyledRow>());
}
