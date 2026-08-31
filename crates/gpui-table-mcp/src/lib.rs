//! Experimental MCP query integration for generated `gpui-table` filters.
//!
//! This crate intentionally keeps GPUI out of the query execution path. It
//! owns table-specific filter decoding and query contracts while delegating
//! shared MCP server and stdio serving mechanics to `component-shape-mcp`.
//! MCP servers retain their shared query registry for the host lifetime;
//! completing a query never requests shutdown.

use std::{collections::BTreeSet, fmt, future::Future, marker::PhantomData, pin::Pin, sync::Arc};

pub use gpui_table_runtime::shape::ComponentShapeMetadata;
use gpui_table_runtime::shape::GpuiTableFilterShape;
use gpui_table_schema::registry::{RegistryFilterType, RustPath, RustType};
pub use serde::Serialize;
use serde_json::{Map, Value, json};

pub type FilterSchemaFn = fn(McpTableFilter) -> McpSchema;
pub type McpTableRowSchemaFn = fn() -> McpSchema;

pub use component_shape::{McpInput, McpInputShape, McpPrimitiveKind, McpRangeBoundKind};
pub use component_shape_mcp::{
    ContentBlock, MCP_PROTOCOL_VERSION, MCP_VALIDATION_PARAMS_NONE, McpAny, McpArguments,
    McpJsonSchema, McpPromptArgument, McpPromptResult, McpRange, McpSchema, McpSchemaProperties,
    McpServer, McpServerBuilder, McpToolAnnotations, McpToolArguments, McpToolCall, McpToolError,
    McpToolInput, McpToolMetadata, McpToolRegistry, McpToolValue, McpTypedTool, McpValidationIssue,
    McpValidationParam, McpValidationRule, McpValidationScope, McpValidationTypeArgMode,
    PromptDefinition, ResourceDefinition, ServeStdioResult, ToolCallResult, ToolDefinition,
    object_schema, serde, serde_json, validation_issues_error,
};
pub use rmcp;

mod descriptor;
mod filter_shape;
mod prompts;
mod query;
pub mod registry;
mod resources;
mod schema;
mod server;

pub use descriptor::*;
pub use filter_shape::*;
pub use prompts::*;
pub use query::*;
pub use resources::*;
pub use schema::*;
pub use server::*;

#[cfg(test)]
mod tests;
