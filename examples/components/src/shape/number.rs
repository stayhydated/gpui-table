use super::shared::{delegate_filter_shape, matches_optional_field};
use super::*;

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
        matches_optional_field(field, value.is_active(), |field| {
            value.matches(&field.to_filter_decimal())
        })
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
                    matches_optional_field(field, value.is_active(), |field| {
                        value.matches(&gpui_table_core::filter::ToDecimal::to_decimal(field))
                    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_table_core::filter::{FilterType, RangeValue};
    use gpui_table_runtime::shape::{GpuiTableFilterShape, GpuiTableFilterShapeFor};

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
}
