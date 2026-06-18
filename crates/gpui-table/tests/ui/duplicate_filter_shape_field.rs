use gpui_table::GpuiTableFilterShape;

#[derive(GpuiTableFilterShape)]
#[gpui_table_filter_shape(
    base = gpui_table_component::TextFilter,
    field = String,
    fields(String)
)]
struct DuplicateFieldFilter;

fn main() {}
