use es_fluent::EsFluentKv;
use fake::decimal::PositiveDecimal;
use fake::faker::{chrono::en::DateTime, internet::en::SafeEmail, name::en::Name};
use fake::uuid::UUIDv4;
use gpui_table::{GpuiTable, TableCell};
use rust_decimal::Decimal;

#[derive(Clone, Debug, fake::Dummy, es_fluent::EsFluent, PartialEq, TableCell)]
pub enum UserStatus {
    Active,
    Suspended,
    Offline,
}

#[derive(fake::Dummy, EsFluentKv, GpuiTable)]
#[fluent_kv(this, keys = ["description", "label"])]
#[gpui_table(fluent = "label")]
pub struct User {
    #[gpui_table(skip)]
    #[dummy(faker = "UUIDv4")]
    id: uuid::Uuid,

    #[gpui_table(sortable, width = 150.)]
    #[dummy(faker = "Name()")]
    name: String,

    #[gpui_table(sortable, width = 80.)]
    #[dummy(faker = "18..67")]
    age: u8,

    #[gpui_table(sortable, width = 150.)]
    #[dummy(faker = "PositiveDecimal")]
    debt: Decimal,

    #[gpui_table(width = 200.)]
    #[dummy(faker = "SafeEmail()")]
    email: String,

    #[gpui_table(width = 70.)]
    active: bool,

    #[gpui_table(width = 100.)]
    status: UserStatus,

    #[gpui_table(sortable, width = 300.)]
    #[dummy(faker = "DateTime()")]
    created_at: chrono::DateTime<chrono::Utc>,
}
