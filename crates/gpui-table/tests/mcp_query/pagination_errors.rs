use gpui_table::mcp::{McpTable as _, serde_json::json, table};

use super::{NoFilterQueryRow, UserRow, no_filter_rows, test_server, user_rows};

#[test]
fn local_rows_registry_keeps_total_when_page_is_limited() {
    let mut server = test_server();
    table::<UserRow>(&mut server)
        .row_source(user_rows)
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
        .row_source(user_rows)
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
        .row_source(user_rows)
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
        .row_source(user_rows)
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
        .row_source(user_rows)
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
