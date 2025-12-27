use es_fluent::{EsFluentKv, EsFluentThis};
use fake::decimal::PositiveDecimal;
use fake::faker::{
    chrono::en::DateTime, company::en::CompanyName, internet::en::SafeEmail, lorem::en::Sentence,
    name::en::Name,
};
use fake::uuid::UUIDv4;
use gpui_component::IconName;
use gpui_table::components::{DateRangeFilter, FacetedFilter, NumberRangeFilter, TextFilter};
use gpui_table::{Filterable, GpuiTable, TableCell};
use rust_decimal::Decimal;

/// Priority levels for tasks/items
#[derive(
    Clone,
    Debug,
    fake::Dummy,
    es_fluent::EsFluent,
    Filterable,
    PartialEq,
    TableCell,
    strum::EnumIter,
)]
#[filter(fluent)]
pub enum Priority {
    #[filter(icon = IconName::ArrowDown)]
    Low,
    #[filter(icon = IconName::Minus)]
    Medium,
    #[filter(icon = IconName::ArrowUp)]
    High,
    #[filter(icon = IconName::TriangleAlert)]
    Critical,
}

/// Categories for classification
#[derive(
    Clone,
    Debug,
    fake::Dummy,
    es_fluent::EsFluent,
    Filterable,
    PartialEq,
    TableCell,
    strum::EnumIter,
)]
#[filter(fluent)]
pub enum Category {
    #[filter(icon = IconName::Settings)]
    Engineering,
    #[filter(icon = IconName::Palette)]
    Design,
    #[filter(icon = IconName::ChartPie)]
    Marketing,
    #[filter(icon = IconName::Star)]
    Sales,
    #[filter(icon = IconName::User)]
    Support,
}

/// A comprehensive example struct that showcases all filter types
#[derive(Clone, fake::Dummy, EsFluentKv, EsFluentThis, GpuiTable)]
#[fluent_this(origin, members)]
#[fluent_kv(keys = ["description", "label"])]
#[gpui_table(fluent = "label")]
pub struct FilterShowcase {
    #[gpui_table(skip)]
    #[dummy(faker = "UUIDv4")]
    #[allow(dead_code)]
    pub id: uuid::Uuid,

    // TextFilter examples
    #[gpui_table(sortable, width = 150., filter = TextFilter)]
    #[dummy(faker = "Name()")]
    pub name: String,

    #[gpui_table(width = 200., filter = TextFilter)]
    #[dummy(faker = "SafeEmail()")]
    pub email: String,

    #[gpui_table(width = 150., filter = TextFilter)]
    #[dummy(faker = "CompanyName()")]
    pub company: String,

    #[gpui_table(width = 250., filter = TextFilter)]
    #[dummy(faker = "Sentence(3..8)")]
    pub description: String,

    // NumberRangeFilter examples
    #[gpui_table(sortable, width = 80., filter = NumberRangeFilter)]
    #[dummy(faker = "18..80")]
    pub age: u8,

    #[gpui_table(sortable, width = 100., filter = NumberRangeFilter)]
    #[dummy(faker = "0..100")]
    pub score: u8,

    #[gpui_table(sortable, width = 120., filter = NumberRangeFilter)]
    #[dummy(faker = "PositiveDecimal")]
    pub amount: Decimal,

    #[gpui_table(sortable, width = 100., filter = NumberRangeFilter)]
    #[dummy(faker = "1..1000")]
    pub quantity: u32,

    // FacetedFilter examples
    #[gpui_table(width = 80., filter = FacetedFilter)]
    pub active: bool,

    #[gpui_table(width = 80., filter = FacetedFilter)]
    pub verified: bool,

    #[gpui_table(width = 100., filter = FacetedFilter)]
    pub priority: Priority,

    #[gpui_table(width = 120., filter = FacetedFilter)]
    pub category: Category,

    // DateRangeFilter examples
    #[gpui_table(sortable, width = 180., filter = DateRangeFilter)]
    #[dummy(faker = "DateTime()")]
    pub created_at: chrono::DateTime<chrono::Utc>,

    #[gpui_table(sortable, width = 180., filter = DateRangeFilter)]
    #[dummy(faker = "DateTime()")]
    pub updated_at: chrono::DateTime<chrono::Utc>,

    #[gpui_table(sortable, width = 180., filter = DateRangeFilter)]
    #[dummy(faker = "DateTime()")]
    pub due_date: chrono::DateTime<chrono::Utc>,
}
