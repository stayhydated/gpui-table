use gpui_table::mcp::{McpTable as _, serde_json::json, tool_definitions};
use serde::Deserialize;

use super::{
    NewtypeValidatedFilterRow, NoFilterQueryRow, PrefixFilterRow, UserRow, ValidatedFilterRow,
};

#[derive(Clone, Debug, Deserialize, gpui_table::mcp::McpJsonSchema, PartialEq)]
#[allow(dead_code)]
#[serde(rename_all = "snake_case")]
enum ExportMode {
    Summary,
    FullDetail,
}

#[derive(Debug, Deserialize, gpui_table::mcp::McpToolInput, PartialEq)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
struct ExportArgs {
    #[mcp(alias = "mode")]
    export_mode: ExportMode,
}

#[test]
fn descriptor_input_schema_maps_filter_fields() {
    let schema = UserRow::descriptor().input_schema();

    assert_eq!(schema["properties"]["name"]["type"], "string");
    assert_eq!(schema["properties"]["status"]["type"], "array");
    assert_eq!(
        schema["properties"]["status"]["items"]["enum"],
        json!(["Active", "Blocked"])
    );
    assert_eq!(schema["properties"]["status"]["uniqueItems"], true);
    assert_eq!(
        schema["properties"]["status"]["x-gpuiTableFacetOptions"][0]["label"],
        "Active"
    );
    assert_eq!(schema["properties"]["age"]["type"], "object");
    assert_eq!(
        schema["properties"]["created_on"]["properties"]["min"]["anyOf"][0]["format"],
        "date"
    );
}

#[test]
fn facade_mcp_json_schema_derive_supports_enums() {
    let schema = <ExportMode as gpui_table::mcp::McpJsonSchema>::json_schema();

    assert_eq!(schema["type"], "string");
    assert_eq!(schema["enum"], json!(["summary", "full_detail"]));
}

#[test]
fn facade_mcp_json_schema_supports_fixed_tuples() {
    let schema = <(u32, String) as gpui_table::mcp::McpJsonSchema>::json_schema();

    assert_eq!(schema["type"], "array");
    assert_eq!(schema["prefixItems"][0]["type"], "integer");
    assert_eq!(schema["prefixItems"][1]["type"], "string");
    assert_eq!(schema["minItems"], 2);
    assert_eq!(schema["maxItems"], 2);
}

#[test]
fn facade_mcp_json_schema_supports_feature_gated_types() {
    let date_schema = <chrono::NaiveDate as gpui_table::mcp::McpJsonSchema>::json_schema();
    assert_eq!(date_schema["type"], "string");
    assert_eq!(date_schema["format"], "date");

    let decimal_schema = <rust_decimal::Decimal as gpui_table::mcp::McpJsonSchema>::json_schema();
    assert_eq!(decimal_schema["anyOf"][0]["type"], "number");
    assert_eq!(decimal_schema["anyOf"][1]["type"], "string");
}

#[test]
fn facade_mcp_tool_input_derive_is_reusable_schema() {
    let schema = <ExportArgs as gpui_table::mcp::McpJsonSchema>::json_schema();

    assert_eq!(schema["properties"]["exportMode"]["type"], "string");
    assert_eq!(
        schema["properties"]["exportMode"]["enum"],
        json!(["summary", "full_detail"])
    );
    assert_eq!(
        schema["properties"]["exportMode"]["x-mcpAliases"],
        json!(["mode"])
    );

    let input = <ExportArgs as gpui_table::mcp::McpToolInput>::from_tool_call(
        gpui_table::mcp::McpToolCall::from_value(Some(json!({ "mode": "summary" })))
            .expect("tool call should normalize"),
    )
    .expect("facade-derived tool input should decode aliases");

    assert_eq!(
        input,
        ExportArgs {
            export_mode: ExportMode::Summary
        }
    );
}

#[test]
fn descriptor_for_no_filter_row_includes_only_pagination_arguments() {
    let schema = NoFilterQueryRow::descriptor().input_schema();

    let properties = schema["properties"]
        .as_object()
        .expect("input schema should have properties");
    assert_eq!(properties.len(), 2);
    assert_eq!(schema["properties"]["limit"]["type"], "integer");
    assert_eq!(schema["properties"]["offset"]["type"], "integer");
    assert!(schema["properties"]["name"].is_null());
}

#[test]
fn descriptor_uses_explicit_mcp_metadata() {
    let descriptor = UserRow::descriptor();

    assert_eq!(descriptor.tool_name(), "query_users");
    assert!(descriptor.has_row_schema());
    assert_eq!(descriptor.title(), "Query users");
    assert_eq!(
        descriptor.description(),
        "Exercise explicit table MCP metadata."
    );
    let output_schema = descriptor.output_schema();
    assert_eq!(
        output_schema["properties"]["rows"]["items"]["properties"]["name"]["type"],
        "string"
    );
    assert_eq!(
        output_schema["properties"]["rows"]["items"]["properties"]["status"]["enum"],
        json!(["Active", "Blocked"])
    );
    let annotations = descriptor.tool_annotations();
    assert_eq!(annotations.title.as_deref(), Some("Query users"));
    assert_eq!(annotations.read_only_hint, Some(true));
    assert_eq!(annotations.destructive_hint, Some(false));
    assert_eq!(annotations.idempotent_hint, Some(true));
    assert_eq!(annotations.open_world_hint, Some(true));
}

#[test]
fn filter_shape_adapter_schema_uses_declared_raw_value_schema() {
    let schema = PrefixFilterRow::descriptor().input_schema();

    assert_eq!(schema["properties"]["name"]["type"], "string");
    assert_eq!(schema["properties"]["name"]["x-prefixText"], true);
    assert_eq!(
        schema["properties"]["name"]["x-gpuiTableFilterType"],
        "text"
    );
}

#[test]
fn koruma_filter_validation_schema_attaches_rules_and_hints() {
    let schema = ValidatedFilterRow::descriptor().input_schema();
    let name = &schema["properties"]["name"];

    assert_eq!(name["type"], "string");
    assert_eq!(name["minLength"], 2);
    assert_eq!(name["maxLength"], 5);
    assert_eq!(name["x-gpuiTableValidation"][0]["scope"], "filter");
    assert_eq!(
        name["x-gpuiTableValidation"][0]["validator"],
        "LenValidation"
    );
    assert_eq!(name["x-gpuiTableValidation"][0]["params"][0]["name"], "min");
    assert_eq!(name["x-gpuiTableValidation"][0]["params"][0]["value"], "2");
    assert_eq!(name["x-gpuiTableValidation"][0]["params"][1]["name"], "max");
    assert_eq!(name["x-gpuiTableValidation"][0]["params"][1]["value"], "5");
}

#[test]
fn newtype_filter_validation_schema_attaches_rule() {
    let schema = NewtypeValidatedFilterRow::descriptor().input_schema();
    let name = &schema["properties"]["name"];

    assert_eq!(name["type"], "string");
    assert_eq!(name["x-gpuiTableValidation"][0]["scope"], "filter");
    assert_eq!(name["x-gpuiTableValidation"][0]["validator"], "newtype");
    assert_eq!(
        name["x-gpuiTableValidation"][0]["path"],
        "ValidatedPrefixText"
    );
}

#[test]
fn inventory_exposes_generated_tool_definition() {
    let expected = UserRow::descriptor().tool_name();
    let tools = tool_definitions().expect("tool definitions should be generated");

    let tool = tools
        .iter()
        .find(|tool| tool.name == expected)
        .expect("tool definition should be generated");
    let annotations = tool
        .annotations
        .as_ref()
        .expect("table query should publish annotations");
    assert_eq!(annotations.read_only_hint, Some(true));
    assert_eq!(annotations.destructive_hint, Some(false));
    assert_eq!(annotations.idempotent_hint, Some(true));
    assert_eq!(annotations.open_world_hint, Some(true));
    let output_schema = tool
        .output_schema
        .as_ref()
        .expect("table query should publish output schema");
    assert_eq!(
        output_schema["properties"]["rows"]["items"]["properties"]["name"]["type"],
        "string"
    );
    assert_eq!(
        output_schema["properties"]["rows"]["items"]["properties"]["status"]["enum"],
        json!(["Active", "Blocked"])
    );
}

#[test]
fn inventory_exposes_pagination_only_mcp_tool_definition() {
    let expected = NoFilterQueryRow::descriptor().tool_name();
    let tools = tool_definitions().expect("tool definitions should be generated");

    let tool = tools
        .iter()
        .find(|tool| tool.name == expected)
        .expect("tool definition should be generated");
    let annotations = tool
        .annotations
        .as_ref()
        .expect("table query should publish default annotations");
    assert_eq!(annotations.read_only_hint, Some(true));
    assert_eq!(annotations.destructive_hint, Some(false));
    assert_eq!(annotations.idempotent_hint, Some(true));
    assert_eq!(annotations.open_world_hint, None);
    let output_schema = tool
        .output_schema
        .as_ref()
        .expect("table query should publish output schema");
    assert_eq!(
        output_schema["properties"]["rows"]["items"],
        serde_json::json!({})
    );
}
