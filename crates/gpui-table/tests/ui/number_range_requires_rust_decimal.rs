use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct NumberRangeWithoutRustDecimalFeature {
    #[gpui_table(filter(gpui_table_component::NumberRangeFilter))]
    age: u8,
}

fn main() {}
