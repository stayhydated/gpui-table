use chrono::NaiveDate;
use gpui_table::{Filterable, GpuiTable, TableCell};

#[derive(Clone, Debug, Eq, Filterable, Hash, PartialEq, TableCell)]
enum Status {
    Active,
    Suspended,
}

#[derive(Clone, GpuiTable)]
#[gpui_table(filters)]
struct User {
    #[gpui_table(filter(gpui_table_component::TextFilter))]
    name: String,

    #[gpui_table(filter(gpui_table_component::FacetedFilter::<Status>))]
    status: Status,

    #[gpui_table(filter(gpui_table_component::FacetedFilter::<Status>))]
    status_history: Vec<Status>,

    #[gpui_table(filter(gpui_table_component::DateRangeFilter))]
    created_on: NaiveDate,
}

fn main() {}
