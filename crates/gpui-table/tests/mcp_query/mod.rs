mod filtering_validation;
mod handler_integration;
mod pagination_errors;
mod resources_registration;
mod schema_metadata;

use chrono::NaiveDate;
use gpui_table::{
    Filterable, GpuiTable, GpuiTableFilterShape, TableCell,
    mcp::{McpServer, serde_json::json},
};
use koruma::NewtypeTryFromInner as _;
use koruma_collection::collection::LenValidation;
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Debug,
    Eq,
    Filterable,
    Hash,
    gpui_table::mcp::McpJsonSchema,
    PartialEq,
    Serialize,
    TableCell,
)]
enum UserStatus {
    Active,
    Blocked,
}

#[derive(Clone, Debug, GpuiTable, gpui_table::mcp::McpJsonSchema, Serialize)]
#[gpui_table(
    id = "users",
    title = "Users",
    filters,
    mcp(
        row_schema,
        name = "query_users",
        title = "Query users",
        description = "Exercise explicit table MCP metadata.",
        read_only = true,
        destructive = false,
        idempotent = true,
        open_world = true
    )
)]
struct UserRow {
    #[gpui_table(filter(gpui_table_component::TextFilter))]
    name: String,

    #[gpui_table(filter(gpui_table_component::FacetedFilter::<UserStatus>))]
    status: UserStatus,

    #[gpui_table(filter(gpui_table_component::NumberRangeFilter))]
    age: u8,

    #[gpui_table(filter(gpui_table_component::DateRangeFilter))]
    created_on: NaiveDate,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(
    id = "no_filter_query_rows",
    title = "No Filter Query Rows",
    mcp(
        name = "query_rows_no_filters",
        title = "Query rows without filters",
        description = "Exercise pagination-only MCP behavior."
    )
)]
struct NoFilterQueryRow {
    name: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct PrefixText(String);

impl gpui_table::mcp::McpToolValue for PrefixText {
    fn tool_value_schema() -> gpui_table::mcp::McpSchema {
        gpui_table::mcp::McpSchema::string().with_extension("x-prefixText", json!(true))
    }

    fn from_tool_value(
        field: &str,
        value: gpui_table::mcp::serde_json::Value,
    ) -> Result<Self, gpui_table::mcp::McpToolError> {
        let raw = value
            .as_str()
            .ok_or_else(|| gpui_table::mcp::McpToolError::decode(field, "expected string"))?;
        Ok(Self(raw.to_string()))
    }
}

#[derive(GpuiTableFilterShape)]
#[gpui_table_filter_shape(
    base = gpui_table_component::TextFilter,
    raw_value = PrefixText,
    field = String,
    into_base = |value: PrefixText| value.0,
    from_base = PrefixText
)]
struct PrefixTextFilter;

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(
    id = "prefix_filter_rows",
    title = "Prefix Filter Rows",
    filters,
    mcp(name = "query_prefix_filter_rows")
)]
struct PrefixFilterRow {
    #[gpui_table(filter(PrefixTextFilter))]
    name: String,
}

#[derive(
    Clone,
    Debug,
    Default,
    Deserialize,
    Eq,
    gpui_table::mcp::McpJsonSchema,
    koruma::Koruma,
    PartialEq,
    Serialize,
    TableCell,
)]
#[serde(transparent)]
#[mcp(crate = gpui_table::mcp, transparent)]
#[koruma(try_new, newtype)]
struct ValidatedPrefixText(#[koruma(LenValidation::<_>.min(2).max(64))] String);

#[derive(GpuiTableFilterShape)]
#[gpui_table_filter_shape(
    base = gpui_table_component::TextFilter,
    field = ValidatedPrefixText,
    koruma_newtype
)]
struct NewtypePrefixTextFilter;

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(
    id = "newtype_validated_filter_rows",
    title = "Newtype Validated Filter Rows",
    filters,
    mcp(name = "query_newtype_validated_filter_rows")
)]
struct NewtypeValidatedFilterRow {
    #[gpui_table(filter(NewtypePrefixTextFilter))]
    #[koruma(newtype)]
    name: ValidatedPrefixText,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(
    id = "validated_filter_rows",
    title = "Validated Filter Rows",
    filters,
    mcp(name = "query_validated_filter_rows")
)]
struct ValidatedFilterRow {
    #[gpui_table(filter(gpui_table_component::TextFilter))]
    #[koruma(LenValidation::<_>.min(2).max(5))]
    name: String,
}

fn test_server() -> McpServer {
    McpServer::new("gpui-table-mcp-test", "0.0.0")
}

fn user_rows() -> Result<Vec<UserRow>, String> {
    Ok(vec![
        UserRow {
            name: "Ann".to_string(),
            status: UserStatus::Active,
            age: 24,
            created_on: date(2026, 6, 1),
        },
        UserRow {
            name: "Annie".to_string(),
            status: UserStatus::Active,
            age: 34,
            created_on: date(2026, 5, 20),
        },
        UserRow {
            name: "Bea".to_string(),
            status: UserStatus::Blocked,
            age: 41,
            created_on: date(2025, 12, 15),
        },
    ])
}

fn no_filter_rows() -> Result<Vec<NoFilterQueryRow>, String> {
    Ok(vec![
        NoFilterQueryRow {
            name: "Ann".to_string(),
        },
        NoFilterQueryRow {
            name: "Annie".to_string(),
        },
        NoFilterQueryRow {
            name: "Bea".to_string(),
        },
    ])
}

fn prefix_filter_rows() -> Result<Vec<PrefixFilterRow>, String> {
    Ok(vec![
        PrefixFilterRow {
            name: "Ann".to_string(),
        },
        PrefixFilterRow {
            name: "Bea".to_string(),
        },
    ])
}

fn validated_filter_rows() -> Result<Vec<ValidatedFilterRow>, String> {
    Ok(vec![ValidatedFilterRow {
        name: "Ann".to_string(),
    }])
}

fn newtype_validated_filter_rows() -> Result<Vec<NewtypeValidatedFilterRow>, String> {
    Ok(vec![NewtypeValidatedFilterRow {
        name: ValidatedPrefixText::try_from_inner("Ann".to_string())
            .expect("fixture should be valid"),
    }])
}

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("test date should be valid")
}
