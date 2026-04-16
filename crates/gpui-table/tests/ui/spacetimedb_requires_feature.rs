use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct DateRangeWithoutSpacetimeDbFeature {
    #[gpui_table(filter(date_range()))]
    created_at: spacetimedb_lib::Timestamp,
}

fn main() {}
