use gpui_table::TableCell;
use std::fmt;

#[derive(TableCell)]
#[table_cell(display)]
struct AccountCode(String);

impl fmt::Display for AccountCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

fn format_percentage(value: &Percentage) -> String {
    format!("{}%", value.0)
}

#[derive(TableCell)]
#[table_cell(format = format_percentage)]
struct Percentage(i64);

fn main() {}
