use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct InvalidNumberRangeType {
    #[gpui_table(filter(gpui_table_component::NumberRangeFilter))]
    name: String,
}

fn main() {}
