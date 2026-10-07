use gpui_table::GpuiTable;
#[derive(Clone, GpuiTable)]
struct Row {
    #[gpui_table(sortable, col = "score")]
    first: i64,
    #[gpui_table(sortable, col = "score")]
    second: i64,
}
fn main() {}
