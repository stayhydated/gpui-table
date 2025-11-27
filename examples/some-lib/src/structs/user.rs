use es_fluent::EsFluentKv;
use fake::faker::{internet::en::SafeEmail, name::en::Name};
use fake::uuid::UUIDv4;
use gpui_table::NamedTableRow;

#[derive(fake::Dummy, EsFluentKv, NamedTableRow)]
#[fluent_kv(display = "std")]
#[fluent_kv(this, keys = ["description", "label"])]
#[table(fluent = "label")]
pub struct User {
    #[table(skip)]
    #[dummy(faker = "UUIDv4")]
    id: uuid::Uuid,

    #[table(sortable, width = 150.)]
    #[dummy(faker = "Name()")]
    name: String,

    #[table(sortable, width = 80.)]
    #[dummy(faker = "18..90")]
    age: u8,

    #[table(width = 200.)]
    #[dummy(faker = "SafeEmail()")]
    email: String,

    #[table(width = 50.)]
    active: bool,
    // #[table(width = 50.)]
    // #[dummy(faker = "DateTime()")]
    // created_at: chrono::DateTime<chrono::Utc>,
}
