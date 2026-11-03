use super::shared::{delegate_filter_shape, matches_optional_field};
use super::*;

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
        matches_optional_field(field, value.is_active(), |field| {
            value.matches(&field.to_filter_naive_date())
        })
    }
}

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
                    matches_optional_field(field, value.is_active(), |field| {
                        value.matches(&gpui_table_core::filter::ToNaiveDate::to_naive_date(field))
                    })
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
        matches_optional_field(field, value.is_active(), |field| {
            value.matches(&field.date_naive())
        })
    }
}

#[cfg(all(feature = "chrono", feature = "spacetimedb"))]
impl_date_range_shape_for!(spacetimedb_lib::Timestamp);

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_table_core::filter::{FilterType, RangeValue};
    use gpui_table_runtime::shape::{GpuiTableFilterShape, GpuiTableFilterShapeFor};

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
