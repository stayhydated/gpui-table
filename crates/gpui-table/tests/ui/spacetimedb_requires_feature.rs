use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct DateRangeWithoutSpacetimeDbFeature {
    #[gpui_table(filter(gpui_table_component::DateRangeFilter))]
    created_at: spacetimedb_lib::Timestamp,
}

fn main() {}
