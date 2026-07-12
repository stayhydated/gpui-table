use gpui_table::{GpuiTable, GpuiTableFilterShape};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq)]
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

#[derive(GpuiTableFilterShape)]
#[gpui_table_filter_shape(
    base = gpui_table_component::TextFilter,
    raw_value = PrefixText,
    field = String,
    into_base = |value: PrefixText| value.0,
    from_base = PrefixText
)]
struct PrefixTextFilter;

#[derive(Clone, Debug, Deserialize, GpuiTable, Serialize)]
#[gpui_table(filters, mcp)]
struct Row {
    #[gpui_table(filter(PrefixTextFilter))]
    name: String,
}

fn assert_shape_contracts()
where
    PrefixTextFilter: gpui_table::runtime::shape::DeclaredGpuiTableFilterShape
        + gpui_table::runtime::shape::GpuiTableFilterShapeFor<String>
        + gpui_table::mcp::McpFilterShape,
{
}

fn main() {
    assert_shape_contracts();
}
