use super::*;

/// MCP prompt template names generated for one table descriptor.
///
/// The `{tool_name}` segment is the table query tool name, including any
/// struct-level `#[gpui_table(mcp(name = "..."))]` override.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpTablePromptNames {
    /// Prompt that asks a client to draft arguments for the table query tool.
    pub query: String,
}

impl McpTablePromptNames {
    /// Return all generated prompt names.
    pub fn all(&self) -> [&str; 1] {
        [self.query.as_str()]
    }
}

/// Return the MCP prompt template names generated for `descriptor`.
pub fn table_prompt_names(descriptor: McpTableDescriptor) -> McpTablePromptNames {
    McpTablePromptNames {
        query: format!("query_{}_table", descriptor.tool_name()),
    }
}

/// Return the MCP prompt template names generated for `Table`.
pub fn table_prompt_names_for<Table>() -> McpTablePromptNames
where
    Table: McpTable,
{
    table_prompt_names(Table::descriptor())
}

pub(super) struct McpTablePromptSpec {
    name: String,
    definition: PromptDefinition,
    descriptor: McpTableDescriptor,
}

/// Register generated query prompt templates for every inventory-discovered table.
///
/// Prompt templates are opt-in. They can be chained with
/// `McpServer::builder(...).register(...)` after `gpui_table::mcp::register`,
/// or called directly on a mutable server.
pub fn register_prompt_templates(server: &mut McpServer) -> Result<(), McpToolError> {
    let resource_specs = inventory_table_resource_specs()?;
    let prompt_specs = inventory_table_prompt_specs()?;
    component_shape_mcp::register_json_resource_specs_if_missing(server, resource_specs)?;
    ensure_prompt_specs_available(server, &prompt_specs)?;
    register_table_prompt_specs(server, prompt_specs)
}

/// Register the generated query prompt template for one table.
///
/// Prompt templates are opt-in and reference the generated descriptor and
/// schema resources. This helper registers those resources first if they are
/// not already present.
pub fn register_table_prompt_templates<Table>(server: &mut McpServer) -> Result<(), McpToolError>
where
    Table: McpTable,
{
    register_table_prompt_templates_for_descriptor(server, Table::descriptor())
}

pub(super) fn register_table_prompt_templates_for_descriptor(
    server: &mut McpServer,
    descriptor: McpTableDescriptor,
) -> Result<(), McpToolError> {
    let resource_specs = table_resource_specs(descriptor)?;
    let prompt_specs = table_prompt_specs(descriptor)?;
    component_shape_mcp::register_json_resource_specs_if_missing(server, resource_specs)?;
    ensure_prompt_specs_available(server, &prompt_specs)?;
    register_table_prompt_specs(server, prompt_specs)
}

pub(super) fn inventory_table_prompt_specs() -> Result<Vec<McpTablePromptSpec>, McpToolError> {
    let mut seen_tool_names = BTreeSet::new();
    let mut specs = Vec::new();
    for registration in registry::table_registrations() {
        push_descriptor_prompt_specs(&mut seen_tool_names, &mut specs, registration.descriptor())?;
    }
    Ok(specs)
}

pub(super) fn push_descriptor_prompt_specs(
    seen_tool_names: &mut BTreeSet<String>,
    specs: &mut Vec<McpTablePromptSpec>,
    descriptor: McpTableDescriptor,
) -> Result<(), McpToolError> {
    if !seen_tool_names.insert(descriptor.tool_name()) {
        return Ok(());
    }
    specs.extend(table_prompt_specs(descriptor)?);
    Ok(())
}

fn table_prompt_specs(
    descriptor: McpTableDescriptor,
) -> Result<Vec<McpTablePromptSpec>, McpToolError> {
    let names = table_prompt_names(descriptor);
    let title = descriptor.title();
    let table_title = descriptor.table_title();
    Ok(vec![table_prompt_spec(
        names.query,
        format!("Query {table_title}"),
        format!("Draft filter and pagination arguments for {title}."),
        vec![
            optional_prompt_argument(
                "goal",
                "Optional user goal or context for querying the table.",
            ),
            optional_prompt_argument(
                "current_filters",
                "Optional current filters or known constraints to preserve.",
            ),
        ],
        descriptor,
    )?])
}

fn table_prompt_spec(
    name: String,
    title: String,
    description: String,
    arguments: Vec<McpPromptArgument>,
    descriptor: McpTableDescriptor,
) -> Result<McpTablePromptSpec, McpToolError> {
    let definition = component_shape_mcp::prompt_definition(
        name.clone(),
        Some(title),
        Some(description),
        Some(arguments),
    )?;
    Ok(McpTablePromptSpec {
        name,
        definition,
        descriptor,
    })
}

fn optional_prompt_argument(name: &'static str, description: &'static str) -> McpPromptArgument {
    McpPromptArgument::new(name)
        .with_description(description)
        .with_required(false)
}

fn ensure_prompt_specs_available(
    server: &McpServer,
    specs: &[McpTablePromptSpec],
) -> Result<(), McpToolError> {
    for spec in specs {
        if server.contains_prompt(&spec.name) {
            return Err(McpToolError::duplicate_prompt(spec.name.clone()));
        }
    }
    Ok(())
}

fn register_table_prompt_specs(
    server: &mut McpServer,
    specs: Vec<McpTablePromptSpec>,
) -> Result<(), McpToolError> {
    for spec in specs {
        let descriptor = spec.descriptor;
        server.add_prompt(spec.definition, move |arguments| {
            table_prompt_result(descriptor, arguments)
        })?;
    }
    Ok(())
}

pub(super) fn table_prompt_result(
    descriptor: McpTableDescriptor,
    arguments: Option<Map<String, Value>>,
) -> McpPromptResult {
    component_shape_mcp::text_prompt_result(
        Some(format!("Query {}.", descriptor.title())),
        table_prompt_text(descriptor, arguments),
    )
}

pub(super) fn table_prompt_text(
    descriptor: McpTableDescriptor,
    arguments: Option<Map<String, Value>>,
) -> String {
    let resources = table_resource_uris(descriptor);
    let mut text = format!(
        "Query gpui-table `{table_name}` through MCP tool `{tool_name}`.\n\
         Read `{descriptor_uri}` for table metadata, filter field types, validation rules, and per-filter schemas.\n\
         Read `{schema_uri}` for the query argument JSON Schema.\n\
         Return a JSON object that can be used as `arguments` for `{tool_name}`; include only filters to apply plus optional `limit` and `offset`.",
        table_name = descriptor.table_name(),
        tool_name = descriptor.tool_name(),
        descriptor_uri = resources.descriptor,
        schema_uri = resources.schema,
    );

    if let Some(arguments) = arguments
        && !arguments.is_empty()
    {
        text.push_str("\n\nCaller-provided context:\n");
        text.push_str(
            &serde_json::to_string_pretty(&Value::Object(arguments)).unwrap_or_else(|error| {
                format!("{{\"error\":\"failed to render prompt arguments: {error}\"}}")
            }),
        );
    }

    text
}
