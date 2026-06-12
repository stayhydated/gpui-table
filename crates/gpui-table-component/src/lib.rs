//! Built-in GPUI filter widgets and table support components.
//!
//! This crate owns the concrete UI for text, faceted, number-range, and
//! date-range filters plus `ResetFilters` and `TableStatusBar`. It is useful
//! when applications need direct widget composition; most derive-based tables
//! should reach it through `gpui-table`.
//!
//! Generated filter entities do not construct widgets through ad hoc paths.
//! They target `gpui_table::runtime::generated_filters` and the runtime shape
//! contract. Manual filter collections can use [`TableFilterComponent`] and
//! [`QueryFilterValue`] directly.

#[cfg(feature = "chrono")]
pub mod date_range_filter;
pub mod faceted_filter;
pub mod i18n;
#[cfg(feature = "rust_decimal")]
pub mod number_range_filter;
pub mod reset_filters;
#[cfg(feature = "story")]
mod stories;
pub mod table_status_bar;
pub mod text_filter;

// Re-export component types and extension traits for convenience.
#[cfg(feature = "chrono")]
pub use date_range_filter::{DateRangeFilter, DateRangeFilterExt};
pub use faceted_filter::{FacetedFilter, FacetedFilterExt};
#[cfg(feature = "rust_decimal")]
pub use number_range_filter::{NumberRangeFilter, NumberRangeFilterExt};
pub use reset_filters::ResetFilters;
pub use table_status_bar::TableStatusBar;
pub use text_filter::{TextFilter, TextFilterExt};

use gpui::{App, Entity, Window};
use gpui_table_core::filter::{FacetedValue, FilterValue, RangeValue, SingleValue, TextValue};
use std::collections::HashSet;

/// Constructor interface shared by the built-in table filter components.
///
/// Implementing this trait is useful for standalone components in your own UI
/// code or for custom filter collections you build manually.
/// The derive macro consumes filter shapes, such as
/// `gpui_table_component::TextFilter`; generated filter entities construct
/// those shapes through `gpui_table::runtime::shape::GpuiTableFilterShape`.
pub trait TableFilterComponent: Sized {
    /// The filter's current value/state type.
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
///
/// Generated `XxxFilterValues` from `#[derive(GpuiTable)]` are composed from
/// wrapper types such as `TextValue`, `RangeValue<T>`, and `FacetedValue<T>`.
/// `SingleValue<T>` is also supported for manual integrations. Server-side
/// loaders can typically call `to_query_string()` directly on those wrapper
/// values.
pub trait QueryFilterValue: Default + Clone + Send + 'static {
    /// Returns true if the filter has no active value.
    fn is_empty(&self) -> bool;

    /// Convert to a string representation suitable for API queries.
    fn to_query_string(&self) -> Option<String>;
}

fn range_query_string<T>(min: Option<&T>, max: Option<&T>) -> Option<String>
where
    T: ToString,
{
    match (min, max) {
        (None, None) => None,
        (Some(min), None) => Some(format!(">={}", min.to_string())),
        (None, Some(max)) => Some(format!("<={}", max.to_string())),
        (Some(min), Some(max)) => Some(format!("{}-{}", min.to_string(), max.to_string())),
    }
}

fn sorted_query_values<I>(values: I) -> Option<String>
where
    I: IntoIterator<Item = String>,
{
    let mut values = values.into_iter().collect::<Vec<_>>();
    if values.is_empty() {
        None
    } else {
        values.sort();
        Some(values.join(","))
    }
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
        sorted_query_values(self.iter().cloned())
    }
}

impl QueryFilterValue for (Option<f64>, Option<f64>) {
    fn is_empty(&self) -> bool {
        self.0.is_none() && self.1.is_none()
    }

    fn to_query_string(&self) -> Option<String> {
        range_query_string(self.0.as_ref(), self.1.as_ref())
    }
}

#[cfg(feature = "rust_decimal")]
impl QueryFilterValue for (Option<rust_decimal::Decimal>, Option<rust_decimal::Decimal>) {
    fn is_empty(&self) -> bool {
        self.0.is_none() && self.1.is_none()
    }

    fn to_query_string(&self) -> Option<String> {
        range_query_string(self.0.as_ref(), self.1.as_ref())
    }
}

#[cfg(feature = "chrono")]
impl QueryFilterValue for (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>) {
    fn is_empty(&self) -> bool {
        self.0.is_none() && self.1.is_none()
    }

    fn to_query_string(&self) -> Option<String> {
        range_query_string(self.0.as_ref(), self.1.as_ref())
    }
}

impl<T> QueryFilterValue for Option<T>
where
    T: FilterValue + Clone + Send + 'static,
{
    fn is_empty(&self) -> bool {
        self.is_none()
    }

    fn to_query_string(&self) -> Option<String> {
        self.as_ref().map(FilterValue::to_filter_string)
    }
}

impl QueryFilterValue for TextValue {
    fn is_empty(&self) -> bool {
        !self.is_active()
    }

    fn to_query_string(&self) -> Option<String> {
        (!self.is_empty()).then(|| self.to_string())
    }
}

impl<T> QueryFilterValue for RangeValue<T>
where
    T: Clone + PartialOrd + ToString + Send + 'static,
{
    fn is_empty(&self) -> bool {
        !self.is_active()
    }

    fn to_query_string(&self) -> Option<String> {
        range_query_string(self.min(), self.max())
    }
}

impl<T> QueryFilterValue for FacetedValue<T>
where
    T: FilterValue + Clone + Send + 'static,
{
    fn is_empty(&self) -> bool {
        !self.is_active()
    }

    fn to_query_string(&self) -> Option<String> {
        sorted_query_values(self.0.iter().map(FilterValue::to_filter_string))
    }
}

impl<T> QueryFilterValue for SingleValue<T>
where
    T: FilterValue + Clone + PartialEq + Send + 'static,
{
    fn is_empty(&self) -> bool {
        !self.is_active()
    }

    fn to_query_string(&self) -> Option<String> {
        self.value().map(FilterValue::to_filter_string)
    }
}

#[cfg(test)]
mod tests {
    use super::{FacetedFilter, QueryFilterValue, TextFilter};
    use gpui_table_core::filter::{FacetedValue, FilterValue, RangeValue, SingleValue, TextValue};
    use std::collections::HashSet;

    #[derive(Clone, Eq, Hash, PartialEq)]
    enum Status {
        Active,
        Pending,
    }

    impl FilterValue for Status {
        fn to_filter_string(&self) -> String {
            match self {
                Self::Active => "active",
                Self::Pending => "pending",
            }
            .to_string()
        }

        fn from_filter_string(s: &str) -> Option<Self> {
            match s {
                "active" => Some(Self::Active),
                "pending" => Some(Self::Pending),
                _ => None,
            }
        }
    }

    #[test]
    fn text_value_serializes_like_plain_text() {
        assert_eq!(
            TextValue::from("mark").to_query_string(),
            Some("mark".into())
        );
        assert_eq!(TextValue::default().to_query_string(), None);
    }

    #[test]
    fn faceted_values_serialize_with_filter_value_strings() {
        let values = FacetedValue::from(HashSet::from([Status::Pending, Status::Active]));
        assert_eq!(values.to_query_string(), Some("active,pending".into()));
    }

    #[test]
    fn raw_hash_sets_serialize_in_stable_order() {
        let values = HashSet::from(["pending".to_string(), "active".to_string()]);
        assert_eq!(values.to_query_string(), Some("active,pending".into()));
    }

    #[test]
    fn single_value_serializes_with_filter_value_string() {
        let value = SingleValue::from(Some(Status::Active));
        assert_eq!(value.to_query_string(), Some("active".into()));
    }

    #[test]
    fn generic_range_values_serialize_bounds() {
        assert_eq!(
            RangeValue::from((Some(5_u8), Some(10_u8))).to_query_string(),
            Some("5-10".into())
        );
        assert_eq!(
            RangeValue::from((Some(5_u8), None)).to_query_string(),
            Some(">=5".into())
        );
    }

    fn assert_declared<Shape>()
    where
        Shape: component_shape::DeclaredComponentShape,
    {
    }

    fn assert_shape_for<Shape, Value>()
    where
        Shape: component_shape::ComponentShapeFor<Value>,
    {
    }

    #[test]
    fn built_in_filters_publish_neutral_shape_markers() {
        assert_declared::<TextFilter>();
        assert_shape_for::<TextFilter, String>();
        assert_shape_for::<TextFilter, Option<String>>();

        assert_declared::<FacetedFilter<bool>>();
        assert_shape_for::<FacetedFilter<bool>, bool>();
        assert_shape_for::<FacetedFilter<bool>, Option<bool>>();
        assert_shape_for::<FacetedFilter<bool>, Vec<bool>>();
        assert_shape_for::<FacetedFilter<bool>, Option<Vec<bool>>>();

        #[cfg(feature = "rust_decimal")]
        {
            use super::NumberRangeFilter;

            assert_declared::<NumberRangeFilter>();
            assert_shape_for::<NumberRangeFilter, i64>();
            assert_shape_for::<NumberRangeFilter, Option<i64>>();
            assert_shape_for::<NumberRangeFilter, rust_decimal::Decimal>();
        }

        #[cfg(feature = "chrono")]
        {
            use super::DateRangeFilter;

            assert_declared::<DateRangeFilter>();
            assert_shape_for::<DateRangeFilter, chrono::NaiveDate>();
            assert_shape_for::<DateRangeFilter, Option<chrono::NaiveDate>>();
            assert_shape_for::<DateRangeFilter, chrono::NaiveDateTime>();
        }
    }
}
