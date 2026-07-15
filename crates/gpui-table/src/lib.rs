//! Facade crate for the `gpui-table` ecosystem.
//!
//! Public API is namespaced by responsibility:
//! - `gpui_table::core` for pure filter semantics
//! - `gpui_table::mcp` for experimental MCP table query integration
//! - `gpui_table::runtime` for GPUI-facing traits/helpers
//! - `gpui_table::schema` for metadata and registry types
//! - root-level proc macros from `gpui-table-derive`, including
//!   `GpuiTableFilterShape` for adapting existing filter shapes
//!
//! Concrete filter widgets and component-owned helpers live in
//! `gpui-table-component` and are intentionally not re-exported from this
//! facade.
//!
//! The facade owns the stable paths that macro-generated code names. Changes to
//! root re-exports, `runtime::generated_filters`, `registry`, `__deps`, or
//! `__private` must stay aligned with `gpui-table-derive`, examples, and UI
//! compile-fail fixtures.
//!
//! # Quick Start
//!
//! ```ignore
//! use gpui_table::GpuiTable;
//!
//! #[derive(Clone, GpuiTable)]
//! struct User {
//!     name: String,
//!     age: u8,
//! }
//! ```
//!
//! # Filter Feature Requirements
//!
//! - `gpui_table_component::NumberRangeFilter` requires the `rust_decimal` feature.
//! - `gpui_table_component::DateRangeFilter` requires the `chrono` feature.
//! - Filtering SpacetimeDB `Timestamp`/`TimeDuration` with range filters requires the `spacetimedb` feature.
//! - `#[gpui_table(filters)]` is required to generate filter entities and matching logic.
//!
//! These are validated during macro expansion with direct compile-time errors.

#[cfg(feature = "derive")]
pub use gpui_table_derive::*;

pub use gpui_table_core as core;
pub use gpui_table_core::filter;
#[cfg(feature = "mcp")]
pub use gpui_table_mcp as mcp;
pub use gpui_table_runtime as runtime;
pub use gpui_table_runtime::{
    FilterEntitiesExt, FilterSidebarData, FilterSidebarGroup, FilterSidebarItem, TableCell,
    TableDataLoader, TableId, TableLoader, TableRowContextMenu, TableRowGeneratedContextMenu,
    TableRowMeta, TableRowStyle,
};
pub use gpui_table_schema as schema;
pub use gpui_table_schema::registry;

/// Typed JSON codec used by generated table-filter presets.
pub trait FilterPresetValue: Default + Clone + Send + 'static {
    /// Encode this typed filter value for a saved preset.
    fn to_preset_json(&self) -> serde_json::Value;

    /// Decode and validate this typed filter value from a saved preset.
    fn from_preset_json(value: &serde_json::Value) -> Result<Self, String>;
}

impl FilterPresetValue for filter::TextValue {
    fn to_preset_json(&self) -> serde_json::Value {
        serde_json::Value::String(self.0.clone())
    }

    fn from_preset_json(value: &serde_json::Value) -> Result<Self, String> {
        value
            .as_str()
            .map(Self::from)
            .ok_or_else(|| "text filter preset must be a string".to_string())
    }
}

impl<T> FilterPresetValue for filter::FacetedValue<T>
where
    T: filter::FilterValue + Clone + Send + 'static,
{
    fn to_preset_json(&self) -> serde_json::Value {
        let mut values = self
            .iter()
            .map(filter::FilterValue::to_filter_string)
            .collect::<Vec<_>>();
        values.sort();
        serde_json::Value::Array(values.into_iter().map(serde_json::Value::String).collect())
    }

    fn from_preset_json(value: &serde_json::Value) -> Result<Self, String> {
        let values = value
            .as_array()
            .ok_or_else(|| "faceted filter preset must be an array".to_string())?;
        values
            .iter()
            .map(|value| {
                let value = value
                    .as_str()
                    .ok_or_else(|| "faceted filter preset entries must be strings".to_string())?;
                T::from_filter_string(value)
                    .ok_or_else(|| format!("invalid faceted filter preset value `{value}`"))
            })
            .collect::<Result<std::collections::HashSet<_>, _>>()
            .map(Self)
    }
}

impl<T> FilterPresetValue for filter::RangeValue<T>
where
    T: Clone + PartialOrd + ToString + std::str::FromStr + Send + 'static,
    T::Err: std::fmt::Display,
{
    fn to_preset_json(&self) -> serde_json::Value {
        serde_json::json!({
            "min": self.0.as_ref().map(ToString::to_string),
            "max": self.1.as_ref().map(ToString::to_string),
        })
    }

    fn from_preset_json(value: &serde_json::Value) -> Result<Self, String> {
        let object = value
            .as_object()
            .ok_or_else(|| "range filter preset must be an object".to_string())?;
        let parse_bound = |name: &str| -> Result<Option<T>, String> {
            match object.get(name) {
                None | Some(serde_json::Value::Null) => Ok(None),
                Some(serde_json::Value::String(value)) => value
                    .parse::<T>()
                    .map(Some)
                    .map_err(|error| format!("invalid range {name} `{value}`: {error}")),
                Some(_) => Err(format!(
                    "range filter preset `{name}` must be a string or null"
                )),
            }
        };
        Ok(Self(parse_bound("min")?, parse_bound("max")?))
    }
}

impl<T> FilterPresetValue for filter::SingleValue<T>
where
    T: filter::FilterValue + Clone + PartialEq + Send + 'static,
{
    fn to_preset_json(&self) -> serde_json::Value {
        self.value().map_or(serde_json::Value::Null, |value| {
            serde_json::Value::String(value.to_filter_string())
        })
    }

    fn from_preset_json(value: &serde_json::Value) -> Result<Self, String> {
        match value {
            serde_json::Value::Null => Ok(Self::default()),
            serde_json::Value::String(value) => T::from_filter_string(value)
                .map(|value| Self(Some(value)))
                .ok_or_else(|| format!("invalid single filter preset value `{value}`")),
            _ => Err("single filter preset must be a string or null".to_string()),
        }
    }
}

#[cfg(test)]
mod filter_preset_tests {
    use super::{FilterPresetValue, filter};

    #[test]
    fn typed_filter_values_round_trip_through_json() {
        let text = filter::TextValue::from("needle");
        assert_eq!(
            filter::TextValue::from_preset_json(&text.to_preset_json()).unwrap(),
            text
        );

        let range = filter::RangeValue(Some(3_i32), Some(9_i32));
        assert_eq!(
            filter::RangeValue::<i32>::from_preset_json(&range.to_preset_json()).unwrap(),
            range
        );

        let facets = filter::FacetedValue(std::collections::HashSet::from([true, false]));
        assert_eq!(
            filter::FacetedValue::<bool>::from_preset_json(&facets.to_preset_json()).unwrap(),
            facets
        );
    }
}

/// Hidden dependency surface used by macro-generated code.
#[doc(hidden)]
pub mod __deps {
    #[cfg(feature = "chrono")]
    pub use chrono;
    #[cfg(feature = "rust_decimal")]
    pub use rust_decimal;
    pub use serde_json;
}

/// Hidden runtime bridge used by macro-generated code.
#[doc(hidden)]
pub mod __private {
    pub use gpui_table_runtime::__private::LoadMoreDelegate;
}
