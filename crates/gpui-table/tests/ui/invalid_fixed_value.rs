use gpui_table::GpuiTable;

#[derive(GpuiTable)]
struct InvalidFixedValue {
    #[gpui_table(fixed = "middle")]
    name: String,
}

fn main() {}
