# Project Overview

`gpui-table` is a table ecosystem written in **Rust**, built on top of `gpui`
and `gpui-component`. It focuses on:

1. **Type Safety**: Derive macros generate strongly-typed columns, filters, and metadata.
1. **Ergonomics**: `#[derive(GpuiTable)]`, `#[derive(Filterable)]`, and `#[gpui_table_impl]`
   minimize boilerplate.
1. **Developer Experience**: Inventory-registered table shapes (via the
   `inventory` feature) enable prototyping and codegen.

## Architecture Documentation Index

| Crate | Link to Architecture Doc | Purpose |
| --- | --- | --- |
| **Core** | | |
| `gpui-table` | [Architecture](crates/gpui-table/docs/ARCHITECTURE.md) | Facade crate; re-exports the core/runtime/schema namespaces and, with `derive`, the macros. |
| `gpui-table-core` | [Architecture](crates/gpui-table-core/docs/ARCHITECTURE.md) | Pure filter semantics, typed filter values, and conversion traits. |
| `gpui-table-derive` | [Architecture](crates/gpui-table-derive/docs/ARCHITECTURE.md) | Proc macros for table derivation, filterable enums, and load-more wiring. |
| **Components & Runtime** | | |
| `gpui-table-component` | [Architecture](crates/gpui-table-component/docs/ARCHITECTURE.md) | GPUI filter components and status bar. |
| `gpui-table-runtime` | [Architecture](crates/gpui-table-runtime/docs/ARCHITECTURE.md) | GPUI-facing row traits, default rendering, and generated-filter runtime glue. |
| `gpui-table-schema` | [Architecture](crates/gpui-table-schema/docs/ARCHITECTURE.md) | UI-neutral filter metadata and table-shape registry types. |
| **Prototyping** | | |
| `gpui-table-prototyping-core` | [Architecture](crates/gpui-table-prototyping-core/docs/ARCHITECTURE.md) | Codegen from inventory shapes for prototyping. |

## Crate Descriptions

### Core Layers

- **`gpui-table`**: User-facing facade. Re-exports the `core`, `runtime`, and
  `schema` namespaces plus the derive macros when the `derive` feature is enabled.
- **`gpui-table-core`**: Pure filter semantics, typed filter values, and
  feature-gated conversion helpers.
- **`gpui-table-derive`**: Proc macros that expand row structs into columns,
  delegates, filters, `Filterable` enums, and optional inventory registrations.

### Components & Runtime

- **`gpui-table-component`**: GPUI UI components for text, faceted, number-range, and
  date-range filters, plus `TableStatusBar`.
- **`gpui-table-runtime`**: GPUI-facing runtime traits, default cell rendering,
  load-more support, and the stable generated-filter facade.
- **`gpui-table-schema`**: Static metadata shared across the workspace,
  including `FilterConfig` and `GpuiTableShape`.

### Prototyping

- **`gpui-table-prototyping-core`**: Builds GPUI table scaffolding by consuming
  `GpuiTableShape` inventory data.

## Agent Notes

- Ignore all folders matching `**/__crate_paths/**` (generated files).
- When changing public APIs or behavior in a crate, update that crate's `docs/ARCHITECTURE.md`.
- Prefer workspace dependencies and feature flags from the root `Cargo.toml`.

## Skills

| Item | Link to llms.txt | Link to llms-full.txt | Purpose |
| -------------- | --------------------------------------------------------- | --------------------------------------------------------- | --------------------------- |
| **Crate** | | | |
| es-fluent | https://stayhydated.github.io/es-fluent/llms.txt | https://stayhydated.github.io/es-fluent/llms-full.txt | i18n |
| koruma | https://stayhydated.github.io/koruma/llms.txt | https://stayhydated.github.io/koruma/llms-full.txt | validation, newtype |
| gpui-component | https://longbridge.github.io/gpui-component/llms.txt | https://longbridge.github.io/gpui-component/llms-full.txt | gpui shadcn-like components |
