use std::collections::HashSet;

use gpui_table_core::filter::Filterable;
use gpui_table_mcp::{
    McpAny, McpFilterShape, McpFilterShapeValidation, McpTableFilter, McpToolError, McpToolValue,
    decode_raw_filter_shape, decode_raw_filter_shape_with_validation, default_filter_input_schema,
    default_filter_shape_input_schema,
};
use gpui_table_runtime::shape::GpuiTableFilterShape;

use crate::{FacetedFilter, TextFilter, TextFilterAdapter};

impl McpFilterShape for TextFilter {
    fn input_schema(filter: McpTableFilter) -> gpui_table_mcp::McpSchema {
        default_filter_shape_input_schema::<Self>(filter)
    }

    fn decode_filter(
        field: &'static str,
        value: McpAny,
    ) -> Result<Self::FilterValue, McpToolError> {
        decode_raw_filter_shape::<Self>(field, value)
    }
}

impl McpFilterShapeValidation for TextFilter {
    fn decode_filter_with_validation<Validate>(
        field: &'static str,
        value: McpAny,
        validate: Validate,
    ) -> Result<Self::FilterValue, McpToolError>
    where
        Validate: FnOnce(&Self::RawValue) -> Result<(), McpToolError>,
    {
        decode_raw_filter_shape_with_validation::<Self, _>(field, value, validate)
    }
}

impl McpFilterShape for TextFilterAdapter {
    fn input_schema(filter: McpTableFilter) -> gpui_table_mcp::McpSchema {
        <TextFilter as McpFilterShape>::input_schema(filter)
    }

    fn decode_filter(
        field: &'static str,
        value: McpAny,
    ) -> Result<Self::FilterValue, McpToolError> {
        <TextFilter as McpFilterShape>::decode_filter(field, value)
    }
}

impl McpFilterShapeValidation for TextFilterAdapter {
    fn decode_filter_with_validation<Validate>(
        field: &'static str,
        value: McpAny,
        validate: Validate,
    ) -> Result<Self::FilterValue, McpToolError>
    where
        Validate: FnOnce(&Self::RawValue) -> Result<(), McpToolError>,
    {
        <TextFilter as McpFilterShapeValidation>::decode_filter_with_validation(
            field, value, validate,
        )
    }
}

impl<T> McpFilterShape for FacetedFilter<T>
where
    T: Filterable,
{
    fn input_schema(filter: McpTableFilter) -> gpui_table_mcp::McpSchema {
        let mut schema = default_filter_input_schema(filter);
        let options = T::options();

        if let Some(object) = schema.as_object_mut() {
            if !options.is_empty()
                && let Some(items) = object
                    .get_mut("items")
                    .and_then(gpui_table_mcp::serde_json::Value::as_object_mut)
            {
                items.insert(
                    "enum".to_string(),
                    gpui_table_mcp::serde_json::Value::Array(
                        options
                            .iter()
                            .map(|option| {
                                gpui_table_mcp::serde_json::Value::String(option.value.clone())
                            })
                            .collect(),
                    ),
                );
            }
            object.insert(
                "x-gpuiTableFacetOptions".to_string(),
                gpui_table_mcp::serde_json::Value::Array(
                    options
                        .into_iter()
                        .map(|option| {
                            gpui_table_mcp::serde_json::json!({
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

    fn decode_filter(
        field: &'static str,
        value: McpAny,
    ) -> Result<Self::FilterValue, McpToolError> {
        let raw_values = <Vec<String> as McpToolValue>::from_tool_value(field, value.into_value())?;
        let mut values = HashSet::new();
        for raw_value in raw_values {
            let value = T::from_filter_string(&raw_value)
                .ok_or_else(|| McpToolError::invalid_field_value(field, raw_value))?;
            values.insert(value);
        }
        Ok(<Self as GpuiTableFilterShape>::wrap_value(values))
    }
}

impl<T> McpFilterShapeValidation for FacetedFilter<T>
where
    T: Filterable,
{
    fn decode_filter_with_validation<Validate>(
        field: &'static str,
        value: McpAny,
        validate: Validate,
    ) -> Result<Self::FilterValue, McpToolError>
    where
        Validate: FnOnce(&Self::RawValue) -> Result<(), McpToolError>,
    {
        let raw_values = <Vec<String> as McpToolValue>::from_tool_value(field, value.into_value())?;
        let mut values = HashSet::new();
        for raw_value in raw_values {
            let value = T::from_filter_string(&raw_value)
                .ok_or_else(|| McpToolError::invalid_field_value(field, raw_value))?;
            values.insert(value);
        }
        validate(&values)?;
        Ok(<Self as GpuiTableFilterShape>::wrap_value(values))
    }
}

#[cfg(feature = "rust_decimal")]
mod number_range {
    use gpui_table_mcp::{
        McpAny, McpFilterShape, McpFilterShapeValidation, McpTableFilter, McpToolError,
        decode_range_filter, range_filter_input_schema,
    };
    use gpui_table_runtime::shape::GpuiTableFilterShape;

    use crate::{NumberRangeFilter, NumberRangeFilterAdapter};

    impl McpFilterShape for NumberRangeFilter {
        fn input_schema(filter: McpTableFilter) -> gpui_table_mcp::McpSchema {
            range_filter_input_schema::<rust_decimal::Decimal>(filter)
        }

        fn decode_filter(
            field: &'static str,
            value: McpAny,
        ) -> Result<Self::FilterValue, McpToolError> {
            let value = decode_range_filter::<rust_decimal::Decimal>(field, value)?;
            Ok(<Self as GpuiTableFilterShape>::wrap_value(value))
        }
    }

    impl McpFilterShapeValidation for NumberRangeFilter {
        fn decode_filter_with_validation<Validate>(
            field: &'static str,
            value: McpAny,
            validate: Validate,
        ) -> Result<Self::FilterValue, McpToolError>
        where
            Validate: FnOnce(&Self::RawValue) -> Result<(), McpToolError>,
        {
            let value = decode_range_filter::<rust_decimal::Decimal>(field, value)?;
            validate(&value)?;
            Ok(<Self as GpuiTableFilterShape>::wrap_value(value))
        }
    }

    impl McpFilterShape for NumberRangeFilterAdapter {
        fn input_schema(filter: McpTableFilter) -> gpui_table_mcp::McpSchema {
            <NumberRangeFilter as McpFilterShape>::input_schema(filter)
        }

        fn decode_filter(
            field: &'static str,
            value: McpAny,
        ) -> Result<Self::FilterValue, McpToolError> {
            <NumberRangeFilter as McpFilterShape>::decode_filter(field, value)
        }
    }

    impl McpFilterShapeValidation for NumberRangeFilterAdapter {
        fn decode_filter_with_validation<Validate>(
            field: &'static str,
            value: McpAny,
            validate: Validate,
        ) -> Result<Self::FilterValue, McpToolError>
        where
            Validate: FnOnce(&Self::RawValue) -> Result<(), McpToolError>,
        {
            <NumberRangeFilter as McpFilterShapeValidation>::decode_filter_with_validation(
                field, value, validate,
            )
        }
    }
}

#[cfg(feature = "chrono")]
mod date_range {
    use gpui_table_mcp::{
        McpAny, McpFilterShape, McpFilterShapeValidation, McpTableFilter, McpToolError,
        decode_range_filter, range_filter_input_schema,
    };
    use gpui_table_runtime::shape::GpuiTableFilterShape;

    use crate::{DateRangeFilter, DateRangeFilterAdapter};

    impl McpFilterShape for DateRangeFilter {
        fn input_schema(filter: McpTableFilter) -> gpui_table_mcp::McpSchema {
            range_filter_input_schema::<chrono::NaiveDate>(filter)
        }

        fn decode_filter(
            field: &'static str,
            value: McpAny,
        ) -> Result<Self::FilterValue, McpToolError> {
            let value = decode_range_filter::<chrono::NaiveDate>(field, value)?;
            Ok(<Self as GpuiTableFilterShape>::wrap_value(value))
        }
    }

    impl McpFilterShapeValidation for DateRangeFilter {
        fn decode_filter_with_validation<Validate>(
            field: &'static str,
            value: McpAny,
            validate: Validate,
        ) -> Result<Self::FilterValue, McpToolError>
        where
            Validate: FnOnce(&Self::RawValue) -> Result<(), McpToolError>,
        {
            let value = decode_range_filter::<chrono::NaiveDate>(field, value)?;
            validate(&value)?;
            Ok(<Self as GpuiTableFilterShape>::wrap_value(value))
        }
    }

    impl McpFilterShape for DateRangeFilterAdapter {
        fn input_schema(filter: McpTableFilter) -> gpui_table_mcp::McpSchema {
            <DateRangeFilter as McpFilterShape>::input_schema(filter)
        }

        fn decode_filter(
            field: &'static str,
            value: McpAny,
        ) -> Result<Self::FilterValue, McpToolError> {
            <DateRangeFilter as McpFilterShape>::decode_filter(field, value)
        }
    }

    impl McpFilterShapeValidation for DateRangeFilterAdapter {
        fn decode_filter_with_validation<Validate>(
            field: &'static str,
            value: McpAny,
            validate: Validate,
        ) -> Result<Self::FilterValue, McpToolError>
        where
            Validate: FnOnce(&Self::RawValue) -> Result<(), McpToolError>,
        {
            <DateRangeFilter as McpFilterShapeValidation>::decode_filter_with_validation(
                field, value, validate,
            )
        }
    }
}
