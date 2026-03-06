use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct NumberRangeWithoutRustDecimalFeature {
    #[gpui_table(filter(number_range(min = 0.0, max = 10.0)))]
    age: u8,
}

fn main() {}
