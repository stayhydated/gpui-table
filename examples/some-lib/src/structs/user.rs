use es_fluent::EsFluentKv;
use fake::decimal::PositiveDecimal;
use fake::faker::{chrono::en::DateTime, internet::en::SafeEmail, name::en::Name};
use fake::uuid::UUIDv4;
use gpui_table::{Filterable, GpuiTable, TableCell};
use rust_decimal::Decimal;

#[derive(Clone, Debug, fake::Dummy, es_fluent::EsFluent, PartialEq, TableCell, Filterable)]
#[filter(fluent)]
pub enum UserStatus {
    #[filter(icon = "Check")]
    Active,
    #[filter(icon = "CircleX")]
    Suspended,
    #[filter(icon = "Moon")]
    Offline,
}

#[derive(fake::Dummy, EsFluentKv, GpuiTable)]
#[fluent_kv(this, keys = ["description", "label"])]
#[gpui_table(fluent = "label")]
pub struct User {
    #[gpui_table(skip)]
    #[dummy(faker = "UUIDv4")]
    #[allow(dead_code)]
    id: uuid::Uuid,

    #[gpui_table(sortable, width = 150., filter = "Text")]
    #[dummy(faker = "Name()")]
    name: String,

    #[gpui_table(sortable, width = 80., filter = "Number")]
    #[dummy(faker = "18..67")]
    age: u8,

    #[gpui_table(sortable, width = 150., filter = "Number")]
    #[dummy(faker = "PositiveDecimal")]
    debt: Decimal,

    #[gpui_table(width = 200., filter = "Text")]
    #[dummy(faker = "SafeEmail()")]
    email: String,

    #[gpui_table(width = 70., filter = "Faceted")]
    active: bool,

    #[gpui_table(width = 100., filter = "Faceted")]
    status: UserStatus,

    #[gpui_table(sortable, width = 300., filter = "Date")]
    #[dummy(faker = "DateTime()")]
    created_at: chrono::DateTime<chrono::Utc>,
}
