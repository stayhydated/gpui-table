# gpui-table Architecture

## Purpose

`gpui-table` is the public facade crate for the workspace. Its job is not to
own much logic; its job is to present a stable dependency and path surface for
application code and for code emitted by `gpui-table-derive`.

## Dependency Edges

- Always depends on `gpui-table-core`, `gpui-table-runtime`, and `gpui-table-schema`.
- Optionally depends on `gpui-table-derive` behind the `derive` feature.
- Re-exports external feature-gated crates through `__deps` so macro-generated
  code can name `chrono` and `rust_decimal` without each downstream crate
  having to mirror those dependency paths.
- Re-exports the load-more bridge through `__private` for macro internals.

## Public Surface Owned Here

`src/lib.rs` defines the stable namespace layout that downstream code and macro
expansion depend on:

- `gpui_table::core`
- `gpui_table::filter`
- `gpui_table::runtime`
- `gpui_table::schema`
- `gpui_table::registry`

It also re-exports the commonly used runtime traits at the crate root and, when
`derive` is enabled, the proc macros from `gpui-table-derive`.

## Internal Contracts

- The facade namespace is part of the generated-code contract. If a re-export
  moves or is renamed here, the derive crate must be updated in lockstep.
- `gpui_table::runtime::generated_filters` is the stable runtime target for
  generated filter code. The facade must continue re-exporting the runtime crate
  unchanged enough for that path to remain valid.
- `__deps` and `__private` are hidden from normal user documentation, but they
  are still semver-sensitive because proc-macro output depends on them.
- Feature flags on this crate are fan-out switches. Their main job is to keep
  `core`, `runtime`, and `derive` on the same capability set.

## Data Flow

1. Downstream code derives `GpuiTable`, `Filterable`, or `TableCell` through this crate.
1. `gpui-table-derive` emits code against the `gpui_table::core`,
   `gpui_table::runtime`, `gpui_table::schema`, and `gpui_table::registry`
   namespaces defined here.
1. Runtime code then executes inside `gpui-table-runtime`, while filter values
   and metadata route through `gpui-table-core` and `gpui-table-schema`.

## Feature Gates

- `derive` adds the proc-macro re-exports.
- `chrono`, `rust_decimal`, and `spacetimedb` forward feature support through
  the workspace layers.
- `fluent` forwards typed `es-fluent` label/title support through the core and
  derive layers. Generated code localizes through
  `gpui_table::runtime::generated_filters::localize_*` helpers.
- `inventory` enables registry metadata emission from the derive layer.
