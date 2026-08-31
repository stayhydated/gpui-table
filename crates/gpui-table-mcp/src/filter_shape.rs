use super::*;

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
