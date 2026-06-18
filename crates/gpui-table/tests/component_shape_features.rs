#[cfg(any(feature = "chrono", feature = "rust_decimal"))]
use gpui_table::runtime::shape::McpRangeBoundKind;
use gpui_table::runtime::shape::{
    ComponentShapeFor, ComponentShapeMetadata, DeclaredComponentShape, McpInputShape,
    McpPrimitiveKind,
};

fn assert_filter_shape<Shape, Value>()
where
    Shape: ComponentShapeMetadata + ComponentShapeFor<Value> + DeclaredComponentShape,
{
}

#[test]
fn facade_runtime_accepts_builtin_text_filter_shape() {
    assert_filter_shape::<gpui_table_component::TextFilter, String>();

    assert_eq!(
        <gpui_table_component::TextFilter as ComponentShapeMetadata>::MCP_INPUT.input_shape(),
        McpInputShape::Scalar(McpPrimitiveKind::String)
    );
}

#[cfg(feature = "chrono")]
#[test]
fn facade_runtime_accepts_builtin_date_range_filter_shape_when_enabled() {
    assert_filter_shape::<gpui_table_component::DateRangeFilter, chrono::NaiveDate>();

    assert_eq!(
        <gpui_table_component::DateRangeFilter as ComponentShapeMetadata>::MCP_INPUT.input_shape(),
        McpInputShape::Range(McpRangeBoundKind::Date)
    );
}

#[cfg(feature = "rust_decimal")]
#[test]
fn facade_runtime_accepts_builtin_number_range_filter_shape_when_enabled() {
    assert_filter_shape::<gpui_table_component::NumberRangeFilter, rust_decimal::Decimal>();

    assert_eq!(
        <gpui_table_component::NumberRangeFilter as ComponentShapeMetadata>::MCP_INPUT
            .input_shape(),
        McpInputShape::Range(McpRangeBoundKind::Decimal)
    );
}
