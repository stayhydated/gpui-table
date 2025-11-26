use es_fluent::EsFluentKv;
use gpui::IntoElement;
use gpui_table::NamedTableRow;

#[derive(NamedTableRow, EsFluentKv)]
#[fluent_kv(display = "std")]
#[fluent_kv(this)]
#[table(fluent)]
pub struct Fruit {
    #[table(width = 100.)]
    name: String,

    #[table(width = 80.)]
    color: String,

    #[table(width = 60.)]
    weight_grams: u32,

    #[table(width = 50.)]
    ripe: bool,
}
