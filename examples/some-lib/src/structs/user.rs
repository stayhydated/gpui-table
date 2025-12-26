use es_fluent::{EsFluentKv, EsFluentThis};
use fake::decimal::PositiveDecimal;
use fake::faker::{chrono::en::DateTime, internet::en::SafeEmail, name::en::Name};
use fake::uuid::UUIDv4;
use gpui_component::IconName;
use gpui_table::components::{DateRangeFilter, FacetedFilter, NumberRangeFilter, TextFilter};
use gpui_table::{Filterable, GpuiTable, TableCell};
use rust_decimal::Decimal;

#[derive(Clone, Debug, fake::Dummy, es_fluent::EsFluent, Filterable, PartialEq, TableCell)]
#[filter(fluent)]
pub enum UserStatus {
    #[filter(icon = IconName::Check)]
    Active,
    #[filter(icon = IconName::CircleX)]
    Suspended,
    #[filter(icon = IconName::Moon)]
    Offline,
}

#[derive(fake::Dummy, EsFluentKv, EsFluentThis, GpuiTable)]
#[fluent_this(origin, members)]
#[fluent_kv(keys = ["description", "label"])]
#[gpui_table(fluent = "label")]
pub struct User {
    #[gpui_table(skip)]
    #[dummy(faker = "UUIDv4")]
    #[allow(dead_code)]
    id: uuid::Uuid,

    #[gpui_table(sortable, width = 150., filter = TextFilter)]
    #[dummy(faker = "Name()")]
    name: String,

    #[gpui_table(sortable, width = 80., filter = NumberRangeFilter)]
    #[dummy(faker = "18..67")]
    age: u8,

    #[gpui_table(sortable, width = 150., filter = NumberRangeFilter)]
    #[dummy(faker = "PositiveDecimal")]
    debt: Decimal,

    #[gpui_table(width = 200., filter = TextFilter)]
    #[dummy(faker = "SafeEmail()")]
    email: String,

    #[gpui_table(width = 70., filter = FacetedFilter)]
    active: bool,

    #[gpui_table(width = 100., filter = FacetedFilter)]
    status: UserStatus,

    #[gpui_table(sortable, width = 300., filter = DateRangeFilter)]
    #[dummy(faker = "DateTime()")]
    created_at: chrono::DateTime<chrono::Utc>,
}
