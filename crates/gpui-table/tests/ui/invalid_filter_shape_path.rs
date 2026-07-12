use gpui_table::GpuiTable;

#[derive(Clone, GpuiTable)]
#[gpui_table(filters)]
struct InvalidFilterShapePath {
    #[gpui_table(filter(gpui_table_component::MissingFilter))]
    name: String,
}

fn main() {}
