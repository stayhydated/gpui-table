//! Table filter shape contracts.

pub use component_shape::{
    ComponentShapeFor, ComponentShapeMetadata, DeclaredComponentShape, McpInput, McpInputShape,
    McpPrimitiveKind, McpRangeBoundKind,
};
use gpui::{App, Entity, Window};
use gpui_table_core::filter::FilterType;
use gpui_table_schema::registry::RegistryFilterType;
use std::marker::PhantomData;

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

    /// Convert a generated typed filter value back into component-facing state.
    ///
    /// Implement this together with [`Self::set_silent`] when generated filter
    /// collections should support `apply_values(...)` for restored presets.
    fn unwrap_value(_value: Self::FilterValue) -> Self::RawValue {
        panic!("filter shape does not support applying a typed preset")
    }

    /// Replace the component value without running its public change callback.
    ///
    /// The default resets the component. Shapes that support applying a
    /// non-default typed preset must override this method.
    fn set_silent(
        entity: &Entity<Self::Component>,
        _value: Self::RawValue,
        window: &mut Window,
        cx: &mut App,
    ) {
        Self::reset_silent(entity, window, cx);
    }

    /// Reset the component without running its public change callback.
    fn reset_silent(entity: &Entity<Self::Component>, window: &mut Window, cx: &mut App);
}

/// Configured builder for a table filter shape.
///
/// Generated code uses this contract when a field selects a filter shape with a
/// configuration expression, such as `FacetedFilter::<Status>.searchable(true)`.
pub trait GpuiTableFilterShapeBuilder<Shape: GpuiTableFilterShape> {
    /// Build the configured filter entity.
    fn build(
        self,
        title: impl Fn(&App) -> String + 'static,
        value: Shape::RawValue,
        on_change: impl Fn(Shape::RawValue, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Shape::Component>;
}

/// Default builder for a shape's normal [`GpuiTableFilterShape::new_for`]
/// behavior.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DefaultGpuiTableFilterShapeBuilder<Shape>(PhantomData<fn() -> Shape>);

impl<Shape> DefaultGpuiTableFilterShapeBuilder<Shape> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<Shape> GpuiTableFilterShapeBuilder<Shape> for DefaultGpuiTableFilterShapeBuilder<Shape>
where
    Shape: GpuiTableFilterShape,
{
    fn build(
        self,
        title: impl Fn(&App) -> String + 'static,
        value: Shape::RawValue,
        on_change: impl Fn(Shape::RawValue, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Shape::Component> {
        Shape::new_for(title, value, on_change, cx)
    }
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

/// Build a filter entity from a configured filter-shape builder.
pub fn build_filter_shape<Shape, Builder>(
    builder: Builder,
    title: impl Fn(&App) -> String + 'static,
    value: Shape::RawValue,
    on_change: impl Fn(Shape::RawValue, &mut Window, &mut App) + 'static,
    cx: &mut App,
) -> Entity<Shape::Component>
where
    Shape: GpuiTableFilterShape,
    Builder: GpuiTableFilterShapeBuilder<Shape>,
{
    builder.build(title, value, on_change, cx)
}

pub type GpuiTableFilterComponentOf<Shape> = <Shape as GpuiTableFilterShape>::Component;
pub type GpuiTableFilterRawValueOf<Shape> = <Shape as GpuiTableFilterShape>::RawValue;
pub type GpuiTableFilterValueOf<Shape> = <Shape as GpuiTableFilterShape>::FilterValue;

pub struct _PrivatePhantom<T>(PhantomData<T>);

#[cfg(test)]
mod tests {
    use super::DefaultGpuiTableFilterShapeBuilder;

    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    struct Shape;

    #[test]
    fn default_shape_builder_is_a_copyable_zero_sized_value() {
        let builder = DefaultGpuiTableFilterShapeBuilder::<Shape>::new();
        let copied = builder;

        assert_eq!(builder, copied);
        assert_eq!(builder, DefaultGpuiTableFilterShapeBuilder::default());
        assert_eq!(std::mem::size_of_val(&builder), 0);
    }
}
