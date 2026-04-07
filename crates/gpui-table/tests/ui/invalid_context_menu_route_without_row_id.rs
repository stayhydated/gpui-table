use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(context_menu_route = "/users/{id}")]
struct InvalidContextMenuRouteWithoutRowId {
    id: u64,
}

fn main() {}
