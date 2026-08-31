use super::*;

type ToolFuture = Pin<Box<dyn Future<Output = ToolCallResult> + Send + 'static>>;

pub const DEFAULT_SERVER_NAME: &str = "gpui-table-mcp";

/// Start registering an MCP query tool for a generated table.
pub fn table<Table>(server: &mut McpServer) -> TableTool<'_, Table>
where
    Table: McpTable,
{
    TableTool::new(server)
}

/// Fluent registration handle for one generated table query tool.
pub struct TableTool<'server, Table> {
    server: &'server mut McpServer,
    _table: PhantomData<fn() -> Table>,
}

impl<'server, Table> TableTool<'server, Table>
where
    Table: McpTable,
{
    fn new(server: &'server mut McpServer) -> Self {
        Self {
            server,
            _table: PhantomData,
        }
    }

    /// Registers a synchronous application-owned query handler.
    pub fn query<Handler, Error>(self, handler: Handler) -> Result<(), McpToolError>
    where
        Handler:
            Fn(TableQuery<Table>) -> Result<TableQueryResult<Table>, Error> + Send + Sync + 'static,
        Table: Serialize,
        Error: fmt::Display,
    {
        insert_executor::<Table, _>(self.server, move |input| {
            component_shape_mcp::serialize_handler_response(handler(input.into_query()))
        })
    }

    /// Registers an asynchronous application-owned query handler.
    pub fn query_async<Handler, Fut, Error>(self, handler: Handler) -> Result<(), McpToolError>
    where
        Handler: Fn(TableQuery<Table>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<TableQueryResult<Table>, Error>> + Send + 'static,
        Table: Serialize,
        Error: fmt::Display,
    {
        let handler = Arc::new(handler);
        insert_executor_async::<Table, _>(self.server, move |input| {
            let handler = Arc::clone(&handler);
            let future = handler(input.into_query());
            Box::pin(async move { component_shape_mcp::serialize_handler_response(future.await) })
        })
    }

    /// Registers a fixed in-memory row set with generated filtering and pagination.
    pub fn rows(self, rows: Vec<Table>) -> Result<(), McpToolError>
    where
        Table: gpui_table_core::filter::Matchable<Table::FilterValues>
            + Clone
            + Send
            + Sync
            + Serialize
            + 'static,
    {
        self.query(move |query| -> Result<TableQueryResult<Table>, String> {
            Ok(query.filter_rows(rows.clone()))
        })
    }

    /// Registers a synchronous row source called once per query.
    pub fn row_source<Source, Error>(self, source: Source) -> Result<(), McpToolError>
    where
        Source: Fn() -> Result<Vec<Table>, Error> + Send + Sync + 'static,
        Table: gpui_table_core::filter::Matchable<Table::FilterValues>
            + Send
            + Sync
            + Serialize
            + 'static,
        Error: fmt::Display,
    {
        let source = Arc::new(source);
        self.query(move |query| {
            let rows = source()?;
            Ok::<TableQueryResult<Table>, Error>(query.filter_rows(rows))
        })
    }

    /// Registers an asynchronous row source called once per query.
    pub fn row_source_async<Source, Fut, Error>(self, source: Source) -> Result<(), McpToolError>
    where
        Source: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Vec<Table>, Error>> + Send + 'static,
        Table: gpui_table_core::filter::Matchable<Table::FilterValues>
            + Send
            + Sync
            + Serialize
            + 'static,
        Error: fmt::Display,
        Table::FilterValues: Send,
    {
        let source = Arc::new(source);
        self.query_async(move |query| {
            let source = Arc::clone(&source);
            async move {
                let rows = source().await?;
                Ok::<TableQueryResult<Table>, Error>(query.filter_rows(rows))
            }
        })
    }
}

fn insert_executor<Table, Call>(server: &mut McpServer, call: Call) -> Result<(), McpToolError>
where
    Table: McpTable,
    Call: Fn(McpTableQueryInput<Table>) -> ToolCallResult + Send + Sync + 'static,
{
    let definition = McpTableQueryInput::<Table>::tool_definition()?;
    let tool_name = definition.definition().name.to_string();
    if server.contains_tool(&tool_name) {
        return Err(McpToolError::duplicate_tool(tool_name));
    }
    register_table_resources_if_missing_for_descriptor(server, Table::descriptor())?;
    server.add_typed_tool(definition, call)
}

fn insert_executor_async<Table, Call>(
    server: &mut McpServer,
    call: Call,
) -> Result<(), McpToolError>
where
    Table: McpTable,
    Call: Fn(McpTableQueryInput<Table>) -> ToolFuture + Send + Sync + 'static,
{
    let definition = McpTableQueryInput::<Table>::tool_definition()?;
    let tool_name = definition.definition().name.to_string();
    if server.contains_tool(&tool_name) {
        return Err(McpToolError::duplicate_tool(tool_name));
    }
    register_table_resources_if_missing_for_descriptor(server, Table::descriptor())?;
    server.add_typed_tool_async(definition, call)
}

pub fn tool_name(source_module_path: &str, table_id: &str) -> String {
    component_shape_mcp::tool_name(source_module_path, table_id, "table_")
}

/// Build a server containing every inventory-discovered table query handler.
pub fn server() -> Result<McpServer, McpToolError> {
    builder().build()
}

/// Build the inventory-discovered table MCP tool registry.
pub fn tool_registry() -> Result<McpToolRegistry, McpToolError> {
    server().map(McpServer::into_tool_registry)
}

/// Serve every inventory-discovered table query handler over stdio.
pub async fn serve_stdio() -> ServeStdioResult {
    server()?.serve_stdio().await
}

/// Serve every inventory-discovered table query handler over stdio from a
/// synchronous `main`.
pub fn serve_stdio_blocking() -> ServeStdioResult {
    server()?.serve_stdio_blocking()
}

/// Build a generated table query server with application-owned metadata.
pub fn server_named(
    server_name: impl Into<std::borrow::Cow<'static, str>>,
    server_version: impl Into<std::borrow::Cow<'static, str>>,
) -> Result<McpServer, McpToolError> {
    builder_named(server_name, server_version).build()
}

/// Build a server builder containing every inventory-discovered table query handler.
pub fn builder() -> McpServerBuilder {
    builder_named(DEFAULT_SERVER_NAME, env!("CARGO_PKG_VERSION"))
}

/// Build a generated table query server builder with application-owned metadata.
pub fn builder_named(
    server_name: impl Into<std::borrow::Cow<'static, str>>,
    server_version: impl Into<std::borrow::Cow<'static, str>>,
) -> McpServerBuilder {
    McpServer::builder(server_name, server_version).register(register)
}

/// Add every inventory-discovered table query handler to a shared MCP server.
pub fn register(server: &mut McpServer) -> Result<(), McpToolError> {
    register_inventory_table_resources(server)?;
    for registration in registry::query_handler_registrations() {
        registration.register(server)?;
    }
    Ok(())
}

/// Return MCP table tool definitions, failing if duplicate tool names exist.
pub fn tool_definitions() -> Result<Vec<ToolDefinition>, McpToolError> {
    let mut seen = BTreeSet::new();
    let mut tools = Vec::new();
    for registration in registry::table_registrations() {
        let definition = registration.descriptor().tool_definition()?;
        let name = definition.name.to_string();
        if !seen.insert(name.clone()) {
            return Err(McpToolError::duplicate_tool(name));
        }
        tools.push(definition);
    }
    Ok(tools)
}
