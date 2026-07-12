use gpui_table::{Filterable, GpuiTable, TableCell};

#[derive(Clone, Debug, Eq, Filterable, Hash, PartialEq, TableCell)]
enum Status {
    Active,
    Suspended,
}

#[derive(Clone, GpuiTable)]
#[gpui_table(filters)]
struct User {
    #[gpui_table(filter(gpui_table_component::TextFilter.numeric_only()))]
    code: String,

    #[gpui_table(filter(gpui_table_component::TextFilter.alphanumeric_only()))]
    identifier: String,

    #[gpui_table(filter(gpui_table_component::TextFilter.matching_regex(r"[A-Z0-9-]*")))]
    external_ref: String,

    #[gpui_table(filter(gpui_table_component::FacetedFilter::<Status>.searchable(true)))]
    status: Status,
}

fn main() {}
