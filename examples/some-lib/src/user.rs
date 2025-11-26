use gpui::{App, Div, IntoElement, Stateful, Window};
use gpui_component::table::{Column, TableDelegate};
use gpui_table::{NamedTableRow, TableRowMeta as _, TableRowStyle as _};

#[derive(NamedTableRow)]
#[table(id = "users", title = "Users")]
pub struct User {
    #[table(col = "id", title = "ID", width = 60.)]
    id: usize,

    #[table(col = "name", title = "Name", sortable, width = 150.)]
    name: String,

    #[table(col = "age", title = "Age", sortable, width = 80.)]
    age: u32,

    #[table(col = "email", title = "Email", width = 200.)]
    email: String,

    #[table(col = "active", title = "Active", width = 50.)]
    active: bool,
}
