use gpui_table::GpuiTable;

#[derive(GpuiTable)]
struct MissingStructFilters {
    #[gpui_table(filter(gpui_table_example_component::TextFilter))]
    name: String,
}

fn main() {}
