use gpui_table::GpuiTable;
#[derive(Clone, GpuiTable)]
#[gpui_table(row_id = "missing")]
struct Row { score: i64 }
fn main() {}
