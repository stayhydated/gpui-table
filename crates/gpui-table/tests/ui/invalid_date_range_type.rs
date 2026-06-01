use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct InvalidDateRangeType {
    #[gpui_table(filter(gpui_table_component::DateRangeFilter))]
    name: String,
}

fn main() {}
