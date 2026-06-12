use chrono::NaiveDate;
use gpui_table::{Filterable, GpuiTable, TableCell};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Filterable, TableCell)]
enum Status {
    Active,
    Suspended,
}

#[derive(Clone, GpuiTable)]
#[gpui_table(filters)]
struct User {
    #[gpui_table(filter)]
    name: String,

    #[gpui_table(filter)]
    status: Status,

    #[gpui_table(filter)]
    status_history: Vec<Status>,

    #[gpui_table(filter)]
    created_on: NaiveDate,
}

fn main() {}
