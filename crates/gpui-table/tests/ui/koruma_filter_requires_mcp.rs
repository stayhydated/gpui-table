use gpui_table::GpuiTable;

#[derive(Clone, GpuiTable)]
#[gpui_table(filters)]
struct UserRow {
    #[gpui_table(filter(gpui_table::runtime::shape::TextFilter))]
    #[koruma(koruma_collection::collection::LenValidation::<_>::min(2))]
    name: String,
}

fn main() {}
