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

#[derive(Clone, Default)]
struct PrefixText(String);

impl gpui_table::mcp::McpToolValue for PrefixText {
    fn tool_value_schema() -> gpui_table::mcp::McpSchema {
        gpui_table::mcp::McpSchema::string()
    }

    fn from_tool_value(
        field: &str,
        value: gpui_table::mcp::serde_json::Value,
    ) -> Result<Self, gpui_table::mcp::McpToolError> {
        let raw = value
            .as_str()
            .ok_or_else(|| gpui_table::mcp::McpToolError::decode(field, "expected string"))?;
        Ok(Self(raw.to_string()))
    }
}

#[derive(McpFilterShape)]
struct LocalTextFilter;

impl ComponentShapeMetadata for LocalTextFilter {}
impl DeclaredComponentShape for LocalTextFilter {}

impl GpuiTableFilterShape for LocalTextFilter {
    type Component = gpui_table_component::TextFilter;
    type RawValue = PrefixText;
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
        PrefixText::default()
    }

    fn wrap_value(value: Self::RawValue) -> Self::FilterValue {
        TextValue::from(value.0)
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
