use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct InvalidNumberRangeMinMax {
    #[gpui_table(filter(number_range(min = 20.0, max = 10.0)))]
    age: u8,
}

fn main() {}
