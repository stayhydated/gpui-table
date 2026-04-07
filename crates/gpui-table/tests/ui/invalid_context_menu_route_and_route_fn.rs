use gpui_table::GpuiTable;

fn route_for_id(id: &u64) -> String {
    format!("/users/{id}")
}

#[derive(GpuiTable)]
#[gpui_table(
    context_menu_row_id = "id",
    context_menu_route = "/users/{id}",
    context_menu_route_fn = route_for_id
)]
struct InvalidContextMenuRouteAndRouteFn {
    id: u64,
}

fn main() {}
