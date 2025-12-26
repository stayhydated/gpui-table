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
#[derive(Clone, Debug, fake::Dummy, es_fluent::EsFluent, Filterable, PartialEq, TableCell)]
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
#[derive(Clone, Debug, fake::Dummy, es_fluent::EsFluent, Filterable, PartialEq, TableCell)]
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
#[derive(fake::Dummy, EsFluentKv, EsFluentThis, GpuiTable)]
#[fluent_this(origin, members)]
#[fluent_kv(keys = ["description", "label"])]
#[gpui_table(fluent = "label")]
pub struct FilterShowcase {
    #[gpui_table(skip)]
    #[dummy(faker = "UUIDv4")]
    #[allow(dead_code)]
    id: uuid::Uuid,

    // TextFilter examples
    #[gpui_table(sortable, width = 150., filter = TextFilter)]
    #[dummy(faker = "Name()")]
    name: String,

    #[gpui_table(width = 200., filter = TextFilter)]
    #[dummy(faker = "SafeEmail()")]
    email: String,

    #[gpui_table(width = 150., filter = TextFilter)]
    #[dummy(faker = "CompanyName()")]
    company: String,

    #[gpui_table(width = 250., filter = TextFilter)]
    #[dummy(faker = "Sentence(3..8)")]
    description: String,

    // NumberRangeFilter examples
    #[gpui_table(sortable, width = 80., filter = NumberRangeFilter)]
    #[dummy(faker = "18..80")]
    age: u8,

    #[gpui_table(sortable, width = 100., filter = NumberRangeFilter)]
    #[dummy(faker = "0..100")]
    score: u8,

    #[gpui_table(sortable, width = 120., filter = NumberRangeFilter)]
    #[dummy(faker = "PositiveDecimal")]
    amount: Decimal,

    #[gpui_table(sortable, width = 100., filter = NumberRangeFilter)]
    #[dummy(faker = "1..1000")]
    quantity: u32,

    // FacetedFilter examples
    #[gpui_table(width = 80., filter = FacetedFilter)]
    active: bool,

    #[gpui_table(width = 80., filter = FacetedFilter)]
    verified: bool,

    #[gpui_table(width = 100., filter = FacetedFilter)]
    priority: Priority,

    #[gpui_table(width = 120., filter = FacetedFilter)]
    category: Category,

    // DateRangeFilter examples
    #[gpui_table(sortable, width = 180., filter = DateRangeFilter)]
    #[dummy(faker = "DateTime()")]
    created_at: chrono::DateTime<chrono::Utc>,

    #[gpui_table(sortable, width = 180., filter = DateRangeFilter)]
    #[dummy(faker = "DateTime()")]
    updated_at: chrono::DateTime<chrono::Utc>,

    #[gpui_table(sortable, width = 180., filter = DateRangeFilter)]
    #[dummy(faker = "DateTime()")]
    due_date: chrono::DateTime<chrono::Utc>,
}
