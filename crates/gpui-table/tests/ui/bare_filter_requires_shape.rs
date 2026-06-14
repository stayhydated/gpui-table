use gpui_table::GpuiTable;

#[derive(Clone, GpuiTable)]
#[gpui_table(filters)]
struct User {
    #[gpui_table(filter)]
    name: String,
}

fn main() {}
