use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct InvalidNumberRangeLiteral {
    #[gpui_table(filter(number_range(step = "oops")))]
    age: u8,
}

fn main() {}
