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

            fn unwrap_value(value: Self::FilterValue) -> Self::RawValue {
                <$base as GpuiTableFilterShape>::unwrap_value(value)
            }

            fn set_silent(
                entity: &Entity<Self::Component>,
                value: Self::RawValue,
                window: &mut Window,
                cx: &mut App,
            ) {
                <$base as GpuiTableFilterShape>::set_silent(entity, value, window, cx);
            }

            fn reset_silent(entity: &Entity<Self::Component>, window: &mut Window, cx: &mut App) {
                <$base as GpuiTableFilterShape>::reset_silent(entity, window, cx);
            }
        }

        impl DeclaredGpuiTableFilterShape for $shape {}
    };
}

pub(super) use delegate_filter_shape;

pub(super) fn matches_optional_field<Field>(
    field: &Option<Field>,
    is_active: bool,
    matches: impl FnOnce(&Field) -> bool,
) -> bool {
    !is_active || field.as_ref().is_some_and(matches)
}
