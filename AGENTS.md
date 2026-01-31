# AGENTS

## Scope and priorities
- Ignore all folders matching "**/__crate_paths/**" (generated; update via `just update_crate_paths`).
- This is a Rust workspace for gpui table derive macros, core traits, UI components, and prototyping helpers.

## Repo layout
- `crates/gpui-table`: facade crate that re-exports core traits, derive macros, and optional components
- `crates/gpui-table-core`: traits, filter types, registry metadata
- `crates/gpui-table-derive`: proc-macros (`GpuiTable`, `TableCell`, `gpui_table_impl`)
- `crates/gpui-table-component`: GPUI filter components and status bar
- `crates/gpui-table-prototyping-core`: codegen helpers for prototyping
- `examples/`: sample apps and storybook output

## Commands
- `just fmt` (format Rust, TOML, and Markdown)
- `just check`
- `just clippy`
- `just test`

## Notes
- Do not edit generated `__crate_paths` files by hand; use `just update_crate_paths`.
- When changing public APIs or behavior in a crate, update that crate's `docs/ARCHITECTURE.md`.
- Prefer workspace dependencies and feature flags from the root `Cargo.toml`.
