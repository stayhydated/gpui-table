use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct InvalidTextFilterType {
    #[gpui_table(filter(text()))]
    active: bool,
}

fn main() {}
