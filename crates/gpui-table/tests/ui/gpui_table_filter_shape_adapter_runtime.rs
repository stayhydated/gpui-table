use gpui_table::{GpuiTable, GpuiTableFilterShape};

#[derive(GpuiTableFilterShape)]
#[gpui_table_filter_shape(
    base = gpui_table_component::TextFilter,
    field = String
)]
struct RuntimeTextFilter;

#[derive(Clone, Debug, GpuiTable)]
#[gpui_table(filters)]
struct RuntimeAdapterRow {
    #[gpui_table(filter(RuntimeTextFilter))]
    name: String,
}

fn assert_shape_contracts()
where
    RuntimeTextFilter: gpui_table::runtime::shape::DeclaredComponentShape
        + gpui_table::runtime::shape::DeclaredGpuiTableFilterShape
        + gpui_table::runtime::shape::GpuiTableFilterShapeFor<String>,
{
}

fn main() {
    assert_shape_contracts();
    let _ = RuntimeAdapterRowFilterValues::default();
}
