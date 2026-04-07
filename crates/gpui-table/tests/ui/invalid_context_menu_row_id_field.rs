use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(context_menu_row_id = "user_id", context_menu_route = "/users/{id}")]
struct InvalidContextMenuRowIdField {
    id: u64,
}

fn main() {}
