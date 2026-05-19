use gpui_table::{Filterable, GpuiTable, TableCell};

#[derive(Clone, Debug, Eq, Filterable, Hash, PartialEq, TableCell)]
enum Status {
    Active,
    Inactive,
}

#[derive(Clone, GpuiTable)]
#[gpui_table(filters)]
struct OptionalFacetedRow {
    #[gpui_table(filter(faceted()))]
    status: Option<Status>,
}

fn main() {
    let _ = OptionalFacetedRowFilterValues::default();
}
