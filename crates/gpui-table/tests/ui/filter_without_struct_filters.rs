use gpui_table::GpuiTable;

#[derive(GpuiTable)]
struct MissingStructFilters {
    #[gpui_table(filter(text()))]
    name: String,
}

fn main() {}
