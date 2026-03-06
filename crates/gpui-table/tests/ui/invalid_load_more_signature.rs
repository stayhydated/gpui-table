use gpui::{Context, Window};
use gpui_component::table::TableState;
use gpui_table::{GpuiTable, gpui_table_impl};

#[derive(GpuiTable)]
#[gpui_table(load_more)]
struct BadLoadMoreRow {
    id: u32,
}

#[gpui_table_impl]
impl BadLoadMoreRowTableDelegate {
    #[load_more]
    fn load_more(&self, _window: &mut Window, _cx: &mut Context<TableState<Self>>) {
    }
}

fn main() {}
