use es_fluent::{EsFluentKv, EsFluentThis};
use fake::decimal::PositiveDecimal;
use fake::faker::{
    chrono::en::DateTime, company::en::CompanyName, internet::en::SafeEmail, lorem::en::Sentence,
    name::en::Name,
};
use fake::uuid::UUIDv4;
use fake::{Fake, Faker};
use gpui::{Context, Window};
use gpui_component::IconName;
use gpui_component::table::TableState;
use gpui_table::{Filterable, GpuiTable, TableCell};
use rust_decimal::Decimal;
use std::time::Duration;

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
#[gpui_table(load_more = "Self::load_more_data")]
#[gpui_table(load_more_threshold = 20)]
pub struct FilterShowcase {
    #[gpui_table(skip)]
    #[dummy(faker = "UUIDv4")]
    #[allow(dead_code)]
    pub id: uuid::Uuid,

    // TextFilter examples
    #[gpui_table(sortable, width = 150., filter(text(validate = "alphanumeric")))]
    #[dummy(faker = "Name()")]
    pub name: String,

    #[gpui_table(width = 200., filter(text(validate = "alphabetic")))]
    #[dummy(faker = "SafeEmail()")]
    pub email: String,

    #[gpui_table(width = 150., filter(text()))]
    #[dummy(faker = "CompanyName()")]
    pub company: String,

    #[gpui_table(width = 250., filter(text()))]
    #[dummy(faker = "Sentence(3..8)")]
    pub description: String,

    // NumberRangeFilter examples
    #[gpui_table(sortable, width = 80., filter(number_range(min = 18.0, max = 150.0)))]
    #[dummy(faker = "18..80")]
    pub age: u8,

    #[gpui_table(sortable, width = 100., filter(number_range()))]
    #[dummy(faker = "0..100")]
    pub score: u8,

    #[gpui_table(sortable, width = 120., filter(number_range()))]
    #[dummy(faker = "PositiveDecimal")]
    pub amount: Decimal,

    #[gpui_table(sortable, width = 100., filter(number_range()))]
    #[dummy(faker = "1..1000")]
    pub quantity: u32,

    // FacetedFilter examples
    #[gpui_table(width = 80., filter(faceted()))]
    pub active: bool,

    #[gpui_table(width = 80., filter(faceted()))]
    pub verified: bool,

    #[gpui_table(width = 100., filter(faceted(searchable)))]
    pub priority: Priority,

    #[gpui_table(width = 120., filter(faceted(searchable)))]
    pub category: Category,

    // DateRangeFilter examples
    #[gpui_table(sortable, width = 180., filter(date_range()))]
    #[dummy(faker = "DateTime()")]
    pub created_at: chrono::DateTime<chrono::Utc>,

    #[gpui_table(sortable, width = 180., filter(date_range()))]
    #[dummy(faker = "DateTime()")]
    pub updated_at: chrono::DateTime<chrono::Utc>,

    #[gpui_table(sortable, width = 180., filter(date_range()))]
    #[dummy(faker = "DateTime()")]
    pub due_date: chrono::DateTime<chrono::Utc>,
}

impl FilterShowcaseTableDelegate {
    /// Load more filter showcase data with fake data generation.
    pub fn load_more_data(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        if self.loading || self.eof {
            return;
        }

        self.loading = true;
        cx.notify();

        cx.spawn(async move |view, cx| {
            // Simulate network delay
            cx.background_executor()
                .timer(Duration::from_millis(100))
                .await;

            // Generate fake data
            let new_rows: Vec<FilterShowcase> = (0..50).map(|_| Faker.fake()).collect();

            _ = cx.update(|cx| {
                view.update(cx, |table, cx| {
                    let delegate = table.delegate_mut();
                    delegate.rows.extend(new_rows);
                    delegate.loading = false;

                    // Stop after 500 rows for demo purposes
                    if delegate.rows.len() >= 500 {
                        delegate.eof = true;
                    }

                    cx.notify();
                })
                .unwrap();
            });
        })
        .detach();
    }
}
