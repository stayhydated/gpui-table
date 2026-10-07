use gpui_table::GpuiTable;
#[derive(Clone, GpuiTable)]
#[gpui_table(mcp)]
struct Row {
    #[gpui_table(filter(gpui_table_component::TextFilter))]
    sort: String,
}
fn main() {}
