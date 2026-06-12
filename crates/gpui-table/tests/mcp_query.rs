#![cfg(all(feature = "mcp", feature = "rust_decimal"))]

use chrono::NaiveDate;
use gpui_table::{
    Filterable, GpuiTable, TableCell,
    mcp::{
        McpServer, McpTable as _, McpToolError, serde_json::json, server as generated_server,
        table, tool_definitions,
    },
};
use serde::Serialize;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Filterable, TableCell)]
enum UserStatus {
    Active,
    Blocked,
}

#[derive(gpui_table::mcp::McpJsonSchema)]
#[allow(dead_code)]
#[mcp(crate = gpui_table::mcp)]
#[serde(rename_all = "snake_case")]
enum ExportMode {
    Summary,
    FullDetail,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(
    id = "users",
    title = "Users",
    filters,
    mcp(
        name = "query_users",
        title = "Query users",
        description = "Exercise explicit table MCP metadata."
    )
)]
struct UserRow {
    #[gpui_table(filter)]
    name: String,

    #[gpui_table(filter)]
    status: UserStatus,

    #[gpui_table(filter)]
    age: u8,

    #[gpui_table(filter)]
    created_on: NaiveDate,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(id = "query_rows", title = "Query Rows", filters, mcp)]
struct QueryRow {
    #[gpui_table(filter)]
    name: String,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(id = "direct_query_rows", title = "Direct Query Rows", filters, mcp)]
struct DirectQueryRow {
    #[gpui_table(filter)]
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
    #[gpui_table(filter)]
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
    #[gpui_table(filter)]
    name: String,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(id = "async_source_rows", title = "Async Source Rows", filters, mcp)]
struct AsyncSourceRow {
    #[gpui_table(filter)]
    name: String,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(id = "result_source_rows", title = "Result Source Rows", filters, mcp)]
struct ResultSourceRow {
    #[gpui_table(filter)]
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
    #[gpui_table(filter)]
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
}

#[test]
fn inventory_exposes_generated_tool_definition() {
    let expected = UserRow::descriptor().tool_name();
    let tools = tool_definitions().expect("tool definitions should be generated");

    assert!(tools.iter().any(|tool| tool.name == expected));
}

#[test]
fn inventory_exposes_pagination_only_mcp_tool_definition() {
    let expected = NoFilterQueryRow::descriptor().tool_name();
    let tools = tool_definitions().expect("tool definitions should be generated");

    assert!(tools.iter().any(|tool| tool.name == expected));
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
        Some(text) if text.text.contains("expected a non-negative integer")
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

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("test date should be valid")
}
