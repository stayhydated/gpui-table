use gpui_table::GpuiTable;
use rust_decimal::Decimal;

#[derive(Clone, GpuiTable)]
#[gpui_table(filters)]
struct Product {
    #[gpui_table(filter(
        gpui_table_component::NumberRangeFilter
            .range(Decimal::new(0, 0), Decimal::new(100, 0))
            .step(Decimal::new(10, 0))
    ))]
    score: Decimal,
}

fn main() {}
