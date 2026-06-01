use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct DateRangeWithoutChronoFeature {
    #[gpui_table(filter(gpui_table_component::DateRangeFilter))]
    created_at: chrono::NaiveDate,
}

fn main() {}
