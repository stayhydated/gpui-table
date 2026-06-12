use gpui_table::GpuiTable;

#[derive(Clone, GpuiTable)]
#[gpui_table(mcp)]
struct UserRow {
    name: String,
}

fn main() {}
