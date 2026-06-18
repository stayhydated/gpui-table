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

pub struct _PrivatePhantom<T>(PhantomData<T>);
