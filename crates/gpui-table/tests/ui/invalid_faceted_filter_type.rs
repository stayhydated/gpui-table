use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct InvalidFacetedFilterType {
    #[gpui_table(filter(faceted()))]
    name: String,
}

fn main() {}
