pub mod date_range_filter;
pub mod faceted_filter;
pub mod i18n;
pub mod infinite_faceted_filter;
pub mod number_range_filter;
pub mod reset_filters;
#[cfg(feature = "story")]
mod stories;
pub mod table_status_bar;
pub mod text_filter;

// Re-export extension traits for convenience
pub use date_range_filter::DateRangeFilterExt;
pub use faceted_filter::FacetedFilterExt;
pub use infinite_faceted_filter::InfiniteFacetedFilter;
pub use number_range_filter::NumberRangeFilterExt;
pub use reset_filters::ResetFilters;
pub use table_status_bar::TableStatusBar;
pub use text_filter::TextFilterExt;

use gpui::{App, Entity, Window};
use std::collections::HashSet;

/// Constructor interface shared by the built-in table filter components.
///
/// Generated `XxxFilterEntities` use this trait to instantiate the built-in
/// filter components supported by `#[derive(GpuiTable)]`.
///
/// Implementing this trait can still be useful for standalone components in
/// your own UI code, but the derive macro currently only wires the built-in
/// `filter(text())`, `filter(number_range())`, `filter(date_range())`,
/// `filter(faceted())`, and `filter(infinite_faceted_filter())` syntaxes.
pub trait TableFilterComponent: Sized {
    /// The type used to store the filter's current value/state.
    type Value: Default + Clone + Send + 'static;

    /// The filter type identifier for registry purposes.
    const FILTER_TYPE: gpui_table_schema::registry::RegistryFilterType;

    /// Create the filter component with the given configuration.
    fn new(
        title: impl Into<String>,
        value: Self::Value,
        on_change: impl Fn(Self::Value, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self>;
}

/// Marker trait for filter value types that can be converted to query parameters.
///
/// This trait enables filter values to be accessed and used in data fetching
/// functions like `load_more`.
pub trait QueryFilterValue: Default + Clone + Send + 'static {
    /// Returns true if the filter has no active value.
    fn is_empty(&self) -> bool;

    /// Convert to a string representation suitable for API queries.
    fn to_query_string(&self) -> Option<String>;
}

impl QueryFilterValue for String {
    fn is_empty(&self) -> bool {
        String::is_empty(self)
    }

    fn to_query_string(&self) -> Option<String> {
        if self.is_empty() {
            None
        } else {
            Some(self.clone())
        }
    }
}

impl QueryFilterValue for HashSet<String> {
    fn is_empty(&self) -> bool {
        HashSet::is_empty(self)
    }

    fn to_query_string(&self) -> Option<String> {
        if self.is_empty() {
            None
        } else {
            Some(self.iter().cloned().collect::<Vec<_>>().join(","))
        }
    }
}

impl QueryFilterValue for (Option<f64>, Option<f64>) {
    fn is_empty(&self) -> bool {
        self.0.is_none() && self.1.is_none()
    }

    fn to_query_string(&self) -> Option<String> {
        match (self.0, self.1) {
            (None, None) => None,
            (Some(min), None) => Some(format!(">={}", min)),
            (None, Some(max)) => Some(format!("<={}", max)),
            (Some(min), Some(max)) => Some(format!("{}-{}", min, max)),
        }
    }
}

impl QueryFilterValue for (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>) {
    fn is_empty(&self) -> bool {
        self.0.is_none() && self.1.is_none()
    }

    fn to_query_string(&self) -> Option<String> {
        match (&self.0, &self.1) {
            (None, None) => None,
            (Some(start), None) => Some(format!(">={}", start)),
            (None, Some(end)) => Some(format!("<={}", end)),
            (Some(start), Some(end)) => Some(format!("{} to {}", start, end)),
        }
    }
}

impl<T> QueryFilterValue for Option<T>
where
    T: gpui_table_core::filter::FilterValue + Clone + Send + 'static,
{
    fn is_empty(&self) -> bool {
        self.is_none()
    }

    fn to_query_string(&self) -> Option<String> {
        self.as_ref()
            .map(gpui_table_core::filter::FilterValue::to_filter_string)
    }
}
