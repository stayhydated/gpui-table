use es_fluent::{EsFluentKv, EsFluentThis};
use fake::decimal::PositiveDecimal;
use fake::faker::{chrono::en::DateTime, internet::en::SafeEmail, name::en::Name};
use fake::uuid::UUIDv4;
use gpui_component::IconName;
use gpui_table::{Filterable, GpuiTable, TableCell};
use rust_decimal::Decimal;

#[derive(
    Clone, Debug, Eq, Hash, fake::Dummy, es_fluent::EsFluent, Filterable, PartialEq, TableCell,
)]
#[filter(fluent)]
pub enum UserStatus {
    #[filter(icon = IconName::Check)]
    Active,
    #[filter(icon = IconName::CircleX)]
    Suspended,
    #[filter(icon = IconName::Moon)]
    Offline,
}

#[derive(Clone, fake::Dummy, EsFluentKv, EsFluentThis, GpuiTable)]
#[fluent_this(origin, members)]
#[fluent_kv(keys = ["description", "label"])]
#[gpui_table(fluent = "label", filters)]
pub struct User {
    #[gpui_table(skip)]
    #[dummy(faker = "UUIDv4")]
    #[allow(dead_code)]
    pub id: uuid::Uuid,

    #[gpui_table(sortable, width = 150., filter(text()))]
    #[dummy(faker = "Name()")]
    pub name: String,

    #[gpui_table(sortable, width = 80., filter(number_range()))]
    #[dummy(faker = "18..67")]
    pub age: u8,

    #[gpui_table(sortable, width = 150., filter(number_range()))]
    #[dummy(faker = "PositiveDecimal")]
    pub debt: Decimal,

    #[gpui_table(width = 200., filter(text()))]
    #[dummy(faker = "SafeEmail()")]
    pub email: String,

    #[gpui_table(width = 70., filter(faceted()))]
    pub active: bool,

    #[gpui_table(width = 100., filter(faceted()))]
    pub status: UserStatus,

    #[gpui_table(sortable, width = 300., filter(date_range()))]
    #[dummy(faker = "DateTime()")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}
