use gpui_table::mcp::{
    McpTable as _, McpToolError, register_table_resources, resource_definitions, serde_json::json,
    server as generated_server, table, table_resource_uris_for,
};

use super::{UserRow, test_server, user_rows};

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
            .any(|resource| resource.uri == uris.descriptor)
    );
    assert!(resources.iter().any(|resource| resource.uri == uris.schema));
}

#[test]
fn manual_table_registration_exposes_table_descriptor_resources() {
    let mut server = test_server();
    table::<UserRow>(&mut server)
        .row_source(user_rows)
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
        .row_source(user_rows)
        .expect("tool should register after resources");
    let result = server.call_tool(
        &UserRow::descriptor().tool_name(),
        Some(json!({ "status": ["Active"] })),
    );

    assert_eq!(result.is_error, Some(false));
}

#[test]
fn duplicate_table_registration_returns_setup_error() {
    let mut server = test_server();
    table::<UserRow>(&mut server)
        .row_source(user_rows)
        .expect("first registration should succeed");

    let error = match table::<UserRow>(&mut server).row_source(user_rows) {
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
