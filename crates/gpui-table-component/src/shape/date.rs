use super::shared::{delegate_filter_shape, matches_optional_field};
use super::*;

#[cfg(feature = "jiff")]
/// Adapter shape for date range filters over application-owned field types.
///
/// Use this when a table field is a transparent or domain-specific value type
/// that should be matched by a date value while reusing the built-in
/// [`DateRangeFilter`] component and MCP schema.
pub struct DateRangeFilterAdapter;

#[cfg(feature = "jiff")]
/// Field conversion contract used by [`DateRangeFilterAdapter`].
pub trait DateRangeFilterField {
    /// Converts the field value into the date matched by [`DateRangeFilter`].
    fn to_filter_date(&self) -> jiff::civil::Date;
}

#[cfg(feature = "jiff")]
macro_rules! impl_date_range_filter_field {
    ($($ty:ty),* $(,)?) => {
        $(
            impl DateRangeFilterField for $ty {
                fn to_filter_date(&self) -> jiff::civil::Date {
                    gpui_table_core::filter::ToCivilDate::to_civil_date(self)
                }
            }
        )*
    };
}

#[cfg(feature = "jiff")]
impl_date_range_filter_field!(
    jiff::civil::Date,
    jiff::civil::DateTime,
    jiff::Timestamp,
    jiff::ZonedDateTime
);

#[cfg(all(feature = "jiff", feature = "spacetimedb"))]
impl_date_range_filter_field!(spacetimedb_lib::Timestamp);

#[cfg(feature = "jiff")]
delegate_filter_shape!(DateRangeFilterAdapter, DateRangeFilter);

#[cfg(feature = "jiff")]
impl<T> ComponentShapeFor<T> for DateRangeFilterAdapter where T: DateRangeFilterField {}

#[cfg(feature = "jiff")]
impl<T> ComponentShapeFor<Option<T>> for DateRangeFilterAdapter where T: DateRangeFilterField {}

#[cfg(feature = "jiff")]
impl<T> GpuiTableFilterShapeFor<T> for DateRangeFilterAdapter
where
    T: DateRangeFilterField,
{
    fn filter_type() -> FilterType {
        FilterType::DateRange
    }

    fn matches_field(field: &T, value: &Self::FilterValue) -> bool {
        value.matches(&field.to_filter_date())
    }
}

#[cfg(feature = "jiff")]
impl<T> GpuiTableFilterShapeFor<Option<T>> for DateRangeFilterAdapter
where
    T: DateRangeFilterField,
{
    fn filter_type() -> FilterType {
        FilterType::DateRange
    }

    fn matches_field(field: &Option<T>, value: &Self::FilterValue) -> bool {
        matches_optional_field(field, value.is_active(), |field| {
            value.matches(&field.to_filter_date())
        })
    }
}

#[cfg(feature = "jiff")]
impl GpuiTableFilterShape for DateRangeFilter {
    type Component = DateRangeFilter;
    type RawValue = (Option<jiff::civil::Date>, Option<jiff::civil::Date>);
    type FilterValue = gpui_table_core::filter::RangeValue<jiff::civil::Date>;

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

    fn unwrap_value(value: Self::FilterValue) -> Self::RawValue {
        (value.0, value.1)
    }

    fn set_silent(
        entity: &Entity<Self::Component>,
        value: Self::RawValue,
        window: &mut Window,
        cx: &mut App,
    ) {
        entity.update(cx, |filter, cx| filter.set_silent(value, window, cx));
    }

    fn reset_silent(entity: &Entity<Self::Component>, window: &mut Window, cx: &mut App) {
        entity.update(cx, |filter, cx| filter.reset_silent(window, cx));
    }
}

#[cfg(feature = "jiff")]
impl DeclaredGpuiTableFilterShape for DateRangeFilter {}

#[cfg(feature = "jiff")]
macro_rules! impl_date_range_shape_for {
    ($($ty:ty),* $(,)?) => {
        $(
            impl GpuiTableFilterShapeFor<$ty> for DateRangeFilter {
                fn filter_type() -> FilterType {
                    FilterType::DateRange
                }

                fn matches_field(field: &$ty, value: &Self::FilterValue) -> bool {
                    value.matches(&gpui_table_core::filter::ToCivilDate::to_civil_date(field))
                }
            }

            impl GpuiTableFilterShapeFor<Option<$ty>> for DateRangeFilter {
                fn filter_type() -> FilterType {
                    FilterType::DateRange
                }

                fn matches_field(field: &Option<$ty>, value: &Self::FilterValue) -> bool {
                    matches_optional_field(field, value.is_active(), |field| {
                        value.matches(&gpui_table_core::filter::ToCivilDate::to_civil_date(field))
                    })
                }
            }
        )*
    };
}

#[cfg(feature = "jiff")]
impl_date_range_shape_for!(
    jiff::civil::Date,
    jiff::civil::DateTime,
    jiff::Timestamp,
    jiff::ZonedDateTime
);

#[cfg(all(feature = "jiff", feature = "spacetimedb"))]
impl_date_range_shape_for!(spacetimedb_lib::Timestamp);

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_table_core::filter::{FilterType, RangeValue};
    use gpui_table_runtime::shape::{GpuiTableFilterShape, GpuiTableFilterShapeFor};

    #[cfg(feature = "jiff")]
    #[test]
    fn date_shapes_match_date_datetime_and_zoned_categories() {
        use jiff::{ZonedDateTime, civil::Date as CivilDate};

        let start = CivilDate::new(2026, 1, 1).unwrap();
        let middle = CivilDate::new(2026, 1, 15).unwrap();
        let end = CivilDate::new(2026, 1, 31).unwrap();
        let active = RangeValue(Some(start), Some(end));
        let inactive = RangeValue::<CivilDate>::new();
        let datetime = middle.at(12, 0, 0, 0);
        let zoned: ZonedDateTime = "2026-01-15T12:00:00+01:00[Europe/Paris]".parse().unwrap();

        assert!(
            <DateRangeFilter as GpuiTableFilterShapeFor<CivilDate>>::matches_field(
                &middle, &active,
            )
        );
        assert!(<DateRangeFilter as GpuiTableFilterShapeFor<
            jiff::civil::DateTime,
        >>::matches_field(&datetime, &active,));
        assert!(<DateRangeFilter as GpuiTableFilterShapeFor<
            ZonedDateTime,
        >>::matches_field(&zoned, &active,));
        assert!(<DateRangeFilter as GpuiTableFilterShapeFor<
            Option<CivilDate>,
        >>::matches_field(&None, &inactive,));
        assert!(!<DateRangeFilter as GpuiTableFilterShapeFor<
            Option<jiff::civil::DateTime>,
        >>::matches_field(&None, &active,));
        assert!(<DateRangeFilterAdapter as GpuiTableFilterShapeFor<
            ZonedDateTime,
        >>::matches_field(&zoned, &active,));
        assert!(!<DateRangeFilterAdapter as GpuiTableFilterShapeFor<
            Option<CivilDate>,
        >>::matches_field(&None, &active,));
        assert_eq!(
            <CivilDate as DateRangeFilterField>::to_filter_date(&middle),
            middle
        );
        assert_eq!(
            <jiff::civil::DateTime as DateRangeFilterField>::to_filter_date(&datetime),
            middle
        );
        assert_eq!(
            <ZonedDateTime as DateRangeFilterField>::to_filter_date(&zoned),
            middle
        );
        assert!(matches!(
            <DateRangeFilter as GpuiTableFilterShapeFor<CivilDate>>::filter_type(),
            FilterType::DateRange
        ));
        assert!(matches!(
            <DateRangeFilterAdapter as GpuiTableFilterShapeFor<CivilDate>>::filter_type(),
            FilterType::DateRange
        ));
        assert_eq!(
            <DateRangeFilter as GpuiTableFilterShape>::wrap_value((Some(start), Some(end))),
            active
        );
    }
}
