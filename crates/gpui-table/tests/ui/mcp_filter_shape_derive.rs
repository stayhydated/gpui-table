use gpui::Entity;
use gpui_table::{
    GpuiTable, McpFilterShape,
    core::filter::{FilterType, TextValue},
    runtime::shape::{
        ComponentShapeMetadata, DeclaredComponentShape, DeclaredGpuiTableFilterShape,
        GpuiTableFilterShape, GpuiTableFilterShapeFor,
    },
    schema::registry::RegistryFilterType,
};
use serde::{Deserialize, Serialize};

#[derive(McpFilterShape)]
struct LocalTextFilter;

impl ComponentShapeMetadata for LocalTextFilter {}
impl DeclaredComponentShape for LocalTextFilter {}

impl GpuiTableFilterShape for LocalTextFilter {
    type Component = gpui_table_component::TextFilter;
    type RawValue = String;
    type FilterValue = TextValue;

    const FILTER_TYPE: RegistryFilterType = RegistryFilterType::Text;

    fn new_for(
        _title: impl Fn(&gpui::App) -> String + 'static,
        _value: Self::RawValue,
        _on_change: impl Fn(Self::RawValue, &mut gpui::Window, &mut gpui::App) + 'static,
        _cx: &mut gpui::App,
    ) -> Entity<Self::Component> {
        unimplemented!()
    }

    fn read_value(_entity: &Entity<Self::Component>, _cx: &gpui::App) -> Self::RawValue {
        String::new()
    }

    fn wrap_value(value: Self::RawValue) -> Self::FilterValue {
        TextValue::from(value)
    }

    fn reset_silent(
        _entity: &Entity<Self::Component>,
        _window: &mut gpui::Window,
        _cx: &mut gpui::App,
    ) {
    }
}

impl DeclaredGpuiTableFilterShape for LocalTextFilter {}

impl GpuiTableFilterShapeFor<String> for LocalTextFilter {
    fn filter_type() -> FilterType {
        FilterType::Text
    }

    fn matches_field(field: &String, value: &Self::FilterValue) -> bool {
        value.matches(field.as_str())
    }
}

#[derive(Clone, Debug, Deserialize, GpuiTable, Serialize)]
#[gpui_table(filters, mcp)]
struct Row {
    #[gpui_table(filter(LocalTextFilter))]
    name: String,
}

fn main() {}
