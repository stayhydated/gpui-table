use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct InvalidFacetedOptionType {
    #[gpui_table(filter(faceted()))]
    active: Option<bool>,
}

fn main() {}
