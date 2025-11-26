use es_fluent::EsFluentKv;
use gpui::IntoElement;
use gpui_table::NamedTableRow;

#[derive(NamedTableRow, EsFluentKv)]
#[fluent_kv(display = "std")]
#[fluent_kv(this, keys = ["description", "label"])]
#[table(fluent = "label")]
pub struct User {
    #[table(skip)]
    id: usize,

    #[table(sortable, width = 150.)]
    name: String,

    #[table(sortable, width = 80.)]
    age: u32,

    #[table(width = 200.)]
    email: String,

    #[table(width = 50.)]
    active: bool,
}
