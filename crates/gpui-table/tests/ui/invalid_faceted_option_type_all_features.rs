use gpui_table::GpuiTable;

type BoolFilter = gpui_table_component::FacetedFilter<bool>;

#[derive(GpuiTable)]
#[gpui_table(filters)]
struct InvalidFacetedOptionType {
    #[gpui_table(filter(BoolFilter))]
    label: Option<String>,
}

fn main() {}
