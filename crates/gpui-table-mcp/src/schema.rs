use super::*;

pub(super) fn input_schema_for_filters(filters: &[McpTableFilter]) -> McpSchema {
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

pub(super) fn schema_for_filter(filter: McpTableFilter) -> McpSchema {
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
