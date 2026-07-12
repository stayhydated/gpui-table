use gpui_table::{GpuiTable, gpui_table_impl};

#[derive(GpuiTable)]
#[gpui_table(load_more)]
struct InvalidImplTargetRow {
    id: u32,
}

#[gpui_table_impl]
impl InvalidImplTargetRowTableDelegate {
    fn load_more(&mut self) {}
}

fn main() {}
