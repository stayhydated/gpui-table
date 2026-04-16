use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct InvalidInfiniteFacetedFilterType {
    #[gpui_table(filter(infinite_faceted_filter()))]
    name: String,
}

fn main() {}
