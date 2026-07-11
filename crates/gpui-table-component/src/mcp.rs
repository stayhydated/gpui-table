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

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_table_mcp::serde_json::json;
    use gpui_table_schema::registry::RustType;

    fn table_filter<Shape: McpFilterShape>(name: &'static str) -> McpTableFilter {
        McpTableFilter::for_shape::<Shape>(name, RustType::from_macro_tokens_unchecked("String"))
    }

    #[test]
    fn text_filter_and_adapter_publish_schema_decode_and_validation_contracts() {
        let filter = table_filter::<TextFilter>("name");
        let schema = <TextFilter as McpFilterShape>::input_schema(filter);
        assert_eq!(schema["type"], "string");

        let decoded =
            <TextFilter as McpFilterShape>::decode_filter("name", McpAny::from(json!("Alice")))
                .unwrap();
        assert_eq!(decoded.as_ref(), "Alice");
        assert!(
            <TextFilter as McpFilterShape>::decode_filter("name", McpAny::from(json!(42)),)
                .is_err()
        );

        let validated = <TextFilter as McpFilterShapeValidation>::decode_filter_with_validation(
            "name",
            McpAny::from(json!("Alice")),
            |value| {
                assert_eq!(value, "Alice");
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(validated.as_ref(), "Alice");
        assert!(
            <TextFilter as McpFilterShapeValidation>::decode_filter_with_validation(
                "name",
                McpAny::from(json!("blocked")),
                |_| Err(McpToolError::invalid_field_value("name", "blocked")),
            )
            .is_err()
        );

        assert_eq!(
            <TextFilterAdapter as McpFilterShape>::input_schema(filter),
            schema
        );
        assert_eq!(
            <TextFilterAdapter as McpFilterShape>::decode_filter(
                "name",
                McpAny::from(json!("Bob")),
            )
            .unwrap()
            .as_ref(),
            "Bob"
        );
        assert_eq!(
            <TextFilterAdapter as McpFilterShapeValidation>::decode_filter_with_validation(
                "name",
                McpAny::from(json!("Bob")),
                |_| Ok(()),
            )
            .unwrap()
            .as_ref(),
            "Bob"
        );
    }

    #[test]
    fn faceted_filter_schema_lists_options_and_decodes_typed_sets() {
        let filter = table_filter::<FacetedFilter<bool>>("enabled");
        let schema = <FacetedFilter<bool> as McpFilterShape>::input_schema(filter);

        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["enum"], json!(["true", "false"]));
        assert_eq!(schema["x-gpuiTableFacetOptions"][0]["value"], "true");
        assert_eq!(schema["x-gpuiTableFacetOptions"][1]["value"], "false");

        let decoded = <FacetedFilter<bool> as McpFilterShape>::decode_filter(
            "enabled",
            McpAny::from(json!(["true", "true"])),
        )
        .unwrap();
        assert_eq!(decoded.0, HashSet::from([true]));
        assert!(
            <FacetedFilter<bool> as McpFilterShape>::decode_filter(
                "enabled",
                McpAny::from(json!(["unknown"])),
            )
            .is_err()
        );

        let validated =
            <FacetedFilter<bool> as McpFilterShapeValidation>::decode_filter_with_validation(
                "enabled",
                McpAny::from(json!(["false"])),
                |values| {
                    assert_eq!(values, &HashSet::from([false]));
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(validated.0, HashSet::from([false]));
        assert!(
            <FacetedFilter<bool> as McpFilterShapeValidation>::decode_filter_with_validation(
                "enabled",
                McpAny::from(json!(["true"])),
                |_| Err(McpToolError::invalid_field_value("enabled", "true")),
            )
            .is_err()
        );
    }

    #[cfg(feature = "rust_decimal")]
    #[test]
    fn number_range_filter_and_adapter_decode_bounds_and_validation() {
        use crate::{NumberRangeFilter, NumberRangeFilterAdapter};
        use rust_decimal::Decimal;

        let filter = table_filter::<NumberRangeFilter>("amount");
        let schema = <NumberRangeFilter as McpFilterShape>::input_schema(filter);
        assert_eq!(schema["type"], "object");

        let decoded = <NumberRangeFilter as McpFilterShape>::decode_filter(
            "amount",
            McpAny::from(json!({ "min": "1.5", "max": 3 })),
        )
        .unwrap();
        assert_eq!(decoded.min(), Some(&Decimal::new(15, 1)));
        assert_eq!(decoded.max(), Some(&Decimal::from(3)));

        let validated =
            <NumberRangeFilter as McpFilterShapeValidation>::decode_filter_with_validation(
                "amount",
                McpAny::from(json!({ "min": 1 })),
                |bounds| {
                    assert_eq!(bounds.0, Some(Decimal::ONE));
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(validated.min(), Some(&Decimal::ONE));

        assert_eq!(
            <NumberRangeFilterAdapter as McpFilterShape>::input_schema(filter),
            schema
        );
        assert!(
            <NumberRangeFilterAdapter as McpFilterShape>::decode_filter(
                "amount",
                McpAny::from(json!({ "max": 10 })),
            )
            .unwrap()
            .matches(&Decimal::from(5))
        );
        assert!(
            <NumberRangeFilterAdapter as McpFilterShapeValidation>::decode_filter_with_validation(
                "amount",
                McpAny::from(json!({ "max": 10 })),
                |_| Ok(()),
            )
            .is_ok()
        );
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn date_range_filter_and_adapter_decode_bounds_and_validation() {
        use crate::{DateRangeFilter, DateRangeFilterAdapter};
        use chrono::NaiveDate;

        let filter = table_filter::<DateRangeFilter>("created_on");
        let schema = <DateRangeFilter as McpFilterShape>::input_schema(filter);
        assert_eq!(schema["type"], "object");

        let decoded = <DateRangeFilter as McpFilterShape>::decode_filter(
            "created_on",
            McpAny::from(json!({ "min": "2026-01-01", "max": "2026-01-31" })),
        )
        .unwrap();
        assert_eq!(
            decoded.min(),
            Some(&NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
        );
        assert_eq!(
            decoded.max(),
            Some(&NaiveDate::from_ymd_opt(2026, 1, 31).unwrap())
        );

        assert!(
            <DateRangeFilter as McpFilterShapeValidation>::decode_filter_with_validation(
                "created_on",
                McpAny::from(json!({ "min": "2026-01-01" })),
                |_| Ok(()),
            )
            .is_ok()
        );
        assert_eq!(
            <DateRangeFilterAdapter as McpFilterShape>::input_schema(filter),
            schema
        );
        assert!(
            <DateRangeFilterAdapter as McpFilterShape>::decode_filter(
                "created_on",
                McpAny::from(json!({ "max": "2026-12-31" })),
            )
            .is_ok()
        );
        assert!(
            <DateRangeFilterAdapter as McpFilterShapeValidation>::decode_filter_with_validation(
                "created_on",
                McpAny::from(json!({ "max": "2026-12-31" })),
                |_| Ok(()),
            )
            .is_ok()
        );
    }
}
