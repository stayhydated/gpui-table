use super::*;

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

pub(super) const fn mcp_input_for_filter_type(filter_type: RegistryFilterType) -> McpInput {
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

    pub(super) fn tool_definition(self) -> Result<ToolDefinition, McpToolError> {
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
