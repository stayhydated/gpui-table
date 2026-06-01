use gpui_table::GpuiTable;

#[derive(Clone, GpuiTable)]
#[gpui_table(filters)]
struct LegacyFilterAliasRejected {
    #[gpui_table(filter(gpui_table_component::filters::Text))]
    name: String,
}

fn main() {}
