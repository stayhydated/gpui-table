//! Experimental MCP query integration for generated `gpui-table` filters.
//!
//! This crate intentionally keeps GPUI out of the query execution path. It
//! owns table-specific filter decoding and query contracts while delegating
//! shared MCP server and stdio serving mechanics to `component-shape-mcp`.

use std::{collections::BTreeSet, fmt, future::Future, marker::PhantomData, pin::Pin, sync::Arc};

pub use gpui_table_runtime::shape::ComponentShapeMetadata;
use gpui_table_runtime::shape::GpuiTableFilterShape;
use gpui_table_schema::registry::{RegistryFilterType, RustPath, RustType};
pub use serde::Serialize;
use serde_json::{Map, Value, json};

pub type FilterSchemaFn = fn(McpTableFilter) -> McpSchema;
pub type McpTableRowSchemaFn = fn() -> McpSchema;

pub use component_shape::{McpInput, McpInputShape, McpPrimitiveKind, McpRangeBoundKind};
pub use component_shape_mcp::{
    ContentBlock, MCP_PROTOCOL_VERSION, MCP_VALIDATION_PARAMS_NONE, McpAny, McpArguments,
    McpJsonSchema, McpPromptArgument, McpPromptResult, McpRange, McpSchema, McpSchemaProperties,
    McpServer, McpServerBuilder, McpToolAnnotations, McpToolArguments, McpToolCall, McpToolError,
    McpToolInput, McpToolMetadata, McpToolValue, McpTypedTool, McpValidationIssue,
    McpValidationParam, McpValidationRule, McpValidationScope, McpValidationTypeArgMode,
    PromptDefinition, ResourceDefinition, ServeStdioResult, ToolCallResult, ToolDefinition,
    object_schema, serde, serde_json, validation_issues_error,
};
pub use rmcp;

type ToolFuture = Pin<Box<dyn Future<Output = ToolCallResult> + Send + 'static>>;

#[derive(Clone, Copy, Debug)]
pub struct McpTableFilter {
    name: &'static str,
    field_type: RustType,
    filter_type: RegistryFilterType,
    input_schema: FilterSchemaFn,
    validation_rules: &'static [McpValidationRule],
}

impl McpTableFilter {
    pub const fn new(
        name: &'static str,
        field_type: RustType,
        filter_type: RegistryFilterType,
    ) -> Self {
        Self {
            name,
            field_type,
            filter_type,
            input_schema: default_filter_input_schema,
            validation_rules: &[],
        }
    }

    pub const fn for_shape<Shape>(name: &'static str, field_type: RustType) -> Self
    where
        Shape: McpFilterShape,
    {
        Self {
            name,
            field_type,
            filter_type: Shape::FILTER_TYPE,
            input_schema: Shape::input_schema,
            validation_rules: &[],
        }
    }

    pub const fn with_validation_rules(
        mut self,
        validation_rules: &'static [McpValidationRule],
    ) -> Self {
        self.validation_rules = validation_rules;
        self
    }

    pub const fn name(self) -> &'static str {
        self.name
    }

    pub const fn field_type(self) -> RustType {
        self.field_type
    }

    pub const fn filter_type(self) -> RegistryFilterType {
        self.filter_type
    }

    pub fn input_schema(self) -> McpSchema {
        (self.input_schema)(self)
    }

    pub const fn mcp_input(self) -> McpInput {
        mcp_input_for_filter_type(self.filter_type)
    }

    pub const fn validation_rules(self) -> &'static [McpValidationRule] {
        self.validation_rules
    }
}

const fn mcp_input_for_filter_type(filter_type: RegistryFilterType) -> McpInput {
    match filter_type {
        RegistryFilterType::Faceted => McpInput::string_set(),
        RegistryFilterType::DateRange => McpInput::date_range(),
        RegistryFilterType::NumberRange => McpInput::decimal_range(),
        RegistryFilterType::Text => McpInput::string(),
    }
}

#[derive(Clone, Copy, Debug)]
pub struct McpTableDescriptor {
    table_name: &'static str,
    table_id: &'static str,
    table_title: &'static str,
    source_module_path: RustPath,
    filters: &'static [McpTableFilter],
    tool_metadata: McpToolMetadata,
    row_schema: Option<McpTableRowSchemaFn>,
}

impl McpTableDescriptor {
    pub const fn new(
        table_name: &'static str,
        table_id: &'static str,
        table_title: &'static str,
        source_module_path: RustPath,
        filters: &'static [McpTableFilter],
        tool_metadata: McpToolMetadata,
    ) -> Self {
        Self {
            table_name,
            table_id,
            table_title,
            source_module_path,
            filters,
            tool_metadata,
            row_schema: None,
        }
    }

    pub const fn with_row_schema(mut self, schema: McpTableRowSchemaFn) -> Self {
        self.row_schema = Some(schema);
        self
    }

    pub const fn has_row_schema(self) -> bool {
        self.row_schema.is_some()
    }

    pub fn row_schema(self) -> Option<McpSchema> {
        self.row_schema.map(|schema| schema())
    }

    pub const fn table_name(self) -> &'static str {
        self.table_name
    }

    pub const fn table_id(self) -> &'static str {
        self.table_id
    }

    pub const fn table_title(self) -> &'static str {
        self.table_title
    }

    pub const fn source_module_path(self) -> RustPath {
        self.source_module_path
    }

    pub const fn filters(self) -> &'static [McpTableFilter] {
        self.filters
    }

    pub const fn tool_metadata(self) -> McpToolMetadata {
        self.tool_metadata
    }

    pub fn tool_name(self) -> String {
        self.tool_metadata
            .name()
            .map(str::to_string)
            .unwrap_or_else(|| tool_name(self.source_module_path.as_str(), self.table_id))
    }

    pub fn title(self) -> String {
        self.tool_metadata
            .title()
            .map(str::to_string)
            .unwrap_or_else(|| format!("{} query", self.table_title))
    }

    pub fn description(self) -> String {
        self.tool_metadata
            .description()
            .map(str::to_string)
            .unwrap_or_else(|| {
                format!(
                    "Query {} gpui-table rows with generated typed filters.",
                    self.table_title
                )
            })
    }

    pub fn input_schema(self) -> McpSchema {
        input_schema_for_filters(self.filters)
    }

    pub fn output_schema(self) -> McpSchema {
        table_query_output_schema(self.row_schema())
    }

    pub fn tool_annotations(self) -> McpToolAnnotations {
        let metadata = self.tool_metadata;
        let destructive = metadata.destructive_hint().unwrap_or(false);
        let idempotent = metadata
            .idempotent_hint()
            .or_else(|| (!destructive).then_some(true));
        McpToolAnnotations::from_raw(
            Some(self.title()),
            Some(metadata.read_only_hint().unwrap_or(true)),
            Some(destructive),
            idempotent,
            metadata.open_world_hint(),
        )
    }

    fn tool_definition(self) -> Result<ToolDefinition, McpToolError> {
        self.tool_metadata.validate()?;
        component_shape_mcp::tool_definition_with_annotations(
            self.tool_name(),
            Some(self.title()),
            Some(self.description()),
            self.input_schema(),
            Some(self.output_schema()),
            Some(self.tool_annotations()),
        )
    }
}

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

struct McpTablePromptSpec {
    name: String,
    definition: PromptDefinition,
    descriptor: McpTableDescriptor,
}

fn table_filter_descriptor_value(filter: McpTableFilter) -> Value {
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

#[derive(Clone, Debug)]
pub struct TableQuery<Table>
where
    Table: McpTable,
{
    pub filters: Table::FilterValues,
    pub limit: Option<usize>,
    pub offset: usize,
}

impl<Table> TableQuery<Table>
where
    Table: McpTable,
{
    pub fn result(&self, rows: Vec<Table>, total: usize) -> TableQueryResult<Table> {
        TableQueryResult {
            rows,
            total,
            offset: self.offset,
            limit: self.limit,
        }
    }

    pub fn filter_rows<Rows>(&self, rows: Rows) -> TableQueryResult<Table>
    where
        Table: gpui_table_core::filter::Matchable<Table::FilterValues>,
        Rows: IntoIterator<Item = Table>,
    {
        let mut total = 0usize;
        let mut page = Vec::new();

        for row in rows {
            if !row.matches_filters(&self.filters) {
                continue;
            }

            if total >= self.offset && self.limit.is_none_or(|limit| page.len() < limit) {
                page.push(row);
            }

            total += 1;
        }

        self.result(page, total)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct TableQueryResult<Row> {
    pub rows: Vec<Row>,
    pub total: usize,
    pub offset: usize,
    pub limit: Option<usize>,
}

pub trait McpTable: Sized + 'static {
    type FilterValues: Default + Clone + 'static;

    fn descriptor() -> McpTableDescriptor;

    fn decode_query(call: McpToolCall) -> Result<TableQuery<Self>, McpToolError>;
}

/// Typed MCP query input for a generated table.
///
/// Registration uses this wrapper to keep the descriptor schema, generated
/// query decoding, and handler input type paired at the shared MCP server
/// boundary.
pub struct McpTableQueryInput<Table>
where
    Table: McpTable,
{
    query: TableQuery<Table>,
}

impl<Table> McpTableQueryInput<Table>
where
    Table: McpTable,
{
    pub fn tool_definition() -> Result<McpTypedTool<Self>, McpToolError> {
        let descriptor = Table::descriptor();
        descriptor.tool_metadata().validate()?;
        component_shape_mcp::tool_definition_for_input_with_annotations::<Self>(
            descriptor.tool_name(),
            Some(descriptor.title()),
            Some(descriptor.description()),
            Some(descriptor.output_schema()),
            Some(descriptor.tool_annotations()),
        )
    }

    pub fn into_query(self) -> TableQuery<Table> {
        self.query
    }
}

impl<Table> McpToolInput for McpTableQueryInput<Table>
where
    Table: McpTable,
{
    fn input_schema() -> McpSchema {
        Table::descriptor().input_schema()
    }

    fn from_tool_call(call: McpToolCall) -> Result<Self, McpToolError> {
        Ok(Self {
            query: Table::decode_query(call)?,
        })
    }
}

#[diagnostic::on_unimplemented(
    message = "table filter shape `{Self}` cannot be decoded from MCP tool arguments",
    note = "derive `gpui_table::McpFilterShape` when the shape raw value implements gpui_table::mcp::McpToolValue, or implement `gpui_table::mcp::McpFilterShape` manually for custom decoding"
)]
pub trait McpFilterShape: GpuiTableFilterShape {
    fn input_schema(filter: McpTableFilter) -> McpSchema {
        default_filter_input_schema(filter)
    }

    fn decode_filter(field: &'static str, value: McpAny)
    -> Result<Self::FilterValue, McpToolError>;
}

#[diagnostic::on_unimplemented(
    message = "table filter shape `{Self}` cannot run Koruma validation on MCP filter arguments",
    note = "use a built-in shape, derive `gpui_table::McpFilterShape` for a shape whose raw value implements gpui_table::mcp::McpToolValue, or implement `gpui_table::mcp::McpFilterShapeValidation` manually"
)]
pub trait McpFilterShapeValidation: McpFilterShape {
    fn decode_filter_with_validation<Validate>(
        field: &'static str,
        value: McpAny,
        validate: Validate,
    ) -> Result<Self::FilterValue, McpToolError>
    where
        Validate: FnOnce(&Self::RawValue) -> Result<(), McpToolError>;
}

#[diagnostic::on_unimplemented(
    message = "table filter shape `{Self}` cannot validate Koruma newtype field `{Field}` from its decoded MCP raw value",
    note = "derive the shape with `#[gpui_table_filter_shape(..., koruma_newtype)]`, or implement `gpui_table::mcp::McpKorumaNewtypeFilterValidation` manually"
)]
pub trait McpKorumaNewtypeFilterValidation<Field>: GpuiTableFilterShape {
    fn validate_koruma_newtype_filter(value: &Self::RawValue) -> bool;
}

pub fn default_filter_shape_input_schema<Shape>(_filter: McpTableFilter) -> McpSchema
where
    Shape: GpuiTableFilterShape,
    Shape::RawValue: McpToolValue,
{
    <Shape::RawValue as McpToolValue>::tool_value_schema()
}

pub fn decode_raw_filter_shape<Shape>(
    field: &'static str,
    value: McpAny,
) -> Result<Shape::FilterValue, McpToolError>
where
    Shape: GpuiTableFilterShape,
    Shape::RawValue: McpToolValue,
{
    let value = <Shape::RawValue as McpToolValue>::from_tool_value(field, value.into_value())?;
    Ok(Shape::wrap_value(value))
}

pub fn decode_raw_filter_shape_with_validation<Shape, Validate>(
    field: &'static str,
    value: McpAny,
    validate: Validate,
) -> Result<Shape::FilterValue, McpToolError>
where
    Shape: GpuiTableFilterShape,
    Shape::RawValue: McpToolValue,
    Validate: FnOnce(&Shape::RawValue) -> Result<(), McpToolError>,
{
    let value = <Shape::RawValue as McpToolValue>::from_tool_value(field, value.into_value())?;
    validate(&value)?;
    Ok(Shape::wrap_value(value))
}

pub mod registry {
    use super::{McpServer, McpTableDescriptor, McpToolError};

    pub use inventory;

    inventory::collect!(McpTableRegistration);
    inventory::collect!(McpQueryHandlerRegistration);

    pub struct McpTableRegistration {
        descriptor: fn() -> McpTableDescriptor,
    }

    impl McpTableRegistration {
        pub const fn new(descriptor: fn() -> McpTableDescriptor) -> Self {
            Self { descriptor }
        }

        pub fn descriptor(&self) -> McpTableDescriptor {
            (self.descriptor)()
        }
    }

    pub fn table_registrations() -> impl Iterator<Item = &'static McpTableRegistration> {
        inventory::iter::<McpTableRegistration>.into_iter()
    }

    pub struct McpQueryHandlerRegistration {
        register: fn(&mut McpServer) -> Result<(), McpToolError>,
    }

    impl McpQueryHandlerRegistration {
        pub const fn new(register: fn(&mut McpServer) -> Result<(), McpToolError>) -> Self {
            Self { register }
        }

        pub fn register(&self, server: &mut McpServer) -> Result<(), McpToolError> {
            (self.register)(server)
        }
    }

    pub fn query_handler_registrations()
    -> impl Iterator<Item = &'static McpQueryHandlerRegistration> {
        inventory::iter::<McpQueryHandlerRegistration>.into_iter()
    }
}

#[cfg(any(feature = "chrono", feature = "rust_decimal"))]
pub fn range_filter_input_schema<T>(_filter: McpTableFilter) -> McpSchema
where
    McpRange<T>: McpToolValue,
{
    <McpRange<T> as McpToolValue>::tool_value_schema()
}

#[cfg(any(feature = "chrono", feature = "rust_decimal"))]
pub fn decode_range_filter<T>(
    field: &'static str,
    value: McpAny,
) -> Result<(Option<T>, Option<T>), McpToolError>
where
    McpRange<T>: McpToolValue,
{
    let range = <McpRange<T> as McpToolValue>::from_tool_value(field, value.into_value())?;
    Ok(range.into_tuple())
}

pub const DEFAULT_SERVER_NAME: &str = "gpui-table-mcp";

/// Start registering an MCP query tool for a generated table.
pub fn table<Table>(server: &mut McpServer) -> TableTool<'_, Table>
where
    Table: McpTable,
{
    TableTool::new(server)
}

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

fn register_table_prompt_templates_for_descriptor(
    server: &mut McpServer,
    descriptor: McpTableDescriptor,
) -> Result<(), McpToolError> {
    let resource_specs = table_resource_specs(descriptor)?;
    let prompt_specs = table_prompt_specs(descriptor)?;
    component_shape_mcp::register_json_resource_specs_if_missing(server, resource_specs)?;
    ensure_prompt_specs_available(server, &prompt_specs)?;
    register_table_prompt_specs(server, prompt_specs)
}

fn inventory_table_resource_specs() -> Result<Vec<McpTableResourceSpec>, McpToolError> {
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

fn register_table_resources_for_descriptor(
    server: &mut McpServer,
    descriptor: McpTableDescriptor,
) -> Result<(), McpToolError> {
    let specs = table_resource_specs(descriptor)?;
    component_shape_mcp::register_json_resource_specs(server, specs)
}

fn register_table_resources_if_missing_for_descriptor(
    server: &mut McpServer,
    descriptor: McpTableDescriptor,
) -> Result<(), McpToolError> {
    let specs = table_resource_specs(descriptor)?;
    component_shape_mcp::register_json_resource_specs_if_missing(server, specs)
}

type McpTableResourceSpec = component_shape_mcp::McpJsonResourceSpec;

fn table_resource_specs(
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

fn inventory_table_prompt_specs() -> Result<Vec<McpTablePromptSpec>, McpToolError> {
    let mut seen_tool_names = BTreeSet::new();
    let mut specs = Vec::new();
    for registration in registry::table_registrations() {
        push_descriptor_prompt_specs(&mut seen_tool_names, &mut specs, registration.descriptor())?;
    }
    Ok(specs)
}

fn push_descriptor_prompt_specs(
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

fn table_prompt_result(
    descriptor: McpTableDescriptor,
    arguments: Option<Map<String, Value>>,
) -> McpPromptResult {
    component_shape_mcp::text_prompt_result(
        Some(format!("Query {}.", descriptor.title())),
        table_prompt_text(descriptor, arguments),
    )
}

fn table_prompt_text(
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

fn input_schema_for_filters(filters: &[McpTableFilter]) -> McpSchema {
    let mut properties = McpSchemaProperties::new();

    for filter in filters {
        properties.insert(filter.name().to_string(), schema_for_filter(*filter));
    }

    properties.insert(
        "limit".to_string(),
        McpSchema::integer().with_minimum(0_u64),
    );
    properties.insert(
        "offset".to_string(),
        McpSchema::integer().with_minimum(0_u64),
    );

    component_shape_mcp::object_schema(properties, std::iter::empty::<&'static str>())
}

fn schema_for_filter(filter: McpTableFilter) -> McpSchema {
    let mut schema = filter.input_schema();

    if let Some(object) = schema.as_object_mut() {
        object.insert(
            "x-rustType".to_string(),
            Value::String(filter.field_type().as_str().to_string()),
        );
        object.insert(
            "x-gpuiTableFilterType".to_string(),
            Value::String(filter.filter_type().as_str().to_string()),
        );
        component_shape_mcp::apply_validation_schema_metadata(
            object,
            "x-gpuiTableValidation",
            filter.validation_rules(),
        );
    }

    schema
}

pub fn default_filter_input_schema(filter: McpTableFilter) -> McpSchema {
    component_shape_mcp::schema_for_input(mcp_input_for_filter_type(filter.filter_type()))
}

pub fn table_query_output_schema(row_schema: Option<McpSchema>) -> McpSchema {
    let row_items = row_schema.unwrap_or_else(McpSchema::any);
    let mut properties = McpSchemaProperties::new();
    properties.insert("rows".to_string(), McpSchema::array(row_items));
    properties.insert(
        "total".to_string(),
        McpSchema::integer().with_minimum(0_u64),
    );
    properties.insert(
        "offset".to_string(),
        McpSchema::integer().with_minimum(0_u64),
    );
    properties.insert(
        "limit".to_string(),
        McpSchema::any_of([McpSchema::integer().with_minimum(0_u64), McpSchema::null()]),
    );

    component_shape_mcp::object_schema(properties, ["rows", "total", "offset", "limit"])
}

pub fn table_query_output_schema_for_row<Row>() -> McpSchema
where
    Row: McpJsonSchema,
{
    table_query_output_schema(Some(Row::json_schema()))
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

#[cfg(test)]
mod tests {
    use super::{
        McpInput, McpRange, McpTableDescriptor, McpTableFilter, McpTableQueryInput,
        McpToolInput as _, default_filter_shape_input_schema,
        register_table_prompt_templates_for_descriptor, register_table_resources_for_descriptor,
        table_descriptor_resource_value, table_prompt_names, table_prompt_text,
        table_query_output_schema, table_query_output_schema_for_row, table_resource_uris,
        tool_name,
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
            _title: impl Fn(&gpui::App) -> String + 'static,
            _value: Self::RawValue,
            _on_change: impl Fn(Self::RawValue, &mut gpui::Window, &mut gpui::App) + 'static,
            _cx: &mut gpui::App,
        ) -> gpui::Entity<Self::Component> {
            unimplemented!("schema test does not instantiate GPUI state")
        }

        fn read_value(_entity: &gpui::Entity<Self::Component>, _cx: &gpui::App) -> Self::RawValue {
            unimplemented!("schema test does not read GPUI state")
        }

        fn wrap_value(value: Self::RawValue) -> Self::FilterValue {
            value
        }

        fn reset_silent(
            _entity: &gpui::Entity<Self::Component>,
            _window: &mut gpui::Window,
            _cx: &mut gpui::App,
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
            tool.output_schema.as_ref().unwrap()["properties"]["rows"]["items"]["properties"]["name"]
                ["type"],
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
}
