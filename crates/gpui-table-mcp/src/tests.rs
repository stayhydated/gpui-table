use super::{
    McpInput, McpRange, McpTableDescriptor, McpTableFilter, McpTableQueryInput, McpToolInput as _,
    default_filter_shape_input_schema, register_table_prompt_templates_for_descriptor,
    register_table_resources_for_descriptor, table_descriptor_resource_value, table_prompt_names,
    table_prompt_text, table_query_output_schema, table_query_output_schema_for_row,
    table_resource_uris, tool_name,
};
use gpui_table_runtime::shape::GpuiTableFilterShape;
use gpui_table_schema::registry::{RegistryFilterType, RustPath, RustType};

struct SchemaRow;

impl super::McpJsonSchema for SchemaRow {
    fn json_schema() -> super::McpSchema {
        let mut properties = super::McpSchemaProperties::new();
        properties.insert("name".to_string(), super::McpSchema::string());
        super::object_schema(properties, ["name"])
    }
}

fn schema_row_schema() -> super::McpSchema {
    <SchemaRow as super::McpJsonSchema>::json_schema()
}

#[test]
fn tool_names_are_stable_and_mcp_friendly() {
    assert_eq!(
        tool_name("example::tables", "UserTable"),
        "example_tables_user_table"
    );
    assert_eq!(tool_name("123", ""), "table_123");
}

#[test]
fn input_schema_maps_filter_types() {
    const FILTERS: &[McpTableFilter] = &[
        McpTableFilter::new(
            "name",
            RustType::from_macro_tokens_unchecked("String"),
            RegistryFilterType::Text,
        ),
        McpTableFilter::new(
            "status",
            RustType::from_macro_tokens_unchecked("crate::Status"),
            RegistryFilterType::Faceted,
        ),
    ];

    let schema = McpTableDescriptor::new(
        "User",
        "user",
        "Users",
        RustPath::from_macro_tokens_unchecked("tests"),
        FILTERS,
        component_shape_mcp::McpToolMetadata::new(),
    )
    .input_schema();

    assert_eq!(schema["properties"]["name"]["type"], "string");
    assert_eq!(schema["properties"]["status"]["type"], "array");
    assert_eq!(schema["properties"]["limit"]["type"], "integer");
}

#[test]
fn table_query_output_schema_uses_generic_row_items_by_default() {
    let schema = table_query_output_schema(None);

    assert_eq!(schema["type"], "object");
    assert_eq!(schema["properties"]["rows"]["type"], "array");
    assert_eq!(schema["properties"]["rows"]["items"], serde_json::json!({}));
}

#[test]
fn table_query_output_schema_uses_row_schema_when_available() {
    let schema = table_query_output_schema_for_row::<SchemaRow>();

    assert_eq!(schema["type"], "object");
    assert_eq!(
        schema["properties"]["rows"]["items"]["properties"]["name"]["type"],
        "string"
    );
    assert_eq!(
        schema["properties"]["rows"]["items"]["additionalProperties"],
        false
    );
}

#[test]
fn table_descriptor_resource_value_publishes_filter_metadata() {
    const FILTERS: &[McpTableFilter] = &[
        McpTableFilter::new(
            "name",
            RustType::from_macro_tokens_unchecked("String"),
            RegistryFilterType::Text,
        ),
        McpTableFilter::new(
            "status",
            RustType::from_macro_tokens_unchecked("crate::Status"),
            RegistryFilterType::Faceted,
        ),
    ];

    let descriptor = McpTableDescriptor::new(
        "UserRow",
        "users",
        "Users",
        RustPath::from_macro_tokens_unchecked("example::tables"),
        FILTERS,
        component_shape_mcp::McpToolMetadata::new(),
    );
    let value = table_descriptor_resource_value(descriptor);

    assert_eq!(value["table_name"], serde_json::json!("UserRow"));
    assert_eq!(
        value["tool_name"],
        serde_json::json!("example_tables_users")
    );
    assert_eq!(
        value["resources"]["descriptor"],
        serde_json::json!("gpui-table://tables/example_tables_users/descriptor")
    );
    assert_eq!(value["filters"][0]["name"], serde_json::json!("name"));
    assert_eq!(
        value["filters"][0]["mcp_input"],
        serde_json::json!({
            "supported": true,
            "shape": "scalar",
            "primitive": "string",
        })
    );
    assert_eq!(
        value["filters"][1]["filter_type"],
        serde_json::json!("faceted")
    );
    assert_eq!(
        value["filters"][1]["mcp_input"],
        serde_json::json!({
            "supported": true,
            "shape": "set",
            "items": "string",
        })
    );
    assert_eq!(value["filters"][1]["schema"]["uniqueItems"], true);
}

#[test]
fn table_descriptor_resource_value_publishes_output_schema() {
    let descriptor = McpTableDescriptor::new(
        "UserRow",
        "users",
        "Users",
        RustPath::from_macro_tokens_unchecked("example::tables"),
        &[],
        component_shape_mcp::McpToolMetadata::new(),
    )
    .with_row_schema(schema_row_schema);

    let value = table_descriptor_resource_value(descriptor);

    assert_eq!(
        value["row_schema"]["properties"]["name"]["type"],
        serde_json::json!("string")
    );
    assert_eq!(
        value["output_schema"]["properties"]["rows"]["items"]["properties"]["name"]["type"],
        serde_json::json!("string")
    );
    assert_eq!(value["output_schema"]["type"], serde_json::json!("object"));
}

#[test]
fn table_resources_register_descriptor_and_schema() {
    const FILTERS: &[McpTableFilter] = &[McpTableFilter::new(
        "name",
        RustType::from_macro_tokens_unchecked("String"),
        RegistryFilterType::Text,
    )];
    let descriptor = McpTableDescriptor::new(
        "UserRow",
        "users",
        "Users",
        RustPath::from_macro_tokens_unchecked("example::tables"),
        FILTERS,
        component_shape_mcp::McpToolMetadata::new(),
    );
    let uris = table_resource_uris(descriptor);
    let mut server = super::McpServer::new("test", "0.0.0");

    register_table_resources_for_descriptor(&mut server, descriptor)
        .expect("table resources should register");

    assert!(server.contains_resource(&uris.descriptor));
    assert!(server.contains_resource(&uris.schema));
    assert_eq!(uris.all(), [uris.descriptor.as_str(), uris.schema.as_str()]);
    assert_eq!(server.resource_count(), 2);
    assert_eq!(
        register_table_resources_for_descriptor(&mut server, descriptor)
            .expect_err("duplicate table resources should fail"),
        super::McpToolError::duplicate_resource(uris.descriptor)
    );
}

#[test]
fn table_prompt_templates_describe_generated_query_workflow() {
    const FILTERS: &[McpTableFilter] = &[McpTableFilter::new(
        "name",
        RustType::from_macro_tokens_unchecked("String"),
        RegistryFilterType::Text,
    )];
    let descriptor = McpTableDescriptor::new(
        "UserRow",
        "users",
        "Users",
        RustPath::from_macro_tokens_unchecked("example::tables"),
        FILTERS,
        component_shape_mcp::McpToolMetadata::new(),
    );
    let prompt_names = table_prompt_names(descriptor);
    let uris = table_resource_uris(descriptor);

    assert_eq!(prompt_names.query, "query_example_tables_users_table");
    assert_eq!(prompt_names.all(), [prompt_names.query.as_str()]);

    let mut server = super::McpServer::new("test", "0.0.0");
    register_table_prompt_templates_for_descriptor(&mut server, descriptor)
        .expect("prompt template should register");

    assert_eq!(server.resource_count(), 2);
    assert!(server.contains_resource(&uris.descriptor));
    assert!(server.contains_resource(&uris.schema));
    assert_eq!(server.prompt_count(), 1);
    assert!(server.contains_prompt(&prompt_names.query));

    let prompts = server.list_prompts();
    let prompt = prompts
        .iter()
        .find(|prompt| prompt.name == prompt_names.query)
        .expect("query prompt should be listed");
    assert_eq!(prompt.title.as_deref(), Some("Query Users"));
    assert_eq!(
        prompt.description.as_deref(),
        Some("Draft filter and pagination arguments for Users query.")
    );
    assert_eq!(
        prompt
            .arguments
            .as_ref()
            .expect("arguments should exist")
            .len(),
        2
    );

    let mut arguments = serde_json::Map::new();
    arguments.insert("goal".to_string(), serde_json::json!("active users"));
    let prompt_text = table_prompt_text(descriptor, Some(arguments));
    assert!(prompt_text.contains("MCP tool `example_tables_users`"));
    assert!(prompt_text.contains(&uris.descriptor));
    assert!(prompt_text.contains(&uris.schema));
    assert!(prompt_text.contains("active users"));
    assert_eq!(
        register_table_prompt_templates_for_descriptor(&mut server, descriptor)
            .expect_err("duplicate table prompts should fail"),
        super::McpToolError::duplicate_prompt(prompt_names.query)
    );
}

struct RawRangeShape;

impl super::ComponentShapeMetadata for RawRangeShape {
    const MCP_INPUT: McpInput = McpInput::string();
}

impl GpuiTableFilterShape for RawRangeShape {
    type Component = ();
    type RawValue = McpRange<u32>;
    type FilterValue = McpRange<u32>;

    const FILTER_TYPE: RegistryFilterType = RegistryFilterType::Text;

    fn new_for(
        _title: impl Fn(&gpui_kit::App) -> String + 'static,
        _value: Self::RawValue,
        _on_change: impl Fn(Self::RawValue, &mut gpui_kit::Window, &mut gpui_kit::App) + 'static,
        _cx: &mut gpui_kit::App,
    ) -> gpui_kit::Entity<Self::Component> {
        unimplemented!("schema test does not instantiate GPUI state")
    }

    fn read_value(
        _entity: &gpui_kit::Entity<Self::Component>,
        _cx: &gpui_kit::App,
    ) -> Self::RawValue {
        unimplemented!("schema test does not read GPUI state")
    }

    fn wrap_value(value: Self::RawValue) -> Self::FilterValue {
        value
    }

    fn reset_silent(
        _entity: &gpui_kit::Entity<Self::Component>,
        _window: &mut gpui_kit::Window,
        _cx: &mut gpui_kit::App,
    ) {
    }
}

impl super::McpFilterShape for RawRangeShape {
    fn input_schema(filter: McpTableFilter) -> super::McpSchema {
        default_filter_shape_input_schema::<Self>(filter)
    }

    fn decode_filter(
        field: &'static str,
        value: super::McpAny,
    ) -> Result<Self::FilterValue, super::McpToolError> {
        super::decode_raw_filter_shape::<Self>(field, value)
    }
}

#[test]
fn derived_filter_shape_schema_uses_raw_value_schema() {
    let filter = McpTableFilter::for_shape::<RawRangeShape>(
        "window",
        RustType::from_macro_tokens_unchecked("u32"),
    );

    let schema = default_filter_shape_input_schema::<RawRangeShape>(filter);

    assert_eq!(schema["type"], "object");
    assert_eq!(schema["properties"]["min"]["anyOf"][0]["type"], "integer");
}

#[derive(Clone, Debug, Default, PartialEq)]
struct EmptyFilters;

#[derive(Clone, super::Serialize)]
struct TypedTable;

impl gpui_table_core::filter::Matchable<EmptyFilters> for TypedTable {
    fn matches_filters(&self, _filters: &EmptyFilters) -> bool {
        true
    }
}

impl super::McpTable for TypedTable {
    type FilterValues = EmptyFilters;

    fn descriptor() -> McpTableDescriptor {
        McpTableDescriptor::new(
            "TypedTable",
            "typed_table",
            "Typed Table",
            RustPath::from_macro_tokens_unchecked("tests"),
            &[],
            component_shape_mcp::McpToolMetadata::new(),
        )
        .with_row_schema(schema_row_schema)
    }

    fn decode_query(
        call: super::McpToolCall,
    ) -> Result<super::TableQuery<Self>, super::McpToolError> {
        let mut arguments = call.into_arguments();
        let limit = arguments.take_present_tool_value::<usize>("limit")?;
        let offset = arguments
            .take_present_tool_value::<usize>("offset")?
            .unwrap_or(0);
        arguments.finish()?;

        Ok(super::TableQuery {
            filters: EmptyFilters,
            limit,
            offset,
        })
    }
}

#[test]
fn table_query_input_pairs_typed_row_output_schema_with_tool_definition() {
    let tool = McpTableQueryInput::<TypedTable>::tool_definition().expect("tool should build");
    fn assert_typed_tool(_tool: &super::McpTypedTool<super::McpTableQueryInput<TypedTable>>) {}
    assert_typed_tool(&tool);
    assert_eq!(tool.input_schema["properties"]["limit"]["type"], "integer");
    assert_eq!(
        tool.output_schema.as_ref().unwrap()["properties"]["rows"]["type"],
        "array"
    );
    assert_eq!(
        tool.output_schema.as_ref().unwrap()["properties"]["rows"]["items"]["properties"]["name"]["type"],
        "string"
    );
    let annotations = tool
        .annotations
        .as_ref()
        .expect("table query tool should publish annotations");
    assert_eq!(annotations.title.as_deref(), Some("Typed Table query"));
    assert_eq!(annotations.read_only_hint, Some(true));
    assert_eq!(annotations.destructive_hint, Some(false));
    assert_eq!(annotations.idempotent_hint, Some(true));
    assert_eq!(annotations.open_world_hint, None);

    let input = McpTableQueryInput::<TypedTable>::from_tool_call(
        super::McpToolCall::from_value(Some(serde_json::json!({
            "limit": 10,
            "offset": 5
        })))
        .expect("arguments should normalize"),
    )
    .expect("query input should decode");
    let query = input.into_query();

    assert_eq!(query.limit, Some(10));
    assert_eq!(query.offset, 5);
    assert_eq!(query.filters, EmptyFilters);
}

#[test]
fn typed_table_helpers_cover_prompt_resource_and_named_server_entry_points() {
    let names = super::table_prompt_names_for::<TypedTable>();
    assert_eq!(names.query, "query_tests_typed_table_table");

    let mut server = super::McpServer::new("test", "0.0.0");
    super::register_table_resources::<TypedTable>(&mut server).unwrap();
    assert_eq!(server.resource_count(), 2);

    super::register_table_prompt_templates::<TypedTable>(&mut server).unwrap();
    assert_eq!(server.prompt_count(), 1);

    let named = super::server_named("custom-table-server", "1.2.3").unwrap();
    assert_eq!(named.tool_count(), 0);
    let server_info = rmcp::ServerHandler::get_info(&named);
    assert_eq!(
        server_info.protocol_version,
        rmcp::model::ProtocolVersion::V_2026_07_28
    );
    assert_eq!(server_info.server_info.name, "custom-table-server");
    assert!(super::server().is_ok());

    let mut inventory_server = super::McpServer::new("inventory", "0.0.0");
    super::register_prompt_templates(&mut inventory_server).unwrap();
}

#[test]
fn table_tool_registration_covers_sync_async_rows_and_sources() {
    let mut rows_server = super::McpServer::new("rows", "0.0.0");
    super::table::<TypedTable>(&mut rows_server)
        .rows(vec![TypedTable])
        .unwrap();
    assert_eq!(rows_server.tool_count(), 1);
    assert!(
        super::table::<TypedTable>(&mut rows_server)
            .rows(vec![])
            .is_err()
    );

    let mut query_server = super::McpServer::new("query", "0.0.0");
    super::table::<TypedTable>(&mut query_server)
        .query(|query| {
            Ok::<_, String>(super::TableQueryResult {
                rows: vec![TypedTable],
                total: 1,
                offset: query.offset,
                limit: query.limit,
            })
        })
        .unwrap();

    let mut async_server = super::McpServer::new("async", "0.0.0");
    super::table::<TypedTable>(&mut async_server)
        .query_async(|query| async move {
            Ok::<_, String>(super::TableQueryResult {
                rows: Vec::new(),
                total: 0,
                offset: query.offset,
                limit: query.limit,
            })
        })
        .unwrap();

    let mut source_server = super::McpServer::new("source", "0.0.0");
    super::table::<TypedTable>(&mut source_server)
        .row_source(|| Ok::<_, String>(vec![TypedTable]))
        .unwrap();

    let mut async_source_server = super::McpServer::new("async-source", "0.0.0");
    super::table::<TypedTable>(&mut async_source_server)
        .row_source_async(|| async { Ok::<_, String>(vec![TypedTable]) })
        .unwrap();
}

#[test]
fn registry_wrappers_invoke_descriptor_and_registration_functions() {
    fn register_noop(_server: &mut super::McpServer) -> Result<(), super::McpToolError> {
        Ok(())
    }

    let table =
        super::registry::McpTableRegistration::new(<TypedTable as super::McpTable>::descriptor);
    assert_eq!(table.descriptor().table_name(), "TypedTable");

    let query = super::registry::McpQueryHandlerRegistration::new(register_noop);
    let mut server = super::McpServer::new("test", "0.0.0");
    query.register(&mut server).unwrap();
}

#[test]
fn raw_filter_shape_decode_and_prompt_helpers_cover_private_composition() {
    use super::McpFilterShape as _;

    let filter = McpTableFilter::for_shape::<RawRangeShape>(
        "window",
        RustType::from_macro_tokens_unchecked("u32"),
    );
    assert_eq!(RawRangeShape::input_schema(filter)["type"], "object");
    let decoded = RawRangeShape::decode_filter(
        "window",
        super::McpAny::from(serde_json::json!({ "min": 1, "max": 3 })),
    )
    .unwrap();
    assert_eq!(decoded.into_tuple(), (Some(1), Some(3)));

    let descriptor = <TypedTable as super::McpTable>::descriptor();
    let prompt = super::table_prompt_result(descriptor, None);
    assert!(
        prompt
            .description
            .as_deref()
            .is_some_and(|value| value.contains("Query Typed Table"))
    );

    let mut seen = std::collections::BTreeSet::new();
    let mut specs = Vec::new();
    super::push_descriptor_prompt_specs(&mut seen, &mut specs, descriptor).unwrap();
    super::push_descriptor_prompt_specs(&mut seen, &mut specs, descriptor).unwrap();
    assert_eq!(specs.len(), 1);
}
