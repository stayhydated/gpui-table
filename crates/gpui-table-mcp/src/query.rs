use super::*;

#[derive(Clone, Debug)]
/// A decoded query for a generated MCP table.
pub struct TableQuery<Table>
where
    Table: McpTable,
{
    /// Generated filter values decoded from tool arguments.
    pub filters: Table::FilterValues,
    /// Maximum number of rows requested for this page.
    pub limit: Option<usize>,
    /// Number of matching rows to skip before this page.
    pub offset: usize,
}

impl<Table> TableQuery<Table>
where
    Table: McpTable,
{
    /// Builds a standard query response from backend-selected rows and a total count.
    pub fn result(&self, rows: Vec<Table>, total: usize) -> TableQueryResult<Table> {
        TableQueryResult {
            rows,
            total,
            offset: self.offset,
            limit: self.limit,
        }
    }

    /// Applies generated filters, offset, and limit to an in-memory row source.
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
/// Standard serialized response for a generated table query tool.
pub struct TableQueryResult<Row> {
    /// Rows in the requested page.
    pub rows: Vec<Row>,
    /// Number of rows matching the query before pagination.
    pub total: usize,
    /// Applied page offset.
    pub offset: usize,
    /// Applied page limit.
    pub limit: Option<usize>,
}

/// Generated table contract used for MCP schema, decoding, and registration.
pub trait McpTable: Sized + 'static {
    /// Generated filter-value type decoded from query tool arguments.
    type FilterValues: Default + Clone + 'static;

    /// Returns table metadata, filters, schemas, and MCP tool annotations.
    fn descriptor() -> McpTableDescriptor;

    /// Decodes a raw MCP tool call into a typed table query.
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
