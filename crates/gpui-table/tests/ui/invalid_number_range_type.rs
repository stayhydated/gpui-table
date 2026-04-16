use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct InvalidNumberRangeType {
    #[gpui_table(filter(number_range()))]
    name: String,
}

fn main() {}
