use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct DateRangeWithoutJiffFeature {
    #[gpui_table(filter(gpui_table_component::DateRangeFilter))]
    created_at: jiff::civil::Date,
}

fn main() {}
