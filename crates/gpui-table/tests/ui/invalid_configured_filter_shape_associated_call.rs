use gpui_table::GpuiTable;

#[derive(Clone, GpuiTable)]
#[gpui_table(filters)]
struct User {
    #[gpui_table(filter(gpui_table_component::TextFilter::numeric_only()))]
    code: String,
}

fn main() {}
