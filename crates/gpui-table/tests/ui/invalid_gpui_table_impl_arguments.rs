use gpui_table::{GpuiTable, gpui_table_impl};

#[derive(GpuiTable)]
#[gpui_table(load_more)]
struct InvalidImplArgumentsRow {
    id: u32,
}

#[gpui_table_impl(gpui_table::runtime::TableLoader)]
impl gpui_table::runtime::TableLoader for InvalidImplArgumentsRowTableDelegate {
    fn load_more(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
    ) {
    }
}

fn main() {}
