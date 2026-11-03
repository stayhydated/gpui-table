use super::shared::matches_optional_field;
use super::*;

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

    fn unwrap_value(value: Self::FilterValue) -> Self::RawValue {
        value.0
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
        matches_optional_field(field, value.is_active(), |field| value.matches(field))
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
        matches_optional_field(field, value.is_active(), |field| {
            field.iter().any(|field| value.matches(field))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_table_core::filter::{FacetedValue, FilterType};
    use gpui_table_runtime::shape::{GpuiTableFilterShape, GpuiTableFilterShapeFor};

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
}
