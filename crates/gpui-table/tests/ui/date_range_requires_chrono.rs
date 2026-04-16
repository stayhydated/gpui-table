use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct DateRangeWithoutChronoFeature {
    #[gpui_table(filter(date_range()))]
    created_at: chrono::NaiveDate,
}

fn main() {}
