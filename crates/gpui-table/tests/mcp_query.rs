#![cfg(all(feature = "mcp", feature = "rust_decimal"))]

use chrono::NaiveDate;
use gpui_table::{
    Filterable, GpuiTable, GpuiTableFilterShape, TableCell,
    mcp::{
        McpServer, McpTable as _, McpToolError, register_table_resources, resource_definitions,
        serde_json::json, server as generated_server, table, table_resource_uris_for,
        tool_definitions,
    },
};
use koruma::NewtypeTryFromInner as _;
use koruma_collection::collection::LenValidation;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Filterable, Hash, PartialEq, Serialize, TableCell)]
enum UserStatus {
    Active,
    Blocked,
}

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

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(
    id = "users",
    title = "Users",
    filters,
    mcp(
        name = "query_users",
        title = "Query users",
        description = "Exercise explicit table MCP metadata.",
        read_only = true,
        destructive = false,
        idempotent = true,
        open_world = true
    )
)]
struct UserRow {
    #[gpui_table(filter(gpui_table::runtime::shape::TextFilter))]
    name: String,

    #[gpui_table(filter(gpui_table::runtime::shape::FacetedFilter::<UserStatus>))]
    status: UserStatus,

    #[gpui_table(filter(gpui_table::runtime::shape::NumberRangeFilter))]
    age: u8,

    #[gpui_table(filter(gpui_table::runtime::shape::DateRangeFilter))]
    created_on: NaiveDate,
}

/// Query rows from inferred docs.
#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(id = "query_rows", title = "Query Rows", filters, mcp)]
struct QueryRow {
    #[gpui_table(filter(gpui_table::runtime::shape::TextFilter))]
    name: String,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(id = "direct_query_rows", title = "Direct Query Rows", filters, mcp)]
struct DirectQueryRow {
    #[gpui_table(filter(gpui_table::runtime::shape::TextFilter))]
    name: String,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(
    id = "no_filter_query_rows",
    title = "No Filter Query Rows",
    mcp(
        name = "query_rows_no_filters",
        title = "Query rows without filters",
        description = "Exercise pagination-only MCP behavior."
    )
)]
struct NoFilterQueryRow {
    name: String,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(id = "async_query_rows", title = "Async Query Rows", filters, mcp)]
struct AsyncQueryRow {
    #[gpui_table(filter(gpui_table::runtime::shape::TextFilter))]
    name: String,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(
    id = "async_value_query_rows",
    title = "Async Value Query Rows",
    filters,
    mcp
)]
struct AsyncValueQueryRow {
    #[gpui_table(filter(gpui_table::runtime::shape::TextFilter))]
    name: String,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(id = "async_source_rows", title = "Async Source Rows", filters, mcp)]
struct AsyncSourceRow {
    #[gpui_table(filter(gpui_table::runtime::shape::TextFilter))]
    name: String,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(id = "result_source_rows", title = "Result Source Rows", filters, mcp)]
struct ResultSourceRow {
    #[gpui_table(filter(gpui_table::runtime::shape::TextFilter))]
    name: String,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(
    id = "async_result_source_rows",
    title = "Async Result Source Rows",
    filters,
    mcp
)]
struct AsyncResultSourceRow {
    #[gpui_table(filter(gpui_table::runtime::shape::TextFilter))]
    name: String,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(
    id = "mcp_only_filter_rows",
    title = "MCP Only Filter Rows",
    mcp(name = "query_mcp_only_filter_rows")
)]
struct McpOnlyFilterRow {
    #[gpui_table(filter(gpui_table::runtime::shape::TextFilter))]
    name: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct PrefixText(String);

impl gpui_table::mcp::McpToolValue for PrefixText {
    fn tool_value_schema() -> gpui_table::mcp::McpSchema {
        gpui_table::mcp::McpSchema::new(json!({
            "type": "string",
            "x-prefixText": true
        }))
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
    base = gpui_table::runtime::shape::TextFilter,
    raw_value = PrefixText,
    field = String,
    into_base = |value: PrefixText| value.0,
    from_base = PrefixText
)]
struct PrefixTextFilter;

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(
    id = "prefix_filter_rows",
    title = "Prefix Filter Rows",
    filters,
    mcp(name = "query_prefix_filter_rows")
)]
struct PrefixFilterRow {
    #[gpui_table(filter(PrefixTextFilter))]
    name: String,
}

#[derive(
    Clone,
    Debug,
    Default,
    Deserialize,
    Eq,
    gpui_table::mcp::McpJsonSchema,
    koruma::Koruma,
    PartialEq,
    Serialize,
    TableCell,
)]
#[serde(transparent)]
#[mcp(crate = gpui_table::mcp, transparent)]
#[koruma(try_new, newtype)]
struct ValidatedPrefixText(#[koruma(LenValidation::<_>::min(2).max(64))] String);

#[derive(GpuiTableFilterShape)]
#[gpui_table_filter_shape(
    base = gpui_table::runtime::shape::TextFilter,
    field = ValidatedPrefixText,
    koruma_newtype
)]
struct NewtypePrefixTextFilter;

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(
    id = "newtype_validated_filter_rows",
    title = "Newtype Validated Filter Rows",
    filters,
    mcp(name = "query_newtype_validated_filter_rows")
)]
struct NewtypeValidatedFilterRow {
    #[gpui_table(filter(NewtypePrefixTextFilter))]
    #[koruma(newtype)]
    name: ValidatedPrefixText,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(
    id = "validated_filter_rows",
    title = "Validated Filter Rows",
    filters,
    mcp(name = "query_validated_filter_rows")
)]
struct ValidatedFilterRow {
    #[gpui_table(filter(gpui_table::runtime::shape::TextFilter))]
    #[koruma(LenValidation::<_>::min(2).max(5))]
    name: String,
}

fn test_server() -> McpServer {
    McpServer::new("gpui-table-mcp-test", "0.0.0")
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
    assert_eq!(descriptor.title(), "Query users");
    assert_eq!(
        descriptor.description(),
        "Exercise explicit table MCP metadata."
    );
    let annotations = descriptor.tool_annotations();
    assert_eq!(annotations.title.as_deref(), Some("Query users"));
    assert_eq!(annotations.read_only_hint, Some(true));
    assert_eq!(annotations.destructive_hint, Some(false));
    assert_eq!(annotations.idempotent_hint, Some(true));
    assert_eq!(annotations.open_world_hint, Some(true));
}

#[test]
fn descriptor_infers_doc_description() {
    assert_eq!(
        QueryRow::descriptor().description(),
        "Query rows from inferred docs."
    );
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
}

#[test]
fn generated_server_exposes_table_descriptor_resources() {
    let server = generated_server().expect("generated tools should register");
    let uris = table_resource_uris_for::<UserRow>();

    for uri in uris.all() {
        assert!(server.contains_resource(uri));
    }
}

#[test]
fn inventory_exposes_generated_resource_definitions() {
    let uris = table_resource_uris_for::<UserRow>();
    let resources = resource_definitions().expect("resource definitions should be generated");

    assert!(
        resources
            .iter()
            .any(|resource| resource.raw.uri == uris.descriptor)
    );
    assert!(
        resources
            .iter()
            .any(|resource| resource.raw.uri == uris.schema)
    );
}

#[test]
fn manual_table_registration_exposes_table_descriptor_resources() {
    let mut server = test_server();
    table::<UserRow>(&mut server)
        .row_source(rows)
        .expect("tool should register");
    let uris = table_resource_uris_for::<UserRow>();

    for uri in uris.all() {
        assert!(server.contains_resource(uri));
    }
}

#[test]
fn manual_table_registration_reuses_existing_table_resources() {
    let mut server = test_server();
    register_table_resources::<UserRow>(&mut server).expect("table resources should register");

    table::<UserRow>(&mut server)
        .row_source(rows)
        .expect("tool should register after resources");
    let result = server.call_tool(
        &UserRow::descriptor().tool_name(),
        Some(json!({ "status": ["Active"] })),
    );

    assert_eq!(result.is_error, Some(false));
}

#[test]
fn local_rows_registry_filters_and_pages_rows() {
    let mut server = test_server();
    table::<UserRow>(&mut server)
        .row_source(rows)
        .expect("tool should register");

    let result = server.call_tool(
        &UserRow::descriptor().tool_name(),
        Some(json!({
            "name": "ann",
            "status": ["Active"],
            "age": { "min": "18", "max": 30 },
            "created_on": { "min": "2026-01-01", "max": "2026-12-31" },
            "limit": 10,
            "offset": 0
        })),
    );

    assert_eq!(result.is_error, Some(false));
    let content = result.structured_content.expect("structured result");
    assert_eq!(content["total"], 1);
    assert_eq!(content["rows"][0]["name"], "Ann");
}

#[test]
fn filter_shape_adapter_decodes_mcp_query_and_filters_rows() {
    let mut server = test_server();
    table::<PrefixFilterRow>(&mut server)
        .row_source(|| {
            Ok::<_, String>(vec![
                PrefixFilterRow {
                    name: "Ann".to_string(),
                },
                PrefixFilterRow {
                    name: "Bea".to_string(),
                },
            ])
        })
        .expect("tool should register");

    let result = server.call_tool(
        &PrefixFilterRow::descriptor().tool_name(),
        Some(json!({ "name": "ann" })),
    );

    assert_eq!(result.is_error, Some(false));
    let content = result.structured_content.expect("structured result");
    assert_eq!(content["total"], 1);
    assert_eq!(content["rows"][0]["name"], "Ann");
}

#[test]
fn koruma_filter_validation_runs_before_query_handler() {
    let mut server = test_server();
    table::<ValidatedFilterRow>(&mut server)
        .row_source(|| {
            Ok::<_, String>(vec![ValidatedFilterRow {
                name: "Ann".to_string(),
            }])
        })
        .expect("tool should register");

    let result = server.call_tool(
        &ValidatedFilterRow::descriptor().tool_name(),
        Some(json!({ "name": "a" })),
    );

    assert_eq!(result.is_error, Some(true));
    let error = result
        .structured_content
        .as_ref()
        .expect("validation should be structured")
        .get("error")
        .expect("structured validation error");
    assert_eq!(error["kind"], json!("validation"));
    let details = error["details"]
        .as_array()
        .expect("validation details should be an array");
    assert_eq!(details.len(), 1);
    assert_eq!(details[0]["scope"], json!("filter"));
    assert_eq!(details[0]["filter"], json!("name"));
    assert_eq!(details[0]["validator"], json!("LenValidation"));
    assert_eq!(details[0]["path"], json!("LenValidation"));
    assert!(
        result.content[0]
            .as_text()
            .is_some_and(|text| text.text.contains("validation failed"))
    );
}

#[test]
fn koruma_newtype_filter_validation_runs_before_query_handler() {
    let mut server = test_server();
    table::<NewtypeValidatedFilterRow>(&mut server)
        .row_source(|| {
            Ok::<_, String>(vec![NewtypeValidatedFilterRow {
                name: ValidatedPrefixText::try_from_inner("Ann".to_string())
                    .expect("fixture should be valid"),
            }])
        })
        .expect("tool should register");

    let result = server.call_tool(
        &NewtypeValidatedFilterRow::descriptor().tool_name(),
        Some(json!({ "name": "a" })),
    );

    assert_eq!(result.is_error, Some(true));
    let error = result
        .structured_content
        .as_ref()
        .expect("validation should be structured")
        .get("error")
        .expect("structured validation error");
    assert_eq!(error["kind"], json!("validation"));
    let details = error["details"]
        .as_array()
        .expect("validation details should be an array");
    assert_eq!(details.len(), 1);
    assert_eq!(details[0]["scope"], json!("filter"));
    assert_eq!(details[0]["filter"], json!("name"));
    assert_eq!(details[0]["validator"], json!("newtype"));
    assert_eq!(details[0]["path"], json!("ValidatedPrefixText"));
}

#[test]
fn local_rows_registry_keeps_total_when_page_is_limited() {
    let mut server = test_server();
    table::<UserRow>(&mut server)
        .row_source(rows)
        .expect("tool should register");

    let result = server.call_tool(
        &UserRow::descriptor().tool_name(),
        Some(json!({
            "limit": 1,
            "offset": 1
        })),
    );

    assert_eq!(result.is_error, Some(false));
    let content = result.structured_content.expect("structured result");
    assert_eq!(content["total"], 3);
    assert_eq!(content["rows"].as_array().expect("rows array").len(), 1);
    assert_eq!(content["rows"][0]["name"], "Annie");
}

#[test]
fn local_rows_registry_pages_rows_for_no_filter_tables() {
    let mut server = test_server();
    table::<NoFilterQueryRow>(&mut server)
        .row_source(no_filter_rows)
        .expect("tool should register");

    let result = server.call_tool(
        &NoFilterQueryRow::descriptor().tool_name(),
        Some(json!({
            "limit": 2,
            "offset": 1,
        })),
    );

    assert_eq!(result.is_error, Some(false));
    let content = result.structured_content.expect("structured result");
    assert_eq!(content["total"], 3);
    assert_eq!(content["rows"].as_array().expect("rows array").len(), 2);
    assert_eq!(content["rows"][0]["name"], "Annie");
}

#[test]
fn invalid_facet_value_is_tool_error() {
    let mut server = test_server();
    table::<UserRow>(&mut server)
        .row_source(rows)
        .expect("tool should register");

    let result = server.call_tool(
        &UserRow::descriptor().tool_name(),
        Some(json!({
            "status": ["Missing"]
        })),
    );

    assert_eq!(result.is_error, Some(true));
    assert!(matches!(
        result.content[0].as_text(),
        Some(text) if text.text.contains("unknown value")
    ));
}

#[test]
fn negative_pagination_limit_is_tool_error() {
    let mut server = test_server();
    table::<UserRow>(&mut server)
        .row_source(rows)
        .expect("tool should register");

    let result = server.call_tool(
        &UserRow::descriptor().tool_name(),
        Some(json!({
            "limit": -1
        })),
    );

    assert_eq!(result.is_error, Some(true));
    assert!(matches!(
        result.content[0].as_text(),
        Some(text) if text.text.contains("failed to decode field `limit`")
    ));
}

#[test]
fn null_pagination_limit_is_tool_error() {
    let mut server = test_server();
    table::<UserRow>(&mut server)
        .row_source(rows)
        .expect("tool should register");

    let result = server.call_tool(
        &UserRow::descriptor().tool_name(),
        Some(json!({
            "limit": null
        })),
    );

    assert_eq!(result.is_error, Some(true));
    assert!(matches!(
        result.content[0].as_text(),
        Some(text) if text.text.contains("does not accept null")
    ));
}

#[test]
fn range_filters_reject_unknown_bound_fields() {
    let mut server = test_server();
    table::<UserRow>(&mut server)
        .row_source(rows)
        .expect("tool should register");

    let result = server.call_tool(
        &UserRow::descriptor().tool_name(),
        Some(json!({
            "age": { "min": 18, "step": 2 }
        })),
    );

    assert_eq!(result.is_error, Some(true));
    assert!(matches!(
        result.content[0].as_text(),
        Some(text) if text.text.contains("unknown field")
    ));
}

#[test]
fn duplicate_table_registration_returns_setup_error() {
    let mut server = test_server();
    table::<UserRow>(&mut server)
        .row_source(rows)
        .expect("first registration should succeed");

    let error = match table::<UserRow>(&mut server).row_source(rows) {
        Ok(_) => panic!("duplicate tool should fail registration"),
        Err(error) => error,
    };

    assert_eq!(
        error,
        McpToolError::DuplicateTool {
            name: UserRow::descriptor().tool_name()
        }
    );
}

#[test]
fn attribute_local_source_handler_registers_with_inventory_registry() {
    let server = generated_server().expect("generated tools should register");

    let result = server.call_tool(
        &UserRow::descriptor().tool_name(),
        Some(json!({
            "status": ["Blocked"]
        })),
    );

    assert_eq!(result.is_error, Some(false));
    let content = result.structured_content.expect("structured result");
    assert_eq!(content["total"], 1);
    assert_eq!(content["rows"][0]["name"], "Bea");
}

#[test]
fn attribute_query_handler_registers_with_inventory_registry() {
    let server = generated_server().expect("generated tools should register");

    let result = server.call_tool(
        &QueryRow::descriptor().tool_name(),
        Some(json!({
            "name": "ignored-by-handler",
            "limit": 1,
            "offset": 0
        })),
    );

    assert_eq!(result.is_error, Some(false));
    let content = result.structured_content.expect("structured result");
    assert_eq!(content["total"], 1);
    assert_eq!(content["rows"][0]["name"], "custom query");

    let result = server.call_tool(
        &DirectQueryRow::descriptor().tool_name(),
        Some(json!({
            "name": "ignored-by-handler"
        })),
    );

    assert_eq!(result.is_error, Some(false));
    let content = result.structured_content.expect("structured result");
    assert_eq!(content["total"], 1);
    assert_eq!(content["rows"][0]["name"], "direct query");

    let result = server.call_tool(
        &AsyncQueryRow::descriptor().tool_name(),
        Some(json!({
            "name": "ignored-by-handler"
        })),
    );

    assert_eq!(result.is_error, Some(false));
    let content = result.structured_content.expect("structured result");
    assert_eq!(content["total"], 1);
    assert_eq!(content["rows"][0]["name"], "async query");

    let result = server.call_tool(
        &AsyncValueQueryRow::descriptor().tool_name(),
        Some(json!({
            "name": "ignored-by-handler"
        })),
    );

    assert_eq!(result.is_error, Some(false));
    let content = result.structured_content.expect("structured result");
    assert_eq!(content["total"], 1);
    assert_eq!(content["rows"][0]["name"], "async direct query");

    let result = server.call_tool(
        &AsyncSourceRow::descriptor().tool_name(),
        Some(json!({
            "name": "source"
        })),
    );

    assert_eq!(result.is_error, Some(false));
    let content = result.structured_content.expect("structured result");
    assert_eq!(content["total"], 1);
    assert_eq!(content["rows"][0]["name"], "async source row");

    let result = server.call_tool(
        &ResultSourceRow::descriptor().tool_name(),
        Some(json!({
            "name": "result"
        })),
    );

    assert_eq!(result.is_error, Some(false));
    let content = result.structured_content.expect("structured result");
    assert_eq!(content["total"], 1);
    assert_eq!(content["rows"][0]["name"], "result source row");

    let result = server.call_tool(
        &AsyncResultSourceRow::descriptor().tool_name(),
        Some(json!({
            "name": "async result"
        })),
    );

    assert_eq!(result.is_error, Some(false));
    let content = result.structured_content.expect("structured result");
    assert_eq!(content["total"], 1);
    assert_eq!(content["rows"][0]["name"], "async result source row");

    let result = server.call_tool(
        &McpOnlyFilterRow::descriptor().tool_name(),
        Some(json!({
            "name": "mcp"
        })),
    );

    assert_eq!(result.is_error, Some(false));
    let content = result.structured_content.expect("structured result");
    assert_eq!(content["total"], 1);
    assert_eq!(content["rows"][0]["name"], "mcp-only filter row");
}

#[gpui_table::mcp_query]
fn rows() -> Result<Vec<UserRow>, String> {
    Ok(vec![
        UserRow {
            name: "Ann".to_string(),
            status: UserStatus::Active,
            age: 24,
            created_on: date(2026, 6, 1),
        },
        UserRow {
            name: "Annie".to_string(),
            status: UserStatus::Active,
            age: 34,
            created_on: date(2026, 5, 20),
        },
        UserRow {
            name: "Bea".to_string(),
            status: UserStatus::Blocked,
            age: 41,
            created_on: date(2025, 12, 15),
        },
    ])
}

#[gpui_table::mcp_query]
fn no_filter_rows() -> Result<Vec<NoFilterQueryRow>, String> {
    Ok(vec![
        NoFilterQueryRow {
            name: "Ann".to_string(),
        },
        NoFilterQueryRow {
            name: "Annie".to_string(),
        },
        NoFilterQueryRow {
            name: "Bea".to_string(),
        },
    ])
}

#[gpui_table::mcp_query]
fn query_rows(
    query: gpui_table::mcp::TableQuery<QueryRow>,
) -> Result<gpui_table::mcp::TableQueryResult<QueryRow>, String> {
    Ok(query.result(
        vec![QueryRow {
            name: "custom query".to_string(),
        }],
        1,
    ))
}

#[gpui_table::mcp_query]
fn direct_query_rows(
    query: gpui_table::mcp::TableQuery<DirectQueryRow>,
) -> Result<gpui_table::mcp::TableQueryResult<DirectQueryRow>, String> {
    Ok(query.result(
        vec![DirectQueryRow {
            name: "direct query".to_string(),
        }],
        1,
    ))
}

#[gpui_table::mcp_query]
async fn async_query_rows(
    query: gpui_table::mcp::TableQuery<AsyncQueryRow>,
) -> Result<gpui_table::mcp::TableQueryResult<AsyncQueryRow>, String> {
    Ok(query.result(
        vec![AsyncQueryRow {
            name: "async query".to_string(),
        }],
        1,
    ))
}

#[gpui_table::mcp_query]
async fn async_value_query_rows(
    query: gpui_table::mcp::TableQuery<AsyncValueQueryRow>,
) -> Result<gpui_table::mcp::TableQueryResult<AsyncValueQueryRow>, String> {
    Ok(query.result(
        vec![AsyncValueQueryRow {
            name: "async direct query".to_string(),
        }],
        1,
    ))
}

#[gpui_table::mcp_query]
async fn async_source_rows() -> Result<Vec<AsyncSourceRow>, String> {
    Ok(vec![AsyncSourceRow {
        name: "async source row".to_string(),
    }])
}

#[gpui_table::mcp_query]
fn result_source_rows() -> Result<Vec<ResultSourceRow>, String> {
    Ok(vec![ResultSourceRow {
        name: "result source row".to_string(),
    }])
}

#[gpui_table::mcp_query]
async fn async_result_source_rows() -> Result<Vec<AsyncResultSourceRow>, String> {
    Ok(vec![AsyncResultSourceRow {
        name: "async result source row".to_string(),
    }])
}

#[gpui_table::mcp_query]
fn mcp_only_filter_rows() -> Vec<McpOnlyFilterRow> {
    vec![
        McpOnlyFilterRow {
            name: "mcp-only filter row".to_string(),
        },
        McpOnlyFilterRow {
            name: "other row".to_string(),
        },
    ]
}

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("test date should be valid")
}
