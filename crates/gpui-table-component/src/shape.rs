#[cfg(feature = "chrono")]
use crate::DateRangeFilter;
use crate::{FacetedFilter, FacetedFilterExt as _, TextFilter, TextFilterExt as _};
#[cfg(feature = "rust_decimal")]
use crate::{NumberRangeFilter, NumberRangeFilterExt as _};
use gpui::{App, Entity, Window};
use gpui_table_core::filter::{FacetedValue, FilterType, Filterable, TextValue};
use gpui_table_runtime::shape::{
    ComponentShapeFor, ComponentShapeMetadata, DeclaredComponentShape,
    DeclaredGpuiTableFilterShape, GpuiTableFilterShape, GpuiTableFilterShapeBuilder,
    GpuiTableFilterShapeFor,
};
use gpui_table_schema::registry::RegistryFilterType;
use std::collections::HashSet;
use std::marker::PhantomData;

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

/// Configured construction options for [`TextFilter`].
///
/// Use `TextFilter.matching_regex(...)`, `TextFilter.numeric_only()`,
/// `TextFilter.alphabetic_only()`, or `TextFilter.alphanumeric_only()` in
/// `#[gpui_table(filter(...))]` when a generated table field should build the
/// text filter with the matching input validator enabled. Regex patterns are
/// matched against the complete candidate input value.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextFilterArgs {
    validation: TextFilterValidation,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum TextFilterValidation {
    #[default]
    None,
    Alphabetic,
    Numeric,
    Alphanumeric,
    MatchingRegex(&'static str),
}

impl TextFilter {
    pub const fn matching_regex(pattern: &'static str) -> TextFilterArgs {
        TextFilterArgs {
            validation: TextFilterValidation::MatchingRegex(pattern),
        }
    }

    pub const fn alphabetic_only() -> TextFilterArgs {
        TextFilterArgs {
            validation: TextFilterValidation::Alphabetic,
        }
    }

    pub const fn numeric_only() -> TextFilterArgs {
        TextFilterArgs {
            validation: TextFilterValidation::Numeric,
        }
    }

    pub const fn alphanumeric_only() -> TextFilterArgs {
        TextFilterArgs {
            validation: TextFilterValidation::Alphanumeric,
        }
    }
}

impl GpuiTableFilterShapeBuilder<TextFilter> for TextFilterArgs {
    fn build(
        self,
        title: impl Fn(&App) -> String + 'static,
        value: <TextFilter as GpuiTableFilterShape>::RawValue,
        on_change: impl Fn(<TextFilter as GpuiTableFilterShape>::RawValue, &mut Window, &mut App)
        + 'static,
        cx: &mut App,
    ) -> Entity<<TextFilter as GpuiTableFilterShape>::Component> {
        let entity = TextFilter::new_for(title, value, on_change, cx);
        match self.validation {
            TextFilterValidation::None => entity,
            TextFilterValidation::Alphabetic => entity.alphabetic_only(cx),
            TextFilterValidation::Numeric => entity.numeric_only(cx),
            TextFilterValidation::Alphanumeric => entity.alphanumeric_only(cx),
            TextFilterValidation::MatchingRegex(pattern) => entity.matching_regex(pattern, cx),
        }
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

#[cfg(feature = "rust_decimal")]
/// Configured construction options for [`NumberRangeFilter`].
///
/// Use `NumberRangeFilter.range(...)`, `NumberRangeFilter.step(...)`, or a
/// chained expression such as `NumberRangeFilter.range(...).step(...)` in
/// `#[gpui_table(filter(...))]` when a generated table field should build the
/// number range filter with explicit slider bounds or step size.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NumberRangeFilterArgs {
    range: Option<(rust_decimal::Decimal, rust_decimal::Decimal)>,
    step: Option<rust_decimal::Decimal>,
}

#[cfg(feature = "rust_decimal")]
impl NumberRangeFilterArgs {
    pub fn range(mut self, min: rust_decimal::Decimal, max: rust_decimal::Decimal) -> Self {
        self.range = Some((min, max));
        self
    }

    pub fn step(mut self, step: rust_decimal::Decimal) -> Self {
        self.step = Some(step);
        self
    }
}

#[cfg(feature = "rust_decimal")]
impl NumberRangeFilter {
    pub fn range(min: rust_decimal::Decimal, max: rust_decimal::Decimal) -> NumberRangeFilterArgs {
        NumberRangeFilterArgs::default().range(min, max)
    }

    pub fn step(step: rust_decimal::Decimal) -> NumberRangeFilterArgs {
        NumberRangeFilterArgs::default().step(step)
    }
}

#[cfg(feature = "rust_decimal")]
impl GpuiTableFilterShapeBuilder<NumberRangeFilter> for NumberRangeFilterArgs {
    fn build(
        self,
        title: impl Fn(&App) -> String + 'static,
        value: <NumberRangeFilter as GpuiTableFilterShape>::RawValue,
        on_change: impl Fn(<NumberRangeFilter as GpuiTableFilterShape>::RawValue, &mut Window, &mut App)
        + 'static,
        cx: &mut App,
    ) -> Entity<<NumberRangeFilter as GpuiTableFilterShape>::Component> {
        let mut entity = NumberRangeFilter::new_for(title, value, on_change, cx);
        if let Some((min, max)) = self.range {
            entity = entity.range(min, max, cx);
        }
        if let Some(step) = self.step {
            entity = entity.step(step, cx);
        }
        entity
    }
}

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

/// Configured construction options for [`FacetedFilter`].
///
/// Use `FacetedFilter::<T>.searchable(true)` in
/// `#[gpui_table(filter(...))]` when a generated table field should build the
/// faceted filter with its search input visible.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FacetedFilterArgs<T> {
    searchable: bool,
    _marker: PhantomData<fn() -> T>,
}

impl<T> FacetedFilterArgs<T> {
    pub const fn searchable(searchable: bool) -> Self {
        Self {
            searchable,
            _marker: PhantomData,
        }
    }
}

impl<T> Default for FacetedFilterArgs<T> {
    fn default() -> Self {
        Self::searchable(false)
    }
}

impl<T> FacetedFilter<T>
where
    T: gpui_table_core::filter::FilterValue,
{
    pub const fn searchable(searchable: bool) -> FacetedFilterArgs<T> {
        FacetedFilterArgs::searchable(searchable)
    }
}

impl<T> GpuiTableFilterShapeBuilder<FacetedFilter<T>> for FacetedFilterArgs<T>
where
    T: Filterable,
{
    fn build(
        self,
        title: impl Fn(&App) -> String + 'static,
        value: <FacetedFilter<T> as GpuiTableFilterShape>::RawValue,
        on_change: impl Fn(<FacetedFilter<T> as GpuiTableFilterShape>::RawValue, &mut Window, &mut App)
        + 'static,
        cx: &mut App,
    ) -> Entity<<FacetedFilter<T> as GpuiTableFilterShape>::Component> {
        let entity = FacetedFilter::<T>::new_for(title, value, on_change, cx);
        if self.searchable {
            entity.searchable(cx)
        } else {
            entity
        }
    }
}

impl<T> GpuiTableFilterShape for FacetedFilter<T>
where
    T: Filterable,
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use gpui_table_core::filter::{FacetedValue, FilterType, RangeValue, TextValue};
    use gpui_table_runtime::shape::{GpuiTableFilterShape, GpuiTableFilterShapeFor};

    #[cfg(feature = "chrono")]
    use super::{DateRangeFilter, DateRangeFilterAdapter, DateRangeFilterField};
    use super::{
        FacetedFilter, FacetedFilterArgs, TextFilter, TextFilterAdapter, TextFilterArgs,
        TextFilterField,
    };
    #[cfg(feature = "rust_decimal")]
    use super::{
        NumberRangeFilter, NumberRangeFilterAdapter, NumberRangeFilterArgs, NumberRangeFilterField,
    };

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    struct Label(String);

    impl TextFilterField for Label {
        fn to_filter_text(&self) -> String {
            self.0.clone()
        }
    }

    #[test]
    fn text_shapes_match_owned_optional_and_adapter_fields() {
        let active = TextValue::from("rust");
        let inactive = TextValue::new();
        let owned = "Trustworthy".to_string();

        assert!(<TextFilter as GpuiTableFilterShapeFor<String>>::matches_field(&owned, &active));
        assert!(
            !<TextFilter as GpuiTableFilterShapeFor<String>>::matches_field(
                &"tables".to_string(),
                &active,
            )
        );
        assert!(
            <TextFilter as GpuiTableFilterShapeFor<Option<String>>>::matches_field(
                &None, &inactive,
            )
        );
        assert!(
            <TextFilter as GpuiTableFilterShapeFor<Option<String>>>::matches_field(
                &Some(owned),
                &active,
            )
        );
        assert!(
            !<TextFilter as GpuiTableFilterShapeFor<Option<String>>>::matches_field(&None, &active,)
        );

        assert!(
            <TextFilterAdapter as GpuiTableFilterShapeFor<Label>>::matches_field(
                &Label("Rust language".into()),
                &active,
            )
        );
        assert!(<TextFilterAdapter as GpuiTableFilterShapeFor<
            Option<Label>,
        >>::matches_field(&None, &inactive,));
        assert!(!<TextFilterAdapter as GpuiTableFilterShapeFor<
            Option<Label>,
        >>::matches_field(&None, &active,));

        assert!(matches!(
            <TextFilter as GpuiTableFilterShapeFor<String>>::filter_type(),
            FilterType::Text
        ));
        assert!(matches!(
            <TextFilterAdapter as GpuiTableFilterShapeFor<Label>>::filter_type(),
            FilterType::Text
        ));
        assert_eq!(
            <TextFilter as GpuiTableFilterShape>::wrap_value("query".into()),
            TextValue::from("query")
        );

        assert_eq!(
            TextFilter::alphabetic_only().validation,
            super::TextFilterValidation::Alphabetic
        );
        assert_eq!(
            TextFilter::numeric_only().validation,
            super::TextFilterValidation::Numeric
        );
        assert_eq!(
            TextFilter::alphanumeric_only().validation,
            super::TextFilterValidation::Alphanumeric
        );
        assert_eq!(
            TextFilter::matching_regex("[a-z]+").validation,
            super::TextFilterValidation::MatchingRegex("[a-z]+")
        );
        assert_eq!(
            TextFilterArgs::default().validation,
            super::TextFilterValidation::None
        );
    }

    #[test]
    fn faceted_shapes_match_scalars_optional_values_and_collections() {
        let active = FacetedValue(HashSet::from([true]));
        let inactive = FacetedValue::<bool>::new();

        assert!(
            <FacetedFilter<bool> as GpuiTableFilterShapeFor<bool>>::matches_field(&true, &active,)
        );
        assert!(
            !<FacetedFilter<bool> as GpuiTableFilterShapeFor<bool>>::matches_field(&false, &active,)
        );
        assert!(<FacetedFilter<bool> as GpuiTableFilterShapeFor<
            Option<bool>,
        >>::matches_field(&None, &inactive,));
        assert!(!<FacetedFilter<bool> as GpuiTableFilterShapeFor<
            Option<bool>,
        >>::matches_field(&None, &active,));
        assert!(<FacetedFilter<bool> as GpuiTableFilterShapeFor<
            Vec<bool>,
        >>::matches_field(&vec![false, true], &active,));
        assert!(!<FacetedFilter<bool> as GpuiTableFilterShapeFor<
            Vec<bool>,
        >>::matches_field(&vec![false], &active,));
        assert!(<FacetedFilter<bool> as GpuiTableFilterShapeFor<
            Option<Vec<bool>>,
        >>::matches_field(&Some(vec![true]), &active,));
        assert!(!<FacetedFilter<bool> as GpuiTableFilterShapeFor<
            Option<Vec<bool>>,
        >>::matches_field(&None, &active,));
        assert!(<FacetedFilter<bool> as GpuiTableFilterShapeFor<
            Option<Vec<bool>>,
        >>::matches_field(&None, &inactive,));
        assert!(matches!(
            <FacetedFilter<bool> as GpuiTableFilterShapeFor<bool>>::filter_type(),
            FilterType::Faceted(options) if options.len() == 2
        ));
        assert_eq!(
            <FacetedFilter<bool> as GpuiTableFilterShape>::wrap_value(HashSet::from([true])),
            active
        );
        assert!(!FacetedFilterArgs::<bool>::default().searchable);
        assert!(FacetedFilter::<bool>::searchable(true).searchable);
    }

    #[cfg(feature = "rust_decimal")]
    #[test]
    fn numeric_shapes_match_all_primitive_categories_and_adapter_fields() {
        use rust_decimal::Decimal;

        let active = RangeValue(Some(Decimal::TEN), Some(Decimal::from(20)));
        let inactive = RangeValue::<Decimal>::new();

        macro_rules! assert_numeric_shape {
            ($($ty:ty),* $(,)?) => {
                $(
                    assert!(
                        <NumberRangeFilter as GpuiTableFilterShapeFor<$ty>>::matches_field(
                            &(15 as $ty),
                            &active,
                        )
                    );
                    assert!(
                        <NumberRangeFilter as GpuiTableFilterShapeFor<Option<$ty>>>::matches_field(
                            &None,
                            &inactive,
                        )
                    );
                    assert!(
                        <NumberRangeFilterAdapter as GpuiTableFilterShapeFor<$ty>>::matches_field(
                            &(15 as $ty),
                            &active,
                        )
                    );
                    assert!(!
                        <NumberRangeFilterAdapter as GpuiTableFilterShapeFor<Option<$ty>>>::matches_field(
                            &None,
                            &active,
                        )
                    );
                    assert_eq!(
                        <$ty as NumberRangeFilterField>::to_filter_decimal(&(15 as $ty)),
                        Decimal::from(15),
                    );
                )*
            };
        }

        assert_numeric_shape!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);
        assert_numeric_shape!(f32, f64);
        assert!(
            <NumberRangeFilter as GpuiTableFilterShapeFor<Decimal>>::matches_field(
                &Decimal::from(15),
                &active,
            )
        );
        assert!(<NumberRangeFilterAdapter as GpuiTableFilterShapeFor<
            Option<Decimal>,
        >>::matches_field(&Some(Decimal::from(15)), &active,));
        assert_eq!(
            <Decimal as NumberRangeFilterField>::to_filter_decimal(&Decimal::from(15)),
            Decimal::from(15)
        );
        assert!(matches!(
            <NumberRangeFilter as GpuiTableFilterShapeFor<i32>>::filter_type(),
            FilterType::NumberRange
        ));
        assert!(matches!(
            <NumberRangeFilterAdapter as GpuiTableFilterShapeFor<i32>>::filter_type(),
            FilterType::NumberRange
        ));
        assert_eq!(
            <NumberRangeFilter as GpuiTableFilterShape>::wrap_value((
                Some(Decimal::TEN),
                Some(Decimal::from(20)),
            )),
            active
        );

        let args = NumberRangeFilter::range(Decimal::ZERO, Decimal::ONE_HUNDRED).step(Decimal::ONE);
        assert_eq!(
            args,
            NumberRangeFilterArgs::default()
                .range(Decimal::ZERO, Decimal::ONE_HUNDRED)
                .step(Decimal::ONE)
        );
        assert_eq!(
            NumberRangeFilter::step(Decimal::TEN).step,
            Some(Decimal::TEN)
        );
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn date_shapes_match_date_datetime_and_zoned_categories() {
        use chrono::{NaiveDate, TimeZone as _, Utc};

        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let middle = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        let active = RangeValue(Some(start), Some(end));
        let inactive = RangeValue::<NaiveDate>::new();
        let datetime = middle.and_hms_opt(12, 0, 0).unwrap();
        let zoned = Utc.from_utc_datetime(&datetime);

        assert!(
            <DateRangeFilter as GpuiTableFilterShapeFor<NaiveDate>>::matches_field(
                &middle, &active,
            )
        );
        assert!(<DateRangeFilter as GpuiTableFilterShapeFor<
            chrono::NaiveDateTime,
        >>::matches_field(&datetime, &active,));
        assert!(<DateRangeFilter as GpuiTableFilterShapeFor<
            chrono::DateTime<Utc>,
        >>::matches_field(&zoned, &active,));
        assert!(<DateRangeFilter as GpuiTableFilterShapeFor<
            Option<NaiveDate>,
        >>::matches_field(&None, &inactive,));
        assert!(!<DateRangeFilter as GpuiTableFilterShapeFor<
            Option<chrono::NaiveDateTime>,
        >>::matches_field(&None, &active,));
        assert!(<DateRangeFilterAdapter as GpuiTableFilterShapeFor<
            chrono::DateTime<Utc>,
        >>::matches_field(&zoned, &active,));
        assert!(!<DateRangeFilterAdapter as GpuiTableFilterShapeFor<
            Option<NaiveDate>,
        >>::matches_field(&None, &active,));
        assert_eq!(
            <NaiveDate as DateRangeFilterField>::to_filter_naive_date(&middle),
            middle
        );
        assert_eq!(
            <chrono::NaiveDateTime as DateRangeFilterField>::to_filter_naive_date(&datetime),
            middle
        );
        assert_eq!(
            <chrono::DateTime<Utc> as DateRangeFilterField>::to_filter_naive_date(&zoned),
            middle
        );
        assert!(matches!(
            <DateRangeFilter as GpuiTableFilterShapeFor<NaiveDate>>::filter_type(),
            FilterType::DateRange
        ));
        assert!(matches!(
            <DateRangeFilterAdapter as GpuiTableFilterShapeFor<NaiveDate>>::filter_type(),
            FilterType::DateRange
        ));
        assert_eq!(
            <DateRangeFilter as GpuiTableFilterShape>::wrap_value((Some(start), Some(end))),
            active
        );
    }
}
