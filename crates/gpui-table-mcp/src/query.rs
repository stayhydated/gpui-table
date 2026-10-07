use super::*;

#[derive(Clone, Debug)]
/// A decoded query for a generated MCP table.
#[non_exhaustive]
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
    /// Full-result ordering to execute before pagination.
    pub ordering: SortOrder,
}

impl<Table> TableQuery<Table>
where
    Table: McpTable,
{
    /// Create a query with source ordering and bounded pagination.
    pub fn new(filters: Table::FilterValues, limit: Option<usize>, offset: usize) -> Self {
        Self {
            filters,
            limit,
            offset,
            ordering: SortOrder::default(),
        }
    }

    pub fn with_ordering(mut self, ordering: SortOrder) -> Self {
        self.ordering = ordering;
        self
    }

    /// Builds a standard query response from backend-selected rows and a total count.
    pub fn result(&self, rows: Vec<Table>, total: usize) -> TableQueryResult<Table> {
        TableQueryResult {
            rows,
            total,
            offset: self.offset,
            limit: self.limit,
        }
    }

    /// Apply filters, complete-result ordering, then offset and limit.
    /// An empty order preserves source order and streams without buffering all matches.
    pub fn filter_rows<Rows>(&self, rows: Rows) -> Result<TableQueryResult<Table>, SortError>
    where
        Table: gpui_table_core::filter::Matchable<Table::FilterValues>,
        Rows: IntoIterator<Item = Table>,
    {
        self.ordering.validate::<Table>()?;
        if !self.ordering.is_empty() {
            let mut rows = rows
                .into_iter()
                .filter(|row| row.matches_filters(&self.filters))
                .collect::<Vec<_>>();
            self.ordering.sort_rows(&mut rows)?;
            let total = rows.len();
            let page = rows
                .into_iter()
                .skip(self.offset)
                .take(self.limit.unwrap_or(usize::MAX))
                .collect();
            return Ok(self.result(page, total));
        }
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

        Ok(self.result(page, total))
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
pub trait McpTable: SortableRow + Sized + 'static {
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

/// Decode and validate ordering against the row's declared sortable keys.
pub fn decode_sort_order<Table: SortableRow>(
    arguments: &mut McpArguments,
) -> Result<SortOrder, McpToolError> {
    let Some(value) = arguments.take_present_tool_value::<McpAny>("sort")? else {
        return Ok(SortOrder::default());
    };
    let ordering: SortOrder = serde_json::from_value(value.into_value())
        .map_err(|error| McpToolError::decode("sort", error.to_string()))?;
    ordering
        .validate::<Table>()
        .map_err(|error| McpToolError::decode("sort", error.to_string()))?;
    Ok(ordering)
}
