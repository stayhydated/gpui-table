
use es_fluent::{EsFluentKv, EsFluentThis};
use fake::decimal::PositiveDecimal;
use fake::faker::{
    chrono::en::DateTime, company::en::CompanyName, internet::en::SafeEmail, lorem::en::Sentence,
    name::en::Name,
};
use fake::uuid::UUIDv4;

use gpui::{Context, Window};
use gpui_component::IconName;
use gpui_component::table::TableState;
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
    /// Initialize with a pre-generated data pool
    pub fn with_data_pool(all_data: Vec<FilterShowcase>) -> Self {
        let rows = all_data.clone();
        Self {
            rows,
            columns: <FilterShowcase as gpui_table::TableRowMeta>::table_columns(),
            visible_rows: Default::default(),
            visible_cols: Default::default(),
            eof: true, // All data is already loaded
            loading: false,
            full_loading: false,
        }
    }

    /// Apply all filters to the data pool and update displayed rows
    pub fn apply_filters(
        &mut self,
        all_data: &[FilterShowcase],
        filters: &FilterShowcaseFilterValues,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) {
        self.rows = all_data
            .iter()
            .filter(|row| {
                // Text filters
                let name_matches = filters.name.is_empty()
                    || row.name.to_lowercase().contains(&filters.name.to_lowercase());
                let email_matches = filters.email.is_empty()
                    || row.email.to_lowercase().contains(&filters.email.to_lowercase());
                let company_matches = filters.company.is_empty()
                    || row.company.to_lowercase().contains(&filters.company.to_lowercase());
                let desc_matches = filters.description.is_empty()
                    || row.description.to_lowercase().contains(&filters.description.to_lowercase());

                // Number range filters
                let age_matches = match (filters.age.0, filters.age.1) {
                    (Some(min), Some(max)) => (row.age as f64) >= min && (row.age as f64) <= max,
                    (Some(min), None) => (row.age as f64) >= min,
                    (None, Some(max)) => (row.age as f64) <= max,
                    (None, None) => true,
                };
                let score_matches = match (filters.score.0, filters.score.1) {
                    (Some(min), Some(max)) => (row.score as f64) >= min && (row.score as f64) <= max,
                    (Some(min), None) => (row.score as f64) >= min,
                    (None, Some(max)) => (row.score as f64) <= max,
                    (None, None) => true,
                };

                // Faceted filters (empty means no filter applied)
                let active_matches = filters.active.is_empty()
                    || filters.active.contains(&row.active.to_string().to_lowercase())
                    || filters.active.contains(if row.active { "True" } else { "False" });
                let verified_matches = filters.verified.is_empty()
                    || filters.verified.contains(&row.verified.to_string().to_lowercase())
                    || filters.verified.contains(if row.verified { "True" } else { "False" });
                let priority_matches = filters.priority.is_empty()
                    || filters.priority.contains(row.priority.variant_name());
                let category_matches = filters.category.is_empty()
                    || filters.category.contains(row.category.variant_name());

                // Date range filters
                let created_at_matches = match (filters.created_at.0, filters.created_at.1) {
                    (Some(start), Some(end)) => {
                        let date = row.created_at.date_naive();
                        date >= start && date <= end
                    }
                    (Some(start), None) => row.created_at.date_naive() >= start,
                    (None, Some(end)) => row.created_at.date_naive() <= end,
                    (None, None) => true,
                };

                name_matches
                    && email_matches
                    && company_matches
                    && desc_matches
                    && age_matches
                    && score_matches
                    && active_matches
                    && verified_matches
                    && priority_matches
                    && category_matches
                    && created_at_matches
            })
            .cloned()
            .collect();

        log::info!("Filtered to {} rows from {} total", self.rows.len(), all_data.len());
        cx.notify();
    }

    /// Load more data (no-op: all data is pre-loaded)
    pub fn load_more_data(&mut self, _window: &mut Window, _cx: &mut Context<TableState<Self>>) {
        // No-op: all data is pre-loaded
    }

    /// Reset and reload data (for compatibility)
    pub fn reset_and_reload(&mut self, _window: &mut Window, _cx: &mut Context<TableState<Self>>) {
        // This is now handled by the story calling apply_filters directly
    }
}

