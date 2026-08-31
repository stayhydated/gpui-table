use gpui_table::mcp::{McpTable as _, serde_json::json, table};

use super::{
    NewtypeValidatedFilterRow, PrefixFilterRow, UserRow, ValidatedFilterRow,
    newtype_validated_filter_rows, prefix_filter_rows, test_server, user_rows,
    validated_filter_rows,
};

#[test]
fn local_rows_registry_filters_and_pages_rows() {
    let mut server = test_server();
    table::<UserRow>(&mut server)
        .row_source(user_rows)
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
        .row_source(prefix_filter_rows)
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
        .row_source(validated_filter_rows)
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
        .row_source(newtype_validated_filter_rows)
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
