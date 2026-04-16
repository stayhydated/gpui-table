use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct NumberRangeStringLiteral {
    #[gpui_table(filter(number_range(min = "-5.25", max = "10.75", step = "0.25")))]
    age: i32,
}

fn main() {}
