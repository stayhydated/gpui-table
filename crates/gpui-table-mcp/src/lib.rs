//! Experimental MCP query integration for generated `gpui-table` filters.
//!
//! This crate intentionally keeps GPUI out of the query execution path. It
//! owns table-specific filter decoding and query contracts while delegating
//! shared MCP server and stdio serving mechanics to `component-shape-mcp`.

use std::{
    collections::{BTreeSet, HashSet},
    fmt,
    future::Future,
    marker::PhantomData,
    sync::Arc,
};

pub use gpui_table_runtime::shape::ComponentShapeMetadata;
use gpui_table_runtime::shape::GpuiTableFilterShape;
use gpui_table_schema::registry::{RegistryFilterType, RustPath, RustType};
pub use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

pub type FilterSchemaFn = fn(McpTableFilter) -> Value;

pub use component_shape::{McpInput, McpInputShape, McpPrimitiveKind};
pub use component_shape_mcp::{
    ContentBlock, MCP_PROTOCOL_VERSION, McpArguments, McpJsonSchema, McpRange, McpServer,
    McpServerBuilder, McpToolArguments, McpToolCall, McpToolError, McpToolMetadata,
    ServeStdioResult, ToolCallResult, ToolDefinition, object_schema, rmcp, serde_json,
};

#[derive(Clone, Copy, Debug)]
pub struct McpTableFilter {
    name: &'static str,
    field_type: RustType,
    filter_type: RegistryFilterType,
    input_schema: FilterSchemaFn,
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
        }
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

    pub fn input_schema(self) -> Value {
        (self.input_schema)(self)
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
        }
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

    pub fn input_schema(self) -> Value {
        input_schema_for_filters(self.filters)
    }

    pub fn output_schema(self) -> Value {
        table_query_output_schema()
    }

    pub fn tool_definition(self) -> Result<ToolDefinition, McpToolError> {
        component_shape_mcp::tool_definition(
            self.tool_name(),
            Some(self.title()),
            Some(self.description()),
            self.input_schema(),
            Some(self.output_schema()),
        )
    }
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

#[diagnostic::on_unimplemented(
    message = "table filter shape `{Self}` cannot be decoded from MCP tool arguments",
    note = "derive `gpui_table::McpFilterShape` when the shape raw value implements serde::de::DeserializeOwned and gpui_table::mcp::McpJsonSchema, or implement `gpui_table::mcp::McpFilterShape` manually for custom decoding"
)]
pub trait McpFilterShape: GpuiTableFilterShape {
    fn input_schema(filter: McpTableFilter) -> Value {
        default_filter_input_schema(filter)
    }

    fn decode_filter(field: &'static str, value: Value) -> Result<Self::FilterValue, McpToolError>;
}

pub fn default_filter_shape_input_schema<Shape>(_filter: McpTableFilter) -> Value
where
    Shape: GpuiTableFilterShape,
    Shape::RawValue: McpJsonSchema,
{
    Shape::RawValue::json_schema()
}

pub fn decode_raw_filter_shape<Shape>(
    field: &'static str,
    value: Value,
) -> Result<Shape::FilterValue, McpToolError>
where
    Shape: GpuiTableFilterShape,
    Shape::RawValue: DeserializeOwned,
{
    let value = component_shape_mcp::decode_present_field::<Shape::RawValue>(field, value)?;
    Ok(Shape::wrap_value(value))
}

impl McpFilterShape for gpui_table_component::TextFilter {
    fn input_schema(filter: McpTableFilter) -> Value {
        default_filter_shape_input_schema::<Self>(filter)
    }

    fn decode_filter(field: &'static str, value: Value) -> Result<Self::FilterValue, McpToolError> {
        decode_raw_filter_shape::<Self>(field, value)
    }
}

impl<T> McpFilterShape for gpui_table_component::FacetedFilter<T>
where
    T: gpui_table_core::filter::Filterable,
{
    fn input_schema(filter: McpTableFilter) -> Value {
        let mut schema = default_filter_input_schema(filter);
        let options = T::options();

        if let Value::Object(object) = &mut schema {
            if !options.is_empty()
                && let Some(items) = object.get_mut("items").and_then(Value::as_object_mut)
            {
                items.insert(
                    "enum".to_string(),
                    Value::Array(
                        options
                            .iter()
                            .map(|option| Value::String(option.value.clone()))
                            .collect(),
                    ),
                );
            }
            object.insert(
                "x-gpuiTableFacetOptions".to_string(),
                Value::Array(
                    options
                        .into_iter()
                        .map(|option| {
                            json!({
                                "value": option.value,
                                "label": option.label,
                                "group": option.group,
                                "count": option.count,
                            })
                        })
                        .collect(),
                ),
            );
        }

        schema
    }

    fn decode_filter(field: &'static str, value: Value) -> Result<Self::FilterValue, McpToolError> {
        let raw_values = component_shape_mcp::decode_present_field::<Vec<String>>(field, value)?;
        let mut values = HashSet::new();
        for raw_value in raw_values {
            let value = T::from_filter_string(&raw_value)
                .ok_or_else(|| McpToolError::invalid_field_value(field, raw_value))?;
            values.insert(value);
        }
        Ok(<Self as GpuiTableFilterShape>::wrap_value(values))
    }
}

#[cfg(feature = "rust_decimal")]
impl McpFilterShape for gpui_table_component::NumberRangeFilter {
    fn decode_filter(field: &'static str, value: Value) -> Result<Self::FilterValue, McpToolError> {
        let value = decode_range_filter(field, value, decode_decimal_bound)?;
        Ok(<Self as GpuiTableFilterShape>::wrap_value(value))
    }
}

#[cfg(feature = "chrono")]
impl McpFilterShape for gpui_table_component::DateRangeFilter {
    fn decode_filter(field: &'static str, value: Value) -> Result<Self::FilterValue, McpToolError> {
        let value = decode_range_filter(field, value, decode_date_bound)?;
        Ok(<Self as GpuiTableFilterShape>::wrap_value(value))
    }
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

fn decode_range_filter<T, Decode>(
    field: &'static str,
    value: Value,
    mut decode_bound: Decode,
) -> Result<(Option<T>, Option<T>), McpToolError>
where
    Decode: FnMut(&'static str, &'static str, Value) -> Result<Option<T>, McpToolError>,
{
    if value.is_null() {
        return Err(McpToolError::UnexpectedNull {
            field: field.to_string(),
        });
    }

    let mut arguments = match value {
        Value::Object(arguments) => arguments,
        _ => {
            return Err(McpToolError::decode(
                field,
                "range filters must be JSON objects with optional `min` and `max` fields",
            ));
        },
    };

    let min = arguments
        .remove("min")
        .map(|value| decode_bound(field, "min", value))
        .transpose()?
        .flatten();
    let max = arguments
        .remove("max")
        .map(|value| decode_bound(field, "max", value))
        .transpose()?
        .flatten();

    if let Some(bound) = arguments.keys().next() {
        return Err(McpToolError::UnknownField {
            field: format!("{field}.{bound}"),
        });
    }

    Ok((min, max))
}

#[cfg(feature = "rust_decimal")]
fn decode_decimal_bound(
    field: &'static str,
    bound: &'static str,
    value: Value,
) -> Result<Option<rust_decimal::Decimal>, McpToolError> {
    use std::str::FromStr as _;

    let raw = match value {
        Value::Null => return Ok(None),
        Value::Number(value) => value.to_string(),
        Value::String(value) => {
            let value = value.trim().to_string();
            if value.is_empty() {
                return Ok(None);
            }
            value
        },
        _ => {
            return Err(McpToolError::decode(
                format!("{field}.{bound}"),
                "number range bounds must be numbers, numeric strings, or null",
            ));
        },
    };

    rust_decimal::Decimal::from_str(&raw)
        .map(Some)
        .map_err(|error| McpToolError::decode(format!("{field}.{bound}"), error.to_string()))
}

#[cfg(feature = "chrono")]
fn decode_date_bound(
    field: &'static str,
    bound: &'static str,
    value: Value,
) -> Result<Option<chrono::NaiveDate>, McpToolError> {
    let raw = match value {
        Value::Null => return Ok(None),
        Value::String(value) => {
            let value = value.trim().to_string();
            if value.is_empty() {
                return Ok(None);
            }
            value
        },
        _ => {
            return Err(McpToolError::decode(
                format!("{field}.{bound}"),
                "date range bounds must be `YYYY-MM-DD` strings or null",
            ));
        },
    };

    chrono::NaiveDate::parse_from_str(&raw, "%Y-%m-%d")
        .map(Some)
        .map_err(|error| McpToolError::decode(format!("{field}.{bound}"), error.to_string()))
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
        insert_executor(self.server, Table::descriptor(), move |arguments| {
            let query = match decode_query::<Table>(arguments) {
                Ok(query) => query,
                Err(error) => return component_shape_mcp::tool_error_result(error.to_string()),
            };

            component_shape_mcp::serialize_handler_response(handler(query))
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
        insert_executor_async(self.server, Table::descriptor(), move |arguments| {
            let handler = Arc::clone(&handler);
            async move {
                let query = match decode_query::<Table>(arguments) {
                    Ok(query) => query,
                    Err(error) => return component_shape_mcp::tool_error_result(error.to_string()),
                };

                component_shape_mcp::serialize_handler_response(handler(query).await)
            }
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

fn insert_executor<Call>(
    server: &mut McpServer,
    descriptor: McpTableDescriptor,
    call: Call,
) -> Result<(), McpToolError>
where
    Call: Fn(McpToolCall) -> ToolCallResult + Send + Sync + 'static,
{
    server.add_tool(descriptor.tool_definition()?, call)
}

fn insert_executor_async<Call, Fut>(
    server: &mut McpServer,
    descriptor: McpTableDescriptor,
    call: Call,
) -> Result<(), McpToolError>
where
    Call: Fn(McpToolCall) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ToolCallResult> + Send + 'static,
{
    server.add_tool_async(descriptor.tool_definition()?, call)
}

pub fn tool_name(source_module_path: &str, table_id: &str) -> String {
    component_shape_mcp::tool_name(source_module_path, table_id, "table_")
}

fn decode_query<Table>(call: McpToolCall) -> Result<TableQuery<Table>, McpToolError>
where
    Table: McpTable,
{
    Table::decode_query(call)
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
    for registration in registry::query_handler_registrations() {
        registration.register(server)?;
    }
    Ok(())
}

fn input_schema_for_filters(filters: &[McpTableFilter]) -> Value {
    let mut properties = Map::new();

    for filter in filters {
        properties.insert(filter.name().to_string(), schema_for_filter(*filter));
    }

    properties.insert(
        "limit".to_string(),
        json!({
            "type": "integer",
            "minimum": 0
        }),
    );
    properties.insert(
        "offset".to_string(),
        json!({
            "type": "integer",
            "minimum": 0
        }),
    );

    component_shape_mcp::object_schema(properties, std::iter::empty::<&'static str>())
}

fn schema_for_filter(filter: McpTableFilter) -> Value {
    let mut schema = filter.input_schema();

    if let Value::Object(object) = &mut schema {
        object.insert(
            "x-rustType".to_string(),
            Value::String(filter.field_type().as_str().to_string()),
        );
        object.insert(
            "x-gpuiTableFilterType".to_string(),
            Value::String(filter_type_name(filter.filter_type()).to_string()),
        );
    }

    schema
}

fn default_filter_input_schema(filter: McpTableFilter) -> Value {
    component_shape_mcp::schema_for_input(mcp_input_for_filter_type(filter.filter_type()))
}

fn table_query_output_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "rows": { "type": "array", "items": {} },
            "total": { "type": "integer", "minimum": 0 },
            "offset": { "type": "integer", "minimum": 0 },
            "limit": {
                "anyOf": [
                    { "type": "integer", "minimum": 0 },
                    { "type": "null" }
                ]
            }
        },
        "required": ["rows", "total", "offset", "limit"],
        "additionalProperties": false
    })
}

fn filter_type_name(filter_type: RegistryFilterType) -> &'static str {
    match filter_type {
        RegistryFilterType::Faceted => "faceted",
        RegistryFilterType::DateRange => "date_range",
        RegistryFilterType::NumberRange => "number_range",
        RegistryFilterType::Text => "text",
    }
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
        McpInput, McpRange, McpTableDescriptor, McpTableFilter, default_filter_shape_input_schema,
        tool_name,
    };
    use gpui_table_runtime::shape::GpuiTableFilterShape;
    use gpui_table_schema::registry::{RegistryFilterType, RustPath, RustType};

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
        fn input_schema(filter: McpTableFilter) -> serde_json::Value {
            default_filter_shape_input_schema::<Self>(filter)
        }

        fn decode_filter(
            field: &'static str,
            value: serde_json::Value,
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
}
