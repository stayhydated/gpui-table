//! Table filter shape contracts.

pub use component_shape::{
    ComponentShapeFor, ComponentShapeMetadata, DeclaredComponentShape, McpInput, McpInputShape,
    McpPrimitiveKind,
};
use gpui::{App, Entity, Window};
use gpui_table_core::filter::{FacetedValue, FilterType, TextValue};
use gpui_table_schema::registry::RegistryFilterType;
use std::collections::HashSet;
use std::marker::PhantomData;

#[cfg(feature = "chrono")]
pub use gpui_table_component::DateRangeFilter;
#[cfg(feature = "rust_decimal")]
pub use gpui_table_component::NumberRangeFilter;
pub use gpui_table_component::{FacetedFilter, TextFilter};

/// Table-specific filter shape contract consumed by `#[derive(GpuiTable)]`.
pub trait GpuiTableFilterShape: ComponentShapeMetadata {
    type Component: 'static;
    type RawValue: Default + Clone + Send + 'static;
    type FilterValue: Clone + Send + 'static;

    const FILTER_TYPE: RegistryFilterType;

    fn new_for(
        title: impl Fn(&App) -> String + 'static,
        value: Self::RawValue,
        on_change: impl Fn(Self::RawValue, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self::Component>;

    fn read_value(entity: &Entity<Self::Component>, cx: &App) -> Self::RawValue;

    fn wrap_value(value: Self::RawValue) -> Self::FilterValue;

    fn reset_silent(entity: &Entity<Self::Component>, window: &mut Window, cx: &mut App);
}

/// Marker for filter shapes declared by a component crate or user crate.
#[diagnostic::on_unimplemented(
    message = "table filter shape `{Self}` must implement `DeclaredGpuiTableFilterShape`",
    note = "use a built-in shape such as `gpui_table_component::TextFilter` or implement the table filter shape contract for your custom shape"
)]
pub trait DeclaredGpuiTableFilterShape: GpuiTableFilterShape + DeclaredComponentShape {}

/// Field support and matching behavior for a shape used against a table field.
#[diagnostic::on_unimplemented(
    message = "table filter shape `{Self}` does not support field value `{Field}`",
    note = "implement `GpuiTableFilterShapeFor<{Field}>` for `{Self}`, or choose a filter shape that supports the field type"
)]
pub trait GpuiTableFilterShapeFor<Field>: GpuiTableFilterShape {
    fn filter_type() -> FilterType;

    fn matches_field(field: &Field, value: &Self::FilterValue) -> bool;
}

pub type GpuiTableFilterComponentOf<Shape> = <Shape as GpuiTableFilterShape>::Component;
pub type GpuiTableFilterRawValueOf<Shape> = <Shape as GpuiTableFilterShape>::RawValue;
pub type GpuiTableFilterValueOf<Shape> = <Shape as GpuiTableFilterShape>::FilterValue;

impl GpuiTableFilterShape for gpui_table_component::TextFilter {
    type Component = gpui_table_component::TextFilter;
    type RawValue = String;
    type FilterValue = TextValue;

    const FILTER_TYPE: RegistryFilterType = RegistryFilterType::Text;

    fn new_for(
        title: impl Fn(&App) -> String + 'static,
        value: Self::RawValue,
        on_change: impl Fn(Self::RawValue, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self::Component> {
        gpui_table_component::TextFilter::new_for(title, value, on_change, cx)
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

impl DeclaredGpuiTableFilterShape for gpui_table_component::TextFilter {}

impl GpuiTableFilterShapeFor<String> for gpui_table_component::TextFilter {
    fn filter_type() -> FilterType {
        FilterType::Text
    }

    fn matches_field(field: &String, value: &Self::FilterValue) -> bool {
        value.matches(field.as_ref())
    }
}

impl GpuiTableFilterShapeFor<Option<String>> for gpui_table_component::TextFilter {
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

impl<T> GpuiTableFilterShape for gpui_table_component::FacetedFilter<T>
where
    T: gpui_table_core::filter::Filterable,
{
    type Component = gpui_table_component::FacetedFilter<T>;
    type RawValue = HashSet<T>;
    type FilterValue = FacetedValue<T>;

    const FILTER_TYPE: RegistryFilterType = RegistryFilterType::Faceted;

    fn new_for(
        title: impl Fn(&App) -> String + 'static,
        value: Self::RawValue,
        on_change: impl Fn(Self::RawValue, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self::Component> {
        gpui_table_component::FacetedFilter::<T>::new_for(title, value, on_change, cx)
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

impl<T> DeclaredGpuiTableFilterShape for gpui_table_component::FacetedFilter<T> where
    T: gpui_table_core::filter::Filterable
{
}

impl<T> GpuiTableFilterShapeFor<T> for gpui_table_component::FacetedFilter<T>
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

impl<T> GpuiTableFilterShapeFor<Option<T>> for gpui_table_component::FacetedFilter<T>
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

impl<T> GpuiTableFilterShapeFor<Vec<T>> for gpui_table_component::FacetedFilter<T>
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

impl<T> GpuiTableFilterShapeFor<Option<Vec<T>>> for gpui_table_component::FacetedFilter<T>
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
impl GpuiTableFilterShape for gpui_table_component::NumberRangeFilter {
    type Component = gpui_table_component::NumberRangeFilter;
    type RawValue = (Option<rust_decimal::Decimal>, Option<rust_decimal::Decimal>);
    type FilterValue = gpui_table_core::filter::RangeValue<rust_decimal::Decimal>;

    const FILTER_TYPE: RegistryFilterType = RegistryFilterType::NumberRange;

    fn new_for(
        title: impl Fn(&App) -> String + 'static,
        value: Self::RawValue,
        on_change: impl Fn(Self::RawValue, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self::Component> {
        gpui_table_component::NumberRangeFilter::new_for(title, value, on_change, cx)
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
impl DeclaredGpuiTableFilterShape for gpui_table_component::NumberRangeFilter {}

#[cfg(feature = "rust_decimal")]
macro_rules! impl_number_range_shape_for {
    ($($ty:ty),* $(,)?) => {
        $(
            impl GpuiTableFilterShapeFor<$ty> for gpui_table_component::NumberRangeFilter {
                fn filter_type() -> FilterType {
                    FilterType::NumberRange
                }

                fn matches_field(field: &$ty, value: &Self::FilterValue) -> bool {
                    value.matches(&gpui_table_core::filter::ToDecimal::to_decimal(field))
                }
            }

            impl GpuiTableFilterShapeFor<Option<$ty>> for gpui_table_component::NumberRangeFilter {
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
impl GpuiTableFilterShape for gpui_table_component::DateRangeFilter {
    type Component = gpui_table_component::DateRangeFilter;
    type RawValue = (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>);
    type FilterValue = gpui_table_core::filter::RangeValue<chrono::NaiveDate>;

    const FILTER_TYPE: RegistryFilterType = RegistryFilterType::DateRange;

    fn new_for(
        title: impl Fn(&App) -> String + 'static,
        value: Self::RawValue,
        on_change: impl Fn(Self::RawValue, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self::Component> {
        gpui_table_component::DateRangeFilter::new_for(title, value, on_change, cx)
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
impl DeclaredGpuiTableFilterShape for gpui_table_component::DateRangeFilter {}

#[cfg(feature = "chrono")]
macro_rules! impl_date_range_shape_for {
    ($($ty:ty),* $(,)?) => {
        $(
            impl GpuiTableFilterShapeFor<$ty> for gpui_table_component::DateRangeFilter {
                fn filter_type() -> FilterType {
                    FilterType::DateRange
                }

                fn matches_field(field: &$ty, value: &Self::FilterValue) -> bool {
                    value.matches(&gpui_table_core::filter::ToNaiveDate::to_naive_date(field))
                }
            }

            impl GpuiTableFilterShapeFor<Option<$ty>> for gpui_table_component::DateRangeFilter {
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
impl<Tz> GpuiTableFilterShapeFor<chrono::DateTime<Tz>> for gpui_table_component::DateRangeFilter
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
impl<Tz> GpuiTableFilterShapeFor<Option<chrono::DateTime<Tz>>>
    for gpui_table_component::DateRangeFilter
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

pub struct _PrivatePhantom<T>(PhantomData<T>);
