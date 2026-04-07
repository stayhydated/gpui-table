use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(context_menu_route = "/users/{id}")]
struct InvalidContextMenuMultipleIdFields {
    #[gpui_table(context_menu_id)]
    id: u64,
    #[gpui_table(context_menu_id)]
    external_id: u64,
}

fn main() {}
