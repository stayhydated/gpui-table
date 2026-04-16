use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct InvalidDateRangeType {
    #[gpui_table(filter(date_range()))]
    name: String,
}

fn main() {}
