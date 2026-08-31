use super::*;

/// MCP resource URIs generated for one table descriptor.
///
/// The `{tool_name}` segment is the table query tool name, including any
/// struct-level `#[gpui_table(mcp(name = "..."))]` override.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpTableResourceUris {
    /// JSON resource with table, tool, filter, validation, and per-filter schema metadata.
    pub descriptor: String,
    /// JSON Schema resource for the generated table query input.
    pub schema: String,
}

impl McpTableResourceUris {
    /// Return all generated resource URIs in descriptor, schema order.
    pub fn all(&self) -> [&str; 2] {
        [self.descriptor.as_str(), self.schema.as_str()]
    }
}

/// Return the MCP resource URIs generated for `descriptor`.
pub fn table_resource_uris(descriptor: McpTableDescriptor) -> McpTableResourceUris {
    let base = format!("gpui-table://tables/{}", descriptor.tool_name());
    McpTableResourceUris {
        descriptor: format!("{base}/descriptor"),
        schema: format!("{base}/schema"),
    }
}

/// Return the MCP resource URIs generated for `Table`.
pub fn table_resource_uris_for<Table>() -> McpTableResourceUris
where
    Table: McpTable,
{
    table_resource_uris(Table::descriptor())
}

/// Build the JSON value served by a table's descriptor resource.
///
/// The descriptor includes resource links, table/tool metadata, filter field
/// types, normalized MCP input metadata, validation rules, and per-filter
/// schemas.
pub fn table_descriptor_resource_value(descriptor: McpTableDescriptor) -> Value {
    let resources = table_resource_uris(descriptor);
    let row_schema = descriptor.row_schema();
    let output_schema = table_query_output_schema(row_schema.clone()).into_value();
    let row_schema = row_schema.map(McpSchema::into_value).unwrap_or(Value::Null);
    json!({
        "table_name": descriptor.table_name(),
        "table_id": descriptor.table_id(),
        "table_title": descriptor.table_title(),
        "source_module_path": descriptor.source_module_path().as_str(),
        "tool_name": descriptor.tool_name(),
        "title": descriptor.title(),
        "description": descriptor.description(),
        "resources": {
            "descriptor": resources.descriptor,
            "schema": resources.schema,
        },
        "output_schema": output_schema,
        "row_schema": row_schema,
        "filters": descriptor
            .filters()
            .iter()
            .map(|filter| table_filter_descriptor_value(*filter))
            .collect::<Vec<_>>(),
    })
}

/// Build the JSON Schema value served by a table's schema resource.
pub fn table_schema_resource_value(descriptor: McpTableDescriptor) -> Value {
    descriptor.input_schema().into_value()
}

pub(super) fn table_filter_descriptor_value(filter: McpTableFilter) -> Value {
    let mut object = serde_json::Map::new();
    object.insert("name".to_string(), Value::String(filter.name().to_string()));
    object.insert(
        "field_type".to_string(),
        Value::String(filter.field_type().as_str().to_string()),
    );
    object.insert(
        "filter_type".to_string(),
        Value::String(filter.filter_type().as_str().to_string()),
    );
    object.insert(
        "mcp_input".to_string(),
        component_shape_mcp::mcp_input_descriptor_value(filter.mcp_input()),
    );
    if !filter.validation_rules().is_empty() {
        object.insert(
            "validation_rules".to_string(),
            Value::Array(
                filter
                    .validation_rules()
                    .iter()
                    .map(|rule| rule.to_value())
                    .collect(),
            ),
        );
    }
    object.insert("schema".to_string(), schema_for_filter(filter).into_value());
    Value::Object(object)
}

/// Add descriptor and schema resources for every inventory-discovered MCP table.
///
/// Use this when a custom server should expose table resources for linked
/// `#[gpui_table(mcp)]` tables without registering their query handlers.
pub fn register_inventory_table_resources(server: &mut McpServer) -> Result<(), McpToolError> {
    let specs = inventory_table_resource_specs()?;
    component_shape_mcp::register_json_resource_specs(server, specs)
}

/// Register descriptor and schema resources for one table.
///
/// Generated server registration calls this automatically for inventory
/// discovered `#[gpui_table(mcp)]` tables. Use this helper when a custom server
/// should expose table resources without registering a query handler.
pub fn register_table_resources<Table>(server: &mut McpServer) -> Result<(), McpToolError>
where
    Table: McpTable,
{
    register_table_resources_for_descriptor(server, Table::descriptor())
}

pub(super) fn inventory_table_resource_specs() -> Result<Vec<McpTableResourceSpec>, McpToolError> {
    let mut specs = Vec::new();
    for registration in registry::table_registrations() {
        specs.extend(table_resource_specs(registration.descriptor())?);
    }
    Ok(specs)
}

/// Return MCP resource definitions for every inventory-discovered MCP table.
///
/// This mirrors [`tool_definitions`] for callers that want to inspect generated
/// resources before building a server.
pub fn resource_definitions() -> Result<Vec<ResourceDefinition>, McpToolError> {
    let specs = inventory_table_resource_specs()?;
    component_shape_mcp::json_resource_definitions(&specs)
}

pub(super) fn register_table_resources_for_descriptor(
    server: &mut McpServer,
    descriptor: McpTableDescriptor,
) -> Result<(), McpToolError> {
    let specs = table_resource_specs(descriptor)?;
    component_shape_mcp::register_json_resource_specs(server, specs)
}

pub(super) fn register_table_resources_if_missing_for_descriptor(
    server: &mut McpServer,
    descriptor: McpTableDescriptor,
) -> Result<(), McpToolError> {
    let specs = table_resource_specs(descriptor)?;
    component_shape_mcp::register_json_resource_specs_if_missing(server, specs)
}

pub(super) type McpTableResourceSpec = component_shape_mcp::McpJsonResourceSpec;

pub(super) fn table_resource_specs(
    descriptor: McpTableDescriptor,
) -> Result<Vec<McpTableResourceSpec>, McpToolError> {
    let uris = table_resource_uris(descriptor);
    let tool_name = descriptor.tool_name();
    let title = descriptor.title();
    Ok(vec![
        table_json_resource_spec(
            uris.descriptor,
            format!("{tool_name}_descriptor"),
            format!("{title} descriptor"),
            format!("Filter metadata for the {title} generated table query."),
            table_descriptor_resource_value(descriptor),
        )?,
        table_json_resource_spec(
            uris.schema,
            format!("{tool_name}_schema"),
            format!("{title} schema"),
            format!("Input JSON Schema for the {title} generated table query."),
            table_schema_resource_value(descriptor),
        )?,
    ])
}

fn table_json_resource_spec(
    uri: String,
    name: String,
    title: String,
    description: String,
    value: Value,
) -> Result<McpTableResourceSpec, McpToolError> {
    component_shape_mcp::McpJsonResourceSpec::new(uri, name, Some(title), Some(description), value)
}
