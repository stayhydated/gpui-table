use gpui_table::GpuiTable;

type BoolFilter = gpui_table_component::FacetedFilter<bool>;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct InvalidFacetedFilterType {
    #[gpui_table(filter(BoolFilter))]
    name: String,
}

fn main() {}
