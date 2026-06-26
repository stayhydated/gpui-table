use gpui::{ParentElement as _, Styled as _};
use gpui_table::{GpuiTable, TableRowMeta};

#[derive(GpuiTable)]
struct StyledCellRow {
    #[gpui_table(width = 80., style = render_score_cell)]
    score: u8,
    name: String,
}

fn render_score_cell(
    row: &StyledCellRow,
    value: &u8,
    _window: &mut gpui::Window,
    _cx: &mut gpui::App,
) -> impl gpui::IntoElement {
    let _ = row;
    gpui::div().child(format!("{value} points")).px_1()
}

fn assert_table_row_style<T: gpui_table::runtime::TableRowStyle>() {}

fn main() {
    assert_table_row_style::<StyledCellRow>();

    let _delegate = StyledCellRowTableDelegate::new(Vec::new());
    let _score_column = StyledCellRowTableColumn::Score;
    let _name_column = StyledCellRowTableColumn::Name;

    assert_eq!(StyledCellRow::table_columns().len(), 2);
}
