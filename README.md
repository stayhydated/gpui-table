# gpui-table

[![Build Status](https://github.com/stayhydated/gpui-table/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/gpui-table/actions/workflows/ci.yml)
[![Docs](https://docs.rs/gpui-table/badge.svg)](https://docs.rs/gpui-table/)
[![Crates.io](https://img.shields.io/crates/v/gpui-table.svg)](https://crates.io/crates/gpui-table)

# Project Overview

`gpui-table` is a table ecosystem written in **Rust**, built on top of `gpui`
and `gpui-component`. It focuses on:

1. **Type Safety**: Derive macros generate strongly-typed columns, filters, and metadata.
1. **Ergonomics**: `#[derive(GpuiTable)]` and `#[gpui_table_impl]` minimize boilerplate.
1. **Developer Experience**: Inventory-based shape registry (via the `inventory` feature)
   enables prototyping and codegen.

## Architecture Documentation Index

| Crate | Link to Architecture Doc | Purpose |
| --- | --- | --- |
| **Core** | | |
| `gpui-table` | [Architecture](crates/gpui-table/docs/ARCHITECTURE.md) | Facade crate; re-exports macros/core/components. |
| `gpui-table-core` | [Architecture](crates/gpui-table-core/docs/ARCHITECTURE.md) | Core traits, filter types, and registry metadata. |
| `gpui-table-derive` | [Architecture](crates/gpui-table-derive/docs/ARCHITECTURE.md) | Proc macros for table derivation and load-more wiring. |
| **Components & Runtime** | | |
| `gpui-table-component` | [Architecture](crates/gpui-table-component/docs/ARCHITECTURE.md) | GPUI filter components and status bar. |
| **Prototyping** | | |
| `gpui-table-prototyping-core` | [Architecture](crates/gpui-table-prototyping-core/docs/ARCHITECTURE.md) | Codegen from inventory shapes for prototyping. |

## Crate Descriptions

### Core Layers

- **`gpui-table`**: User-facing facade. Re-exports derive macros, core metadata, and
  optional UI components.
- **`gpui-table-core`**: Shared metadata and traits for table rows, filters, and registry.
- **`gpui-table-derive`**: Proc macros that expand row structs into columns, delegates,
  filters, and optional inventory registrations.

### Components & Runtime

- **`gpui-table-component`**: GPUI UI components for text, faceted, number-range, and
  date-range filters, plus `TableStatusBar`.

### Prototyping

- **`gpui-table-prototyping-core`**: Builds GPUI table scaffolding by consuming
  `GpuiTableShape` inventory data.

## Development

| Task | Command |
| --- | --- |
| Format | `just fmt` |
| Check | `just check` |
| Clippy | `just clippy` |
| Tests | `just test` |
| Update crate paths | `just update_crate_paths` |

## Agent Notes

- Ignore all folders matching `**/__crate_paths/**` (generated files).
- Prefer `rg` for search and keep edits minimal.
- When changing public APIs or behavior in a crate, update that crate's `docs/ARCHITECTURE.md`.
