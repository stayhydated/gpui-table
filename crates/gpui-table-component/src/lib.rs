//! Built-in GPUI filter widgets and table support components.
//!
//! This crate owns the concrete UI for text, faceted, number-range, and
//! date-range filters plus `ResetFilters` and `TableStatusBar`. It is useful
//! when applications need direct widget composition or built-in filter shapes
//! for `#[derive(gpui_table::GpuiTable)]`.
//!
//! Built-in filter types also implement the runtime table filter shape
//! contract, so they can be used directly in
//! `#[gpui_table(filter(gpui_table_component::TextFilter))]`. Manual filter
//! collections can use [`TableFilterComponent`] and [`QueryFilterValue`]
//! directly.
//!
//! Adapter shapes such as [`TextFilterAdapter`], [`NumberRangeFilterAdapter`],
//! and [`DateRangeFilterAdapter`] reuse the built-in widgets and MCP schemas
//! for application-owned value types that implement the matching field trait.

#[cfg(feature = "chrono")]
pub mod date_range_filter;
pub mod faceted_filter;
pub mod i18n;
#[cfg(feature = "mcp")]
mod mcp;
#[cfg(feature = "rust_decimal")]
pub mod number_range_filter;
pub mod reset_filters;
mod shape;
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
#[cfg(feature = "chrono")]
pub use shape::{DateRangeFilterAdapter, DateRangeFilterField};
pub use shape::{FacetedFilterArgs, TextFilterAdapter, TextFilterArgs, TextFilterField};
#[cfg(feature = "rust_decimal")]
pub use shape::{NumberRangeFilterAdapter, NumberRangeFilterArgs, NumberRangeFilterField};
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
    use super::{FacetedFilter, QueryFilterValue, TextFilter, TextFilterAdapter, TextFilterField};
    use gpui_table_core::filter::{
        FacetedValue, FilterValue, Filterable, RangeValue, SingleValue, TextValue,
    };
    use gpui_table_runtime::shape::{
        DeclaredGpuiTableFilterShape, GpuiTableFilterShape, GpuiTableFilterShapeFor,
    };
    use std::collections::HashSet;

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

    impl Filterable for Status {
        fn options() -> Vec<gpui_table_schema::filter::FacetedFilterOption> {
            Vec::new()
        }
    }

    struct AccountCode(String);

    impl TextFilterField for AccountCode {
        fn to_filter_text(&self) -> String {
            self.0.clone()
        }
    }

    #[cfg(feature = "rust_decimal")]
    struct Amount(rust_decimal::Decimal);

    #[cfg(feature = "rust_decimal")]
    impl super::NumberRangeFilterField for Amount {
        fn to_filter_decimal(&self) -> rust_decimal::Decimal {
            self.0
        }
    }

    #[cfg(feature = "chrono")]
    struct BusinessDate(chrono::NaiveDate);

    #[cfg(feature = "chrono")]
    impl super::DateRangeFilterField for BusinessDate {
        fn to_filter_naive_date(&self) -> chrono::NaiveDate {
            self.0
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
        assert_eq!(
            RangeValue::from((None, Some(10_u8))).to_query_string(),
            Some("<=10".into())
        );
        assert_eq!(RangeValue::<u8>::default().to_query_string(), None);
    }

    #[test]
    fn manual_query_value_implementations_cover_empty_and_bounded_forms() {
        assert!(String::new().is_empty());
        assert_eq!(String::new().to_query_string(), None);
        assert_eq!("value".to_string().to_query_string(), Some("value".into()));

        assert_eq!(HashSet::<String>::new().to_query_string(), None);
        assert_eq!((None, None::<f64>).to_query_string(), None);
        assert_eq!((Some(1.5), None).to_query_string(), Some(">=1.5".into()));
        assert_eq!((None, Some(2.5)).to_query_string(), Some("<=2.5".into()));
        assert_eq!(
            (Some(1.5), Some(2.5)).to_query_string(),
            Some("1.5-2.5".into())
        );

        let no_status: Option<Status> = None;
        assert!(QueryFilterValue::is_empty(&no_status));
        assert_eq!(no_status.to_query_string(), None);
        assert_eq!(
            Some(Status::Pending).to_query_string(),
            Some("pending".into())
        );

        let single = SingleValue::<Status>::default();
        assert!(QueryFilterValue::is_empty(&single));
        let facets = FacetedValue::<Status>::default();
        assert!(QueryFilterValue::is_empty(&facets));
        assert!(QueryFilterValue::is_empty(&TextValue::default()));
        assert!(QueryFilterValue::is_empty(&RangeValue::<u8>::default()));

        #[cfg(feature = "rust_decimal")]
        {
            use rust_decimal::Decimal;
            let bounds = (Some(Decimal::new(15, 1)), Some(Decimal::new(25, 1)));
            assert!(!QueryFilterValue::is_empty(&bounds));
            assert_eq!(bounds.to_query_string(), Some("1.5-2.5".into()));
        }

        #[cfg(feature = "chrono")]
        {
            use chrono::NaiveDate;
            let bounds = (
                Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
                Some(NaiveDate::from_ymd_opt(2026, 1, 31).unwrap()),
            );
            assert!(!QueryFilterValue::is_empty(&bounds));
            assert_eq!(
                bounds.to_query_string(),
                Some("2026-01-01-2026-01-31".into())
            );
        }
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

    fn assert_table_filter_shape<Shape, Value>()
    where
        Shape: DeclaredGpuiTableFilterShape + GpuiTableFilterShape + GpuiTableFilterShapeFor<Value>,
    {
    }

    #[test]
    fn built_in_filters_publish_neutral_shape_markers() {
        assert_declared::<TextFilter>();
        assert_shape_for::<TextFilter, String>();
        assert_shape_for::<TextFilter, Option<String>>();
        assert_table_filter_shape::<TextFilter, String>();
        assert_table_filter_shape::<TextFilter, Option<String>>();
        assert_declared::<TextFilterAdapter>();
        assert_shape_for::<TextFilterAdapter, AccountCode>();
        assert_shape_for::<TextFilterAdapter, Option<AccountCode>>();
        assert_table_filter_shape::<TextFilterAdapter, AccountCode>();
        assert_table_filter_shape::<TextFilterAdapter, Option<AccountCode>>();

        assert_declared::<FacetedFilter<bool>>();
        assert_shape_for::<FacetedFilter<bool>, bool>();
        assert_shape_for::<FacetedFilter<bool>, Option<bool>>();
        assert_shape_for::<FacetedFilter<bool>, Vec<bool>>();
        assert_shape_for::<FacetedFilter<bool>, Option<Vec<bool>>>();
        assert_table_filter_shape::<FacetedFilter<bool>, bool>();
        assert_table_filter_shape::<FacetedFilter<bool>, Option<bool>>();
        assert_table_filter_shape::<FacetedFilter<bool>, Vec<bool>>();
        assert_table_filter_shape::<FacetedFilter<bool>, Option<Vec<bool>>>();

        #[cfg(feature = "rust_decimal")]
        {
            use super::{NumberRangeFilter, NumberRangeFilterAdapter};

            assert_declared::<NumberRangeFilter>();
            assert_shape_for::<NumberRangeFilter, i64>();
            assert_shape_for::<NumberRangeFilter, Option<i64>>();
            assert_shape_for::<NumberRangeFilter, rust_decimal::Decimal>();
            assert_table_filter_shape::<NumberRangeFilter, i64>();
            assert_table_filter_shape::<NumberRangeFilter, Option<i64>>();
            assert_table_filter_shape::<NumberRangeFilter, rust_decimal::Decimal>();
            assert_declared::<NumberRangeFilterAdapter>();
            assert_shape_for::<NumberRangeFilterAdapter, Amount>();
            assert_shape_for::<NumberRangeFilterAdapter, Option<Amount>>();
            assert_table_filter_shape::<NumberRangeFilterAdapter, Amount>();
            assert_table_filter_shape::<NumberRangeFilterAdapter, Option<Amount>>();
        }

        #[cfg(feature = "chrono")]
        {
            use super::{DateRangeFilter, DateRangeFilterAdapter};

            assert_declared::<DateRangeFilter>();
            assert_shape_for::<DateRangeFilter, chrono::NaiveDate>();
            assert_shape_for::<DateRangeFilter, Option<chrono::NaiveDate>>();
            assert_shape_for::<DateRangeFilter, chrono::NaiveDateTime>();
            assert_table_filter_shape::<DateRangeFilter, chrono::NaiveDate>();
            assert_table_filter_shape::<DateRangeFilter, Option<chrono::NaiveDate>>();
            assert_table_filter_shape::<DateRangeFilter, chrono::NaiveDateTime>();
            assert_declared::<DateRangeFilterAdapter>();
            assert_shape_for::<DateRangeFilterAdapter, BusinessDate>();
            assert_shape_for::<DateRangeFilterAdapter, Option<BusinessDate>>();
            assert_table_filter_shape::<DateRangeFilterAdapter, BusinessDate>();
            assert_table_filter_shape::<DateRangeFilterAdapter, Option<BusinessDate>>();
        }
    }

    #[test]
    fn text_filter_adapter_matches_custom_fields_and_options() {
        let active = TextValue::from("ACCT");
        let inactive = TextValue::default();

        assert!(<TextFilterAdapter as GpuiTableFilterShapeFor<
            AccountCode,
        >>::matches_field(
            &AccountCode("sales-acct".to_string()), &active,
        ));
        assert!(<TextFilterAdapter as GpuiTableFilterShapeFor<
            Option<AccountCode>,
        >>::matches_field(
            &Some(AccountCode("tax-acct".to_string())),
            &active,
        ));
        assert!(!<TextFilterAdapter as GpuiTableFilterShapeFor<
            Option<AccountCode>,
        >>::matches_field(&None, &active,));
        assert!(<TextFilterAdapter as GpuiTableFilterShapeFor<
            Option<AccountCode>,
        >>::matches_field(&None, &inactive,));

        assert_eq!(
            <TextFilterAdapter as GpuiTableFilterShapeFor<AccountCode>>::filter_type(),
            gpui_table_core::filter::FilterType::Text
        );
        assert_eq!(
            <TextFilter as GpuiTableFilterShapeFor<String>>::filter_type(),
            gpui_table_core::filter::FilterType::Text
        );
        assert!(
            <TextFilter as GpuiTableFilterShapeFor<String>>::matches_field(
                &"sales-acct".into(),
                &active,
            )
        );
        assert!(
            !<TextFilter as GpuiTableFilterShapeFor<Option<String>>>::matches_field(&None, &active,)
        );
        assert!(
            <TextFilter as GpuiTableFilterShapeFor<Option<String>>>::matches_field(
                &None, &inactive,
            )
        );
        assert_eq!(
            <TextFilter as GpuiTableFilterShape>::wrap_value("value".into()),
            TextValue::from("value")
        );
    }

    #[test]
    fn faceted_shape_matching_covers_scalar_optional_and_collection_fields() {
        use gpui_table_core::filter::FilterType;

        let active = FacetedValue::from(HashSet::from([Status::Active]));
        let inactive = FacetedValue::default();

        assert!(matches!(
            <FacetedFilter<Status> as GpuiTableFilterShapeFor<Status>>::filter_type(),
            FilterType::Faceted(options) if options.is_empty()
        ));
        assert!(
            <FacetedFilter<Status> as GpuiTableFilterShapeFor<Status>>::matches_field(
                &Status::Active,
                &active
            )
        );
        assert!(!<FacetedFilter<Status> as GpuiTableFilterShapeFor<
            Status,
        >>::matches_field(&Status::Pending, &active));
        assert!(!<FacetedFilter<Status> as GpuiTableFilterShapeFor<
            Option<Status>,
        >>::matches_field(&None, &active));
        assert!(<FacetedFilter<Status> as GpuiTableFilterShapeFor<
            Option<Status>,
        >>::matches_field(&None, &inactive));
        assert!(<FacetedFilter<Status> as GpuiTableFilterShapeFor<
            Vec<Status>,
        >>::matches_field(
            &vec![Status::Pending, Status::Active], &active
        ));
        assert!(!<FacetedFilter<Status> as GpuiTableFilterShapeFor<
            Vec<Status>,
        >>::matches_field(&vec![Status::Pending], &active));
        assert!(<FacetedFilter<Status> as GpuiTableFilterShapeFor<
            Option<Vec<Status>>,
        >>::matches_field(
            &Some(vec![Status::Active]), &active
        ));
        assert!(!<FacetedFilter<Status> as GpuiTableFilterShapeFor<
            Option<Vec<Status>>,
        >>::matches_field(&None, &active));
        assert!(<FacetedFilter<Status> as GpuiTableFilterShapeFor<
            Option<Vec<Status>>,
        >>::matches_field(&None, &inactive));
        assert_eq!(
            <FacetedFilter<Status> as GpuiTableFilterShape>::wrap_value(HashSet::from([
                Status::Pending
            ])),
            FacetedValue::from(HashSet::from([Status::Pending]))
        );
    }

    #[cfg(feature = "rust_decimal")]
    #[test]
    fn number_range_filter_adapter_matches_custom_fields_and_options() {
        use super::{NumberRangeFilter, NumberRangeFilterAdapter, NumberRangeFilterField as _};
        use rust_decimal::Decimal;

        let active = RangeValue::from((Some(Decimal::new(10, 0)), Some(Decimal::new(20, 0))));
        let inactive = RangeValue::default();

        assert!(<NumberRangeFilterAdapter as GpuiTableFilterShapeFor<
            Amount,
        >>::matches_field(
            &Amount(Decimal::new(15, 0)), &active,
        ));
        assert!(!<NumberRangeFilterAdapter as GpuiTableFilterShapeFor<
            Amount,
        >>::matches_field(
            &Amount(Decimal::new(25, 0)), &active,
        ));
        assert!(<NumberRangeFilterAdapter as GpuiTableFilterShapeFor<
            Option<Amount>,
        >>::matches_field(
            &Some(Amount(Decimal::new(15, 0))), &active,
        ));
        assert!(!<NumberRangeFilterAdapter as GpuiTableFilterShapeFor<
            Option<Amount>,
        >>::matches_field(&None, &active,));
        assert!(<NumberRangeFilterAdapter as GpuiTableFilterShapeFor<
            Option<Amount>,
        >>::matches_field(&None, &inactive,));

        assert_eq!(
            <NumberRangeFilterAdapter as GpuiTableFilterShapeFor<Amount>>::filter_type(),
            gpui_table_core::filter::FilterType::NumberRange
        );
        assert_eq!(
            <NumberRangeFilter as GpuiTableFilterShape>::wrap_value((
                Some(Decimal::from(10)),
                Some(Decimal::from(20)),
            )),
            active
        );

        macro_rules! assert_number_shape {
            ($ty:ty, $inside:expr, $outside:expr) => {{
                let inside: $ty = $inside;
                let outside: $ty = $outside;
                assert_eq!(
                    <NumberRangeFilter as GpuiTableFilterShapeFor<$ty>>::filter_type(),
                    gpui_table_core::filter::FilterType::NumberRange
                );
                assert!(
                    <NumberRangeFilter as GpuiTableFilterShapeFor<$ty>>::matches_field(
                        &inside, &active
                    )
                );
                assert!(
                    !<NumberRangeFilter as GpuiTableFilterShapeFor<$ty>>::matches_field(
                        &outside, &active
                    )
                );
                assert!(<NumberRangeFilter as GpuiTableFilterShapeFor<
                    Option<$ty>,
                >>::matches_field(&None, &inactive));
                assert!(!<NumberRangeFilter as GpuiTableFilterShapeFor<
                    Option<$ty>,
                >>::matches_field(&None, &active));
                assert_eq!(inside.to_filter_decimal(), Decimal::from(15));
            }};
        }

        assert_number_shape!(i8, 15_i8, 25_i8);
        assert_number_shape!(i16, 15_i16, 25_i16);
        assert_number_shape!(i32, 15_i32, 25_i32);
        assert_number_shape!(i64, 15_i64, 25_i64);
        assert_number_shape!(isize, 15_isize, 25_isize);
        assert_number_shape!(u8, 15_u8, 25_u8);
        assert_number_shape!(u16, 15_u16, 25_u16);
        assert_number_shape!(u32, 15_u32, 25_u32);
        assert_number_shape!(u64, 15_u64, 25_u64);
        assert_number_shape!(usize, 15_usize, 25_usize);
        assert_number_shape!(f32, 15.0_f32, 25.0_f32);
        assert_number_shape!(f64, 15.0_f64, 25.0_f64);
        assert_number_shape!(Decimal, Decimal::from(15), Decimal::from(25));

        #[cfg(feature = "spacetimedb")]
        {
            use spacetimedb_lib::{TimeDuration, Timestamp};
            assert_number_shape!(
                Timestamp,
                Timestamp::from_micros_since_unix_epoch(15),
                Timestamp::from_micros_since_unix_epoch(25)
            );
            assert_number_shape!(
                TimeDuration,
                TimeDuration::from_micros(15),
                TimeDuration::from_micros(25)
            );
        }
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn date_range_filter_adapter_matches_custom_fields_and_options() {
        use super::{DateRangeFilter, DateRangeFilterAdapter, DateRangeFilterField as _};
        use chrono::{DateTime, NaiveDate, Utc};

        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let inside = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let outside = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        let active = RangeValue::from((Some(start), Some(end)));
        let inactive = RangeValue::default();

        assert!(<DateRangeFilterAdapter as GpuiTableFilterShapeFor<
            BusinessDate,
        >>::matches_field(&BusinessDate(inside), &active,));
        assert!(!<DateRangeFilterAdapter as GpuiTableFilterShapeFor<
            BusinessDate,
        >>::matches_field(&BusinessDate(outside), &active,));
        assert!(<DateRangeFilterAdapter as GpuiTableFilterShapeFor<
            Option<BusinessDate>,
        >>::matches_field(
            &Some(BusinessDate(inside)), &active,
        ));
        assert!(!<DateRangeFilterAdapter as GpuiTableFilterShapeFor<
            Option<BusinessDate>,
        >>::matches_field(&None, &active,));
        assert!(<DateRangeFilterAdapter as GpuiTableFilterShapeFor<
            Option<BusinessDate>,
        >>::matches_field(&None, &inactive,));

        assert_eq!(
            <DateRangeFilterAdapter as GpuiTableFilterShapeFor<BusinessDate>>::filter_type(),
            gpui_table_core::filter::FilterType::DateRange
        );
        assert_eq!(
            <DateRangeFilter as GpuiTableFilterShape>::wrap_value((Some(start), Some(end))),
            active
        );

        let inside_datetime = inside.and_hms_opt(12, 0, 0).unwrap();
        let outside_datetime = outside.and_hms_opt(12, 0, 0).unwrap();
        assert_eq!(inside.to_filter_naive_date(), inside);
        assert_eq!(inside_datetime.to_filter_naive_date(), inside);
        assert!(<DateRangeFilter as GpuiTableFilterShapeFor<
            chrono::NaiveDate,
        >>::matches_field(&inside, &active));
        assert!(!<DateRangeFilter as GpuiTableFilterShapeFor<
            chrono::NaiveDate,
        >>::matches_field(&outside, &active));
        assert!(<DateRangeFilter as GpuiTableFilterShapeFor<
            Option<chrono::NaiveDate>,
        >>::matches_field(&None, &inactive));
        assert!(!<DateRangeFilter as GpuiTableFilterShapeFor<
            Option<chrono::NaiveDate>,
        >>::matches_field(&None, &active));
        assert!(<DateRangeFilter as GpuiTableFilterShapeFor<
            chrono::NaiveDateTime,
        >>::matches_field(&inside_datetime, &active));
        assert!(!<DateRangeFilter as GpuiTableFilterShapeFor<
            chrono::NaiveDateTime,
        >>::matches_field(&outside_datetime, &active));
        assert!(<DateRangeFilter as GpuiTableFilterShapeFor<
            Option<chrono::NaiveDateTime>,
        >>::matches_field(&None, &inactive));

        let inside_zoned = DateTime::<Utc>::from_naive_utc_and_offset(inside_datetime, Utc);
        let outside_zoned = DateTime::<Utc>::from_naive_utc_and_offset(outside_datetime, Utc);
        assert_eq!(inside_zoned.to_filter_naive_date(), inside);
        assert!(<DateRangeFilter as GpuiTableFilterShapeFor<
            DateTime<Utc>,
        >>::matches_field(&inside_zoned, &active));
        assert!(!<DateRangeFilter as GpuiTableFilterShapeFor<
            DateTime<Utc>,
        >>::matches_field(&outside_zoned, &active));
        assert!(<DateRangeFilter as GpuiTableFilterShapeFor<
            Option<DateTime<Utc>>,
        >>::matches_field(&None, &inactive));
        assert!(!<DateRangeFilter as GpuiTableFilterShapeFor<
            Option<DateTime<Utc>>,
        >>::matches_field(&None, &active));

        #[cfg(feature = "spacetimedb")]
        {
            use spacetimedb_lib::Timestamp;
            let inside_timestamp = Timestamp::from_micros_since_unix_epoch(
                inside_datetime.and_utc().timestamp_micros(),
            );
            let outside_timestamp = Timestamp::from_micros_since_unix_epoch(
                outside_datetime.and_utc().timestamp_micros(),
            );
            assert_eq!(inside_timestamp.to_filter_naive_date(), inside);
            assert!(
                <DateRangeFilter as GpuiTableFilterShapeFor<Timestamp>>::matches_field(
                    &inside_timestamp,
                    &active
                )
            );
            assert!(
                !<DateRangeFilter as GpuiTableFilterShapeFor<Timestamp>>::matches_field(
                    &outside_timestamp,
                    &active
                )
            );
            assert!(<DateRangeFilter as GpuiTableFilterShapeFor<
                Option<Timestamp>,
            >>::matches_field(&None, &inactive));
        }
    }
}
