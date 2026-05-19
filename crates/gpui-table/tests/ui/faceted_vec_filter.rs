use gpui_table::{Filterable, GpuiTable, TableCell};

#[derive(Clone, Debug, Eq, Filterable, Hash, PartialEq, TableCell)]
enum Role {
    Client,
    Supplier,
}

#[derive(Clone, GpuiTable)]
#[gpui_table(filters)]
struct VecFacetedRow {
    #[gpui_table(filter(faceted()))]
    roles: Vec<Role>,
}

#[derive(Clone, GpuiTable)]
#[gpui_table(filters)]
struct OptionalVecFacetedRow {
    #[gpui_table(filter(faceted()))]
    roles: Option<Vec<Role>>,
}

fn main() {
    let _ = VecFacetedRowFilterValues::default();
    let _ = OptionalVecFacetedRowFilterValues::default();
}
