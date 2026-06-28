#[cfg(feature = "chrono")]
use crate::DateRangeFilter;
#[cfg(feature = "rust_decimal")]
use crate::NumberRangeFilter;
use crate::{FacetedFilter, TextFilter};
use gpui::{App, Entity, Window};
use gpui_table_core::filter::{FacetedValue, FilterType, TextValue};
use gpui_table_runtime::shape::{
    ComponentShapeFor, ComponentShapeMetadata, DeclaredComponentShape,
    DeclaredGpuiTableFilterShape, GpuiTableFilterShape, GpuiTableFilterShapeFor,
};
use gpui_table_schema::registry::RegistryFilterType;
use std::collections::HashSet;

/// Adapter shape for text filters over application-owned field types.
///
/// Use this when a table field is a transparent or domain-specific value type
/// that should be matched by its text representation while reusing the built-in
/// [`TextFilter`] component and MCP schema.
pub struct TextFilterAdapter;

/// Field conversion contract used by [`TextFilterAdapter`].
pub trait TextFilterField {
    /// Converts the field value into the text matched by [`TextFilter`].
    fn to_filter_text(&self) -> String;
}

impl TextFilterField for String {
    fn to_filter_text(&self) -> String {
        self.clone()
    }
}

#[cfg(feature = "rust_decimal")]
/// Adapter shape for decimal range filters over application-owned field types.
///
/// Use this when a table field is a transparent or domain-specific value type
/// that should be matched by a decimal value while reusing the built-in
/// [`NumberRangeFilter`] component and MCP schema.
pub struct NumberRangeFilterAdapter;

#[cfg(feature = "rust_decimal")]
/// Field conversion contract used by [`NumberRangeFilterAdapter`].
pub trait NumberRangeFilterField {
    /// Converts the field value into the decimal matched by [`NumberRangeFilter`].
    fn to_filter_decimal(&self) -> rust_decimal::Decimal;
}

#[cfg(feature = "rust_decimal")]
macro_rules! impl_number_range_filter_field {
    ($($ty:ty),* $(,)?) => {
        $(
            impl NumberRangeFilterField for $ty {
                fn to_filter_decimal(&self) -> rust_decimal::Decimal {
                    gpui_table_core::filter::ToDecimal::to_decimal(self)
                }
            }
        )*
    };
}

#[cfg(feature = "rust_decimal")]
impl_number_range_filter_field!(
    f32,
    f64,
    i8,
    i16,
    i32,
    i64,
    isize,
    rust_decimal::Decimal,
    u8,
    u16,
    u32,
    u64,
    usize,
);

#[cfg(all(feature = "rust_decimal", feature = "spacetimedb"))]
impl_number_range_filter_field!(spacetimedb_lib::Timestamp, spacetimedb_lib::TimeDuration);

#[cfg(feature = "chrono")]
/// Adapter shape for date range filters over application-owned field types.
///
/// Use this when a table field is a transparent or domain-specific value type
/// that should be matched by a date value while reusing the built-in
/// [`DateRangeFilter`] component and MCP schema.
pub struct DateRangeFilterAdapter;

#[cfg(feature = "chrono")]
/// Field conversion contract used by [`DateRangeFilterAdapter`].
pub trait DateRangeFilterField {
    /// Converts the field value into the date matched by [`DateRangeFilter`].
    fn to_filter_naive_date(&self) -> chrono::NaiveDate;
}

#[cfg(feature = "chrono")]
macro_rules! impl_date_range_filter_field {
    ($($ty:ty),* $(,)?) => {
        $(
            impl DateRangeFilterField for $ty {
                fn to_filter_naive_date(&self) -> chrono::NaiveDate {
                    gpui_table_core::filter::ToNaiveDate::to_naive_date(self)
                }
            }
        )*
    };
}

#[cfg(feature = "chrono")]
impl_date_range_filter_field!(chrono::NaiveDate, chrono::NaiveDateTime);

#[cfg(feature = "chrono")]
impl<Tz> DateRangeFilterField for chrono::DateTime<Tz>
where
    Tz: chrono::TimeZone,
{
    fn to_filter_naive_date(&self) -> chrono::NaiveDate {
        self.date_naive()
    }
}

#[cfg(all(feature = "chrono", feature = "spacetimedb"))]
impl_date_range_filter_field!(spacetimedb_lib::Timestamp);

macro_rules! delegate_filter_shape {
    ($shape:ty, $base:ty) => {
        impl ComponentShapeMetadata for $shape {
            const MCP_INPUT: gpui_table_runtime::shape::McpInput =
                <$base as ComponentShapeMetadata>::MCP_INPUT;
        }

        impl DeclaredComponentShape for $shape {}

        impl GpuiTableFilterShape for $shape {
            type Component = <$base as GpuiTableFilterShape>::Component;
            type FilterValue = <$base as GpuiTableFilterShape>::FilterValue;
            type RawValue = <$base as GpuiTableFilterShape>::RawValue;

            const FILTER_TYPE: RegistryFilterType = <$base as GpuiTableFilterShape>::FILTER_TYPE;

            fn new_for(
                title: impl Fn(&App) -> String + 'static,
                value: Self::RawValue,
                on_change: impl Fn(Self::RawValue, &mut Window, &mut App) + 'static,
                cx: &mut App,
            ) -> Entity<Self::Component> {
                <$base as GpuiTableFilterShape>::new_for(title, value, on_change, cx)
            }

            fn read_value(entity: &Entity<Self::Component>, cx: &App) -> Self::RawValue {
                <$base as GpuiTableFilterShape>::read_value(entity, cx)
            }

            fn wrap_value(value: Self::RawValue) -> Self::FilterValue {
                <$base as GpuiTableFilterShape>::wrap_value(value)
            }

            fn reset_silent(entity: &Entity<Self::Component>, window: &mut Window, cx: &mut App) {
                <$base as GpuiTableFilterShape>::reset_silent(entity, window, cx);
            }
        }

        impl DeclaredGpuiTableFilterShape for $shape {}
    };
}

delegate_filter_shape!(TextFilterAdapter, TextFilter);

impl<T> ComponentShapeFor<T> for TextFilterAdapter where T: TextFilterField {}

impl<T> ComponentShapeFor<Option<T>> for TextFilterAdapter where T: TextFilterField {}

impl<T> GpuiTableFilterShapeFor<T> for TextFilterAdapter
where
    T: TextFilterField,
{
    fn filter_type() -> FilterType {
        FilterType::Text
    }

    fn matches_field(field: &T, value: &Self::FilterValue) -> bool {
        value.matches(&field.to_filter_text())
    }
}

impl<T> GpuiTableFilterShapeFor<Option<T>> for TextFilterAdapter
where
    T: TextFilterField,
{
    fn filter_type() -> FilterType {
        FilterType::Text
    }

    fn matches_field(field: &Option<T>, value: &Self::FilterValue) -> bool {
        !value.is_active()
            || field
                .as_ref()
                .is_some_and(|field| value.matches(&field.to_filter_text()))
    }
}

#[cfg(feature = "rust_decimal")]
delegate_filter_shape!(NumberRangeFilterAdapter, NumberRangeFilter);

#[cfg(feature = "rust_decimal")]
impl<T> ComponentShapeFor<T> for NumberRangeFilterAdapter where T: NumberRangeFilterField {}

#[cfg(feature = "rust_decimal")]
impl<T> ComponentShapeFor<Option<T>> for NumberRangeFilterAdapter where T: NumberRangeFilterField {}

#[cfg(feature = "rust_decimal")]
impl<T> GpuiTableFilterShapeFor<T> for NumberRangeFilterAdapter
where
    T: NumberRangeFilterField,
{
    fn filter_type() -> FilterType {
        FilterType::NumberRange
    }

    fn matches_field(field: &T, value: &Self::FilterValue) -> bool {
        value.matches(&field.to_filter_decimal())
    }
}

#[cfg(feature = "rust_decimal")]
impl<T> GpuiTableFilterShapeFor<Option<T>> for NumberRangeFilterAdapter
where
    T: NumberRangeFilterField,
{
    fn filter_type() -> FilterType {
        FilterType::NumberRange
    }

    fn matches_field(field: &Option<T>, value: &Self::FilterValue) -> bool {
        !value.is_active()
            || field
                .as_ref()
                .is_some_and(|field| value.matches(&field.to_filter_decimal()))
    }
}

#[cfg(feature = "chrono")]
delegate_filter_shape!(DateRangeFilterAdapter, DateRangeFilter);

#[cfg(feature = "chrono")]
impl<T> ComponentShapeFor<T> for DateRangeFilterAdapter where T: DateRangeFilterField {}

#[cfg(feature = "chrono")]
impl<T> ComponentShapeFor<Option<T>> for DateRangeFilterAdapter where T: DateRangeFilterField {}

#[cfg(feature = "chrono")]
impl<T> GpuiTableFilterShapeFor<T> for DateRangeFilterAdapter
where
    T: DateRangeFilterField,
{
    fn filter_type() -> FilterType {
        FilterType::DateRange
    }

    fn matches_field(field: &T, value: &Self::FilterValue) -> bool {
        value.matches(&field.to_filter_naive_date())
    }
}

#[cfg(feature = "chrono")]
impl<T> GpuiTableFilterShapeFor<Option<T>> for DateRangeFilterAdapter
where
    T: DateRangeFilterField,
{
    fn filter_type() -> FilterType {
        FilterType::DateRange
    }

    fn matches_field(field: &Option<T>, value: &Self::FilterValue) -> bool {
        !value.is_active()
            || field
                .as_ref()
                .is_some_and(|field| value.matches(&field.to_filter_naive_date()))
    }
}

impl GpuiTableFilterShape for TextFilter {
    type Component = TextFilter;
    type RawValue = String;
    type FilterValue = TextValue;

    const FILTER_TYPE: RegistryFilterType = RegistryFilterType::Text;

    fn new_for(
        title: impl Fn(&App) -> String + 'static,
        value: Self::RawValue,
        on_change: impl Fn(Self::RawValue, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self::Component> {
        TextFilter::new_for(title, value, on_change, cx)
    }

    fn read_value(entity: &Entity<Self::Component>, cx: &App) -> Self::RawValue {
        entity.read(cx).value().to_string()
    }

    fn wrap_value(value: Self::RawValue) -> Self::FilterValue {
        TextValue::from(value)
    }

    fn reset_silent(entity: &Entity<Self::Component>, window: &mut Window, cx: &mut App) {
        entity.update(cx, |filter, cx| filter.reset_silent(window, cx));
    }
}

impl DeclaredGpuiTableFilterShape for TextFilter {}

impl GpuiTableFilterShapeFor<String> for TextFilter {
    fn filter_type() -> FilterType {
        FilterType::Text
    }

    fn matches_field(field: &String, value: &Self::FilterValue) -> bool {
        value.matches(field.as_ref())
    }
}

impl GpuiTableFilterShapeFor<Option<String>> for TextFilter {
    fn filter_type() -> FilterType {
        FilterType::Text
    }

    fn matches_field(field: &Option<String>, value: &Self::FilterValue) -> bool {
        !value.is_active()
            || field
                .as_ref()
                .is_some_and(|field| value.matches(field.as_ref()))
    }
}

impl<T> GpuiTableFilterShape for FacetedFilter<T>
where
    T: gpui_table_core::filter::Filterable,
{
    type Component = FacetedFilter<T>;
    type RawValue = HashSet<T>;
    type FilterValue = FacetedValue<T>;

    const FILTER_TYPE: RegistryFilterType = RegistryFilterType::Faceted;

    fn new_for(
        title: impl Fn(&App) -> String + 'static,
        value: Self::RawValue,
        on_change: impl Fn(Self::RawValue, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self::Component> {
        FacetedFilter::<T>::new_for(title, value, on_change, cx)
    }

    fn read_value(entity: &Entity<Self::Component>, cx: &App) -> Self::RawValue {
        entity.read(cx).value().clone()
    }

    fn wrap_value(value: Self::RawValue) -> Self::FilterValue {
        FacetedValue::from(value)
    }

    fn reset_silent(entity: &Entity<Self::Component>, window: &mut Window, cx: &mut App) {
        entity.update(cx, |filter, cx| filter.reset_silent(window, cx));
    }
}

impl<T> DeclaredGpuiTableFilterShape for FacetedFilter<T> where
    T: gpui_table_core::filter::Filterable
{
}

impl<T> GpuiTableFilterShapeFor<T> for FacetedFilter<T>
where
    T: gpui_table_core::filter::Filterable,
{
    fn filter_type() -> FilterType {
        FilterType::Faceted(T::options())
    }

    fn matches_field(field: &T, value: &Self::FilterValue) -> bool {
        value.matches(field)
    }
}

impl<T> GpuiTableFilterShapeFor<Option<T>> for FacetedFilter<T>
where
    T: gpui_table_core::filter::Filterable,
{
    fn filter_type() -> FilterType {
        FilterType::Faceted(T::options())
    }

    fn matches_field(field: &Option<T>, value: &Self::FilterValue) -> bool {
        !value.is_active() || field.as_ref().is_some_and(|field| value.matches(field))
    }
}

impl<T> GpuiTableFilterShapeFor<Vec<T>> for FacetedFilter<T>
where
    T: gpui_table_core::filter::Filterable,
{
    fn filter_type() -> FilterType {
        FilterType::Faceted(T::options())
    }

    fn matches_field(field: &Vec<T>, value: &Self::FilterValue) -> bool {
        !value.is_active() || field.iter().any(|field| value.matches(field))
    }
}

impl<T> GpuiTableFilterShapeFor<Option<Vec<T>>> for FacetedFilter<T>
where
    T: gpui_table_core::filter::Filterable,
{
    fn filter_type() -> FilterType {
        FilterType::Faceted(T::options())
    }

    fn matches_field(field: &Option<Vec<T>>, value: &Self::FilterValue) -> bool {
        !value.is_active()
            || field
                .as_ref()
                .is_some_and(|field| field.iter().any(|field| value.matches(field)))
    }
}

#[cfg(feature = "rust_decimal")]
impl GpuiTableFilterShape for NumberRangeFilter {
    type Component = NumberRangeFilter;
    type RawValue = (Option<rust_decimal::Decimal>, Option<rust_decimal::Decimal>);
    type FilterValue = gpui_table_core::filter::RangeValue<rust_decimal::Decimal>;

    const FILTER_TYPE: RegistryFilterType = RegistryFilterType::NumberRange;

    fn new_for(
        title: impl Fn(&App) -> String + 'static,
        value: Self::RawValue,
        on_change: impl Fn(Self::RawValue, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self::Component> {
        NumberRangeFilter::new_for(title, value, on_change, cx)
    }

    fn read_value(entity: &Entity<Self::Component>, cx: &App) -> Self::RawValue {
        entity.read(cx).value()
    }

    fn wrap_value(value: Self::RawValue) -> Self::FilterValue {
        gpui_table_core::filter::RangeValue::from(value)
    }

    fn reset_silent(entity: &Entity<Self::Component>, window: &mut Window, cx: &mut App) {
        entity.update(cx, |filter, cx| filter.reset_silent(window, cx));
    }
}

#[cfg(feature = "rust_decimal")]
impl DeclaredGpuiTableFilterShape for NumberRangeFilter {}

#[cfg(feature = "rust_decimal")]
macro_rules! impl_number_range_shape_for {
    ($($ty:ty),* $(,)?) => {
        $(
            impl GpuiTableFilterShapeFor<$ty> for NumberRangeFilter {
                fn filter_type() -> FilterType {
                    FilterType::NumberRange
                }

                fn matches_field(field: &$ty, value: &Self::FilterValue) -> bool {
                    value.matches(&gpui_table_core::filter::ToDecimal::to_decimal(field))
                }
            }

            impl GpuiTableFilterShapeFor<Option<$ty>> for NumberRangeFilter {
                fn filter_type() -> FilterType {
                    FilterType::NumberRange
                }

                fn matches_field(field: &Option<$ty>, value: &Self::FilterValue) -> bool {
                    !value.is_active()
                        || field
                            .as_ref()
                            .is_some_and(|field| value.matches(&gpui_table_core::filter::ToDecimal::to_decimal(field)))
                }
            }
        )*
    };
}

#[cfg(feature = "rust_decimal")]
impl_number_range_shape_for!(
    f32,
    f64,
    i8,
    i16,
    i32,
    i64,
    isize,
    rust_decimal::Decimal,
    u8,
    u16,
    u32,
    u64,
    usize,
);

#[cfg(all(feature = "rust_decimal", feature = "spacetimedb"))]
impl_number_range_shape_for!(spacetimedb_lib::Timestamp, spacetimedb_lib::TimeDuration);

#[cfg(feature = "chrono")]
impl GpuiTableFilterShape for DateRangeFilter {
    type Component = DateRangeFilter;
    type RawValue = (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>);
    type FilterValue = gpui_table_core::filter::RangeValue<chrono::NaiveDate>;

    const FILTER_TYPE: RegistryFilterType = RegistryFilterType::DateRange;

    fn new_for(
        title: impl Fn(&App) -> String + 'static,
        value: Self::RawValue,
        on_change: impl Fn(Self::RawValue, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self::Component> {
        DateRangeFilter::new_for(title, value, on_change, cx)
    }

    fn read_value(entity: &Entity<Self::Component>, cx: &App) -> Self::RawValue {
        entity.read(cx).value()
    }

    fn wrap_value(value: Self::RawValue) -> Self::FilterValue {
        gpui_table_core::filter::RangeValue::from(value)
    }

    fn reset_silent(entity: &Entity<Self::Component>, window: &mut Window, cx: &mut App) {
        entity.update(cx, |filter, cx| filter.reset_silent(window, cx));
    }
}

#[cfg(feature = "chrono")]
impl DeclaredGpuiTableFilterShape for DateRangeFilter {}

#[cfg(feature = "chrono")]
macro_rules! impl_date_range_shape_for {
    ($($ty:ty),* $(,)?) => {
        $(
            impl GpuiTableFilterShapeFor<$ty> for DateRangeFilter {
                fn filter_type() -> FilterType {
                    FilterType::DateRange
                }

                fn matches_field(field: &$ty, value: &Self::FilterValue) -> bool {
                    value.matches(&gpui_table_core::filter::ToNaiveDate::to_naive_date(field))
                }
            }

            impl GpuiTableFilterShapeFor<Option<$ty>> for DateRangeFilter {
                fn filter_type() -> FilterType {
                    FilterType::DateRange
                }

                fn matches_field(field: &Option<$ty>, value: &Self::FilterValue) -> bool {
                    !value.is_active()
                        || field
                            .as_ref()
                            .is_some_and(|field| value.matches(&gpui_table_core::filter::ToNaiveDate::to_naive_date(field)))
                }
            }
        )*
    };
}

#[cfg(feature = "chrono")]
impl_date_range_shape_for!(chrono::NaiveDate, chrono::NaiveDateTime);

#[cfg(feature = "chrono")]
impl<Tz> GpuiTableFilterShapeFor<chrono::DateTime<Tz>> for DateRangeFilter
where
    Tz: chrono::TimeZone,
{
    fn filter_type() -> FilterType {
        FilterType::DateRange
    }

    fn matches_field(field: &chrono::DateTime<Tz>, value: &Self::FilterValue) -> bool {
        value.matches(&field.date_naive())
    }
}

#[cfg(feature = "chrono")]
impl<Tz> GpuiTableFilterShapeFor<Option<chrono::DateTime<Tz>>> for DateRangeFilter
where
    Tz: chrono::TimeZone,
{
    fn filter_type() -> FilterType {
        FilterType::DateRange
    }

    fn matches_field(field: &Option<chrono::DateTime<Tz>>, value: &Self::FilterValue) -> bool {
        !value.is_active()
            || field
                .as_ref()
                .is_some_and(|field| value.matches(&field.date_naive()))
    }
}

#[cfg(all(feature = "chrono", feature = "spacetimedb"))]
impl_date_range_shape_for!(spacetimedb_lib::Timestamp);
