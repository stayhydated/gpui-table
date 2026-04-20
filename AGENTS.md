# AGENTS.md

This file is the working guide for contributors and coding agents in the
`gpui-table` workspace.

Use it to answer three questions quickly:

1. Where does this documentation belong?
2. Which crates are the default entry points vs integration points vs internals?
3. What other surfaces must be updated in the same change?

## Project summary

`gpui-table` is a Rust table ecosystem centered on strongly typed GPUI tables.

Its priorities are:

1. **Type safety**: keep generated columns, filters, delegates, and metadata strongly typed.
2. **Ergonomics**: keep `#[derive(GpuiTable)]`, `#[derive(Filterable)]`, and `#[gpui_table_impl]` concise enough for normal application use.
3. **Developer experience**: support built-in filters, inventory-backed table shapes, storybook examples, and prototyping/codegen workflows.

For most application code, start with `crates/gpui-table`.

Reach for `crates/gpui-table-component` when you need direct control over the
built-in filter widgets or `TableStatusBar`.

Reach for `crates/gpui-table-prototyping-core` when you are generating table
stories or scaffolding from inventory-registered shapes.

## Audience labels

These labels describe the crate or surface itself, not the documentation file
you are editing:

- **User-facing**: normal entry points for application developers.
- **Public integration**: public crates meant for extensions, tooling, or deeper customization, but not usually the default starting point.
- **Internal**: workspace plumbing, generated outputs, examples-as-infrastructure, and maintenance tooling.

## Documentation rules

### User-facing documentation

These surfaces are user-facing:

- the root `README.md`,
- `examples/README.md`,
- crate-level `README.md` files.

Even for public-integration crates, a `README.md` should explain:

- who the crate is for,
- what it does,
- what most users should use instead.

### Internal documentation

Only `docs/ARCHITECTURE.md` files are internal documentation.

Use them for:

- macro expansion and parsing details,
- subsystem boundaries,
- generated-code and runtime contracts,
- data flow,
- design rationale,
- inter-crate relationships.

Do not put implementation detail into READMEs.

## Synchronization rules

When changing a public workflow, feature-flag story, derive syntax, generated
filter behavior, registry metadata shape, or user-visible API shape:

1. Update the root `README.md`.
2. Update the affected crate `README.md` files.
3. Update `examples/README.md` and the relevant example crates when behavior is demonstrated there.
4. Keep these surfaces aligned in the same change unless there is a documented reason not to.

Additional rules:

- User-facing documentation should be example-first.
- Prefer a Rust snippet over prose-only explanations when showing derive, filter, loader, or registry behavior.
- `examples/some-lib` and `examples/some-lib-tables` are the canonical end-to-end usage examples.
- If inventory or codegen behavior changes, update `examples/prototyping` and regenerate `examples/prototyping/output`.
- `examples/prototyping/output` and all `**/__crate_paths/**` directories are generated surfaces; regenerate them instead of hand-editing them.
- When changing public APIs or behavior in a crate, update that crate's `docs/ARCHITECTURE.md`.

## Workspace map

### Main user-facing entry points

- `crates/gpui-table`
  Audience: **User-facing**
  Docs: [Architecture](crates/gpui-table/docs/ARCHITECTURE.md)
  Role: workspace facade, default entry point, and home of the public feature gates. Re-exports the core/runtime/schema namespaces and, with `derive`, the proc macros.

- `crates/gpui-table-component`
  Audience: **User-facing**
  Docs: [Architecture](crates/gpui-table-component/docs/ARCHITECTURE.md)
  Role: built-in GPUI filter widgets and `TableStatusBar` for teams that need direct UI composition outside fully derive-generated flows. Most users should still start with `gpui-table`.

### Public integration crates

- `crates/gpui-table-core`
  Audience: **Public integration**
  Docs: [Architecture](crates/gpui-table-core/docs/ARCHITECTURE.md)
  Role: pure filter semantics, typed filter values, and feature-gated conversion helpers. Most application users should start with `gpui-table` instead.

- `crates/gpui-table-runtime`
  Audience: **Public integration**
  Docs: [Architecture](crates/gpui-table-runtime/docs/ARCHITECTURE.md)
  Role: GPUI-facing row traits, default cell rendering, load-more support, and the stable generated-filter runtime facade. Most application users should start with `gpui-table` instead.

- `crates/gpui-table-schema`
  Audience: **Public integration**
  Docs: [Architecture](crates/gpui-table-schema/docs/ARCHITECTURE.md)
  Role: UI-neutral filter metadata and inventory-backed table-shape registry types used by tooling and generated flows. Most application users should start with `gpui-table` instead.

- `crates/gpui-table-derive`
  Audience: **Public integration**
  Docs: [Architecture](crates/gpui-table-derive/docs/ARCHITECTURE.md)
  Role: proc-macro crate for `GpuiTable`, `Filterable`, `TableCell`, and `gpui_table_impl`. Most users should depend on `gpui-table` instead of this crate directly.

- `crates/gpui-table-prototyping-core`
  Audience: **Public integration**
  Docs: [Architecture](crates/gpui-table-prototyping-core/docs/ARCHITECTURE.md)
  Role: code-generation helpers that consume `GpuiTableShape` inventory metadata to generate GPUI table stories and scaffolding. Most application users should add this only when building tooling or prototypes.

### Internal tooling and generated surfaces

- `justfile`
  Audience: **Internal**
  Role: workspace maintenance commands for formatting, linting, tests, dry-run publishing, and refreshing generated crate-path snapshots.

- `examples/prototyping/output`
  Audience: **Internal**
  Role: generated story modules emitted by `examples/prototyping`. Regenerate with `cargo run -p prototyping` instead of editing by hand.

- `**/__crate_paths/**`
  Audience: **Internal**
  Role: generated crate-path snapshots used by proc-macro support code. Ignore them during normal edits and regenerate with `just update_crate_paths` when needed.

- `crates/gpui-table/tests/ui`
  Audience: **Internal**
  Role: compile-fail coverage for derive diagnostics and feature-gated macro behavior.

- `crates/gpui-table/wip`
  Audience: **Internal**
  Role: scratch stderr artifacts for macro work. Treat this as maintenance data, not user-facing documentation.

### Examples and supporting surfaces

- `examples/README.md`
  Canonical index of runnable examples. Keep this aligned with the root `README.md`.

- `examples/some-lib`
  Shared domain types, derived tables, and filterable enums used by the example app and prototyping generator.

- `examples/some-lib-tables`
  Storybook-style GPUI app for exercising generated tables and filters. `cargo run` from the workspace root lands here by default.

- `examples/prototyping`
  Inventory-driven generator that writes story modules into `examples/prototyping/output`.

- `examples/i18n`
  Shared Fluent translation assets used by the example crates.

## Working rules by change type

### When editing docs

- Keep READMEs user-facing.
- Move macro expansion details, generated-code internals, and subsystem design into `docs/ARCHITECTURE.md`.
- Prefer examples over prose-only explanations.
- Sync the root `README.md`, affected crate `README.md` files, and `examples/README.md` in the same change when public behavior changes.
- If the change affects runnable flows, update the relevant example crates too.

### When editing Rust crates

- Use `cargo` for build, test, and run tasks.
- Keep dependency versions in the workspace root `Cargo.toml`.
- Use `workspace = true` in member crates.
- Prefer workspace dependencies and shared feature wiring from the root `Cargo.toml`.
- Non-example crates should reference workspace crates with `workspace = true`, not explicit paths.

### When editing derives, filters, or registry metadata

- Keep `gpui-table`, `gpui-table-derive`, `gpui-table-runtime`, and `gpui-table-schema` aligned when generated code contracts change.
- If `GpuiTableShape`, `ColumnVariant`, `FilterVariant`, or inventory registration changes, update the relevant README files, architecture docs, and regenerate `examples/prototyping/output`.
- If built-in filter behavior or query-value behavior changes, update `crates/gpui-table-component` stories and any affected example tables in `examples/some-lib-tables`.
- If `fluent` behavior changes, update the relevant `i18n/` assets in crates and examples.

### When writing tests

- Prefer `trybuild` for proc-macro compile errors and keep `.stderr` fixtures aligned.
- Prefer [insta](https://insta.rs/) snapshots when verifying structured table-rendering output is clearer than assertion-heavy unit tests.
- Use raw multiline strings or `quote! { ... }` for embedded Rust code in macro tests instead of heavily escaped single-line literals.
