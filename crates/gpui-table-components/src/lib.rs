pub mod date_range_filter;
pub mod faceted_filter;
pub mod number_range_filter;
pub mod text_filter;

use gpui::{App, Entity, Window};
use std::collections::HashSet;

/// Trait implemented by filter components to enable type-safe filter references.
///
/// This trait allows the `#[gpui_table]` macro to reference filters by their
/// component type (e.g., `filter = TextFilter`) instead of string literals.
///
/// # Example
/// ```ignore
/// #[derive(GpuiTable)]
/// pub struct MyRow {
///     #[gpui_table(filter = TextFilter)]
///     pub name: String,
/// }
/// ```
pub trait TableFilterComponent: Sized {
    /// The type used to store the filter's current value/state.
    type Value: Default + Clone + Send + 'static;

    /// The filter type identifier for registry purposes.
    const FILTER_TYPE: gpui_table_core::registry::RegistryFilterType;

    /// Build the filter component with the given configuration.
    fn build(
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
pub trait FilterValue: Default + Clone + Send + 'static {
    /// Returns true if the filter has no active value.
    fn is_empty(&self) -> bool;

    /// Convert to a string representation suitable for API queries.
    fn to_query_string(&self) -> Option<String>;
}

impl FilterValue for String {
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

impl FilterValue for HashSet<String> {
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

impl FilterValue for (Option<f64>, Option<f64>) {
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

impl FilterValue for (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>) {
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
