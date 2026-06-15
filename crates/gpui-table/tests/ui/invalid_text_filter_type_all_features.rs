use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct InvalidTextFilterType {
    #[gpui_table(filter(gpui_table_component::TextFilter))]
    active: bool,
}

fn main() {}
