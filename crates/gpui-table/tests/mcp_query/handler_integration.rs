use gpui_table::{
    GpuiTable,
    mcp::{McpTable as _, TableQuery, TableQueryResult, serde_json::json, server},
};
use serde::Serialize;

use super::{NoFilterQueryRow, UserRow, no_filter_rows as fixture_no_filter_rows, user_rows};

/// Query rows from inferred docs.
#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(id = "query_rows", title = "Query Rows", filters, mcp)]
struct QueryRow {
    #[gpui_table(filter(gpui_table_component::TextFilter))]
    name: String,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(id = "direct_query_rows", title = "Direct Query Rows", filters, mcp)]
struct DirectQueryRow {
    #[gpui_table(filter(gpui_table_component::TextFilter))]
    name: String,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(id = "async_query_rows", title = "Async Query Rows", filters, mcp)]
struct AsyncQueryRow {
    #[gpui_table(filter(gpui_table_component::TextFilter))]
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
    #[gpui_table(filter(gpui_table_component::TextFilter))]
    name: String,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(id = "async_source_rows", title = "Async Source Rows", filters, mcp)]
struct AsyncSourceRow {
    #[gpui_table(filter(gpui_table_component::TextFilter))]
    name: String,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(id = "result_source_rows", title = "Result Source Rows", filters, mcp)]
struct ResultSourceRow {
    #[gpui_table(filter(gpui_table_component::TextFilter))]
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
    #[gpui_table(filter(gpui_table_component::TextFilter))]
    name: String,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(
    id = "mcp_only_filter_rows",
    title = "MCP Only Filter Rows",
    mcp(name = "query_mcp_only_filter_rows")
)]
struct McpOnlyFilterRow {
    #[gpui_table(filter(gpui_table_component::TextFilter))]
    name: String,
}

#[test]
fn attribute_local_source_handler_registers_with_inventory_registry() {
    let server = server().expect("generated tools should register");

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
fn descriptor_infers_doc_description() {
    assert_eq!(
        QueryRow::descriptor().description(),
        "Query rows from inferred docs."
    );
}

#[test]
fn attribute_query_handler_registers_with_inventory_registry() {
    let server = server().expect("generated tools should register");

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
    user_rows()
}

#[gpui_table::mcp_query]
fn no_filter_rows() -> Result<Vec<NoFilterQueryRow>, String> {
    fixture_no_filter_rows()
}

#[gpui_table::mcp_query]
fn query_rows(query: TableQuery<QueryRow>) -> Result<TableQueryResult<QueryRow>, String> {
    Ok(query.result(
        vec![QueryRow {
            name: "custom query".to_string(),
        }],
        1,
    ))
}

#[gpui_table::mcp_query]
fn direct_query_rows(
    query: TableQuery<DirectQueryRow>,
) -> Result<TableQueryResult<DirectQueryRow>, String> {
    Ok(query.result(
        vec![DirectQueryRow {
            name: "direct query".to_string(),
        }],
        1,
    ))
}

#[gpui_table::mcp_query]
async fn async_query_rows(
    query: TableQuery<AsyncQueryRow>,
) -> Result<TableQueryResult<AsyncQueryRow>, String> {
    Ok(query.result(
        vec![AsyncQueryRow {
            name: "async query".to_string(),
        }],
        1,
    ))
}

#[gpui_table::mcp_query]
async fn async_value_query_rows(
    query: TableQuery<AsyncValueQueryRow>,
) -> Result<TableQueryResult<AsyncValueQueryRow>, String> {
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
