use gpui_table::GpuiTable;
use serde::Serialize;

#[derive(Clone, GpuiTable, Serialize)]
#[gpui_table(mcp(row_schema))]
struct Row {
    name: String,
}

fn main() {}
