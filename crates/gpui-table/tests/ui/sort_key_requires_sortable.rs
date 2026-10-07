use gpui_table::GpuiTable;
#[derive(Clone, GpuiTable)]
struct Row {
    #[gpui_table(sort_key = key)]
    score: i64,
}
fn key(row: &Row) -> Option<i64> { Some(row.score) }
fn main() {}
