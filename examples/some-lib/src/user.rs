use es_fluent::EsFluentKv;
use gpui::IntoElement;
use gpui_table::NamedTableRow;

#[derive(NamedTableRow, EsFluentKv)]
#[fluent_kv(display = "std")]
#[fluent_kv(this, keys = ["description", "label"])]
#[table(id = "users", title = "Users", fluent = "description")]
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
