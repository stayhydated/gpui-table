use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct InvalidNumberRangeSuffix {
    #[gpui_table(filter(number_range(step = 1f32)))]
    age: u8,
}

fn main() {}
