use es_fluent::EsFluentKv;
use fake::decimal::PositiveDecimal;
use fake::faker::{chrono::en::DateTime, internet::en::SafeEmail, name::en::Name};
use fake::uuid::UUIDv4;
use gpui_table::NamedTableRow;
use rust_decimal::Decimal;

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
    #[dummy(faker = "18..67")]
    age: u8,

    #[table(sortable, width = 150.)]
    #[dummy(faker = "PositiveDecimal")]
    debt: Decimal,

    #[table(width = 200.)]
    #[dummy(faker = "SafeEmail()")]
    email: String,

    #[table(width = 70.)]
    active: bool,

    #[table(sortable, width = 300.)]
    #[dummy(faker = "DateTime()")]
    created_at: chrono::DateTime<chrono::Utc>,
}
