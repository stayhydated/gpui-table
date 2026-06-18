# AGENTS.md

This is the working guide for contributors and coding agents in the
`gpui-table` workspace.

Use it to decide where a change belongs, which documentation and generated
surfaces must move with it, and which validation command should run before
handoff.

For most application code, start with `crates/gpui-table`.

Reach for `crates/gpui-table-component` when you need direct control over the
built-in filter widgets or `TableStatusBar`.

Reach for `crates/gpui-table-prototyping-core` when you are generating table
stories or scaffolding from inventory-registered shapes.

## Project Summary

`gpui-table` is a Rust table ecosystem centered on strongly typed GPUI tables.

Its priorities are:

1. **Type safety**: keep generated columns, filters, delegates, and metadata strongly typed.
2. **Ergonomics**: keep `#[derive(GpuiTable)]`, `#[derive(Filterable)]`, and `#[gpui_table_impl]` concise enough for normal application use.
3. **Developer experience**: support built-in filters, inventory-backed table shapes, storybook examples, and prototyping and codegen workflows.

## Quick Decision Flow

Before editing, classify the change:

1. **Find the surface in the workspace map.** Use its audience label to decide
   how much public explanation the change needs.
2. **Place documentation by content, not by crate audience.** README files,
   `examples/README.md`, `skills/use-gpui-table`, and
   `skills/use-gpui-table-component-shapes` are user-facing. Internal
   contracts belong next to code in rustdoc, tests, snapshots, or this routing
   guide.
3. **Sync public workflow changes.** If derive syntax, generated filter
   behavior, registry metadata shape, feature flags, generated output,
   localization, or recommended usage changes, update the relevant README,
   example, rustdoc, generated surface, `skills/use-gpui-table`, and
   `skills/use-gpui-table-component-shapes` guidance in the same change when
   applicable.
4. **Validate narrowly.** Run the smallest command that proves the edited
   behavior or documentation surface is still sound.

## Audience Labels

These labels describe the crate or surface itself, not the documentation file
being edited:

- **User-facing**: normal entry points for application developers.
- **Public integration**: public crates meant for extensions, tooling, or deeper customization. These are usually not the default starting point.
- **Internal**: workspace plumbing, generated outputs, examples-as-infrastructure, and maintenance tooling.

## Documentation Placement

Treat these surfaces as user-facing:

- the root `README.md`,
- `examples/README.md`,
- crate-level `README.md` files,
- `examples/some-lib` and `examples/some-lib-tables`,
- `examples/mcp-query`,
- `skills/use-gpui-table`,
- `skills/use-gpui-table-component-shapes`.

Even README files for public-integration crates should explain:

- who the crate is for,
- what it does,
- what most users should use instead.

Keep user-facing documentation example-first. Prefer Rust snippets over
prose-only explanations when showing derive, filter, loader, or registry
behavior.

### Internal Documentation

Keep implementation details out of user-facing READMEs unless they are required
to use the public API. Put durable internal contracts in crate or module
rustdocs near the code that enforces them. Prefer tests, compile-fail fixtures,
snapshots, and generated examples for executable contracts.

Use this guide only for cross-surface routing and synchronization rules. Do not
recreate long design notes here.

### Skill Guidance

`skills/use-gpui-table` and `skills/use-gpui-table-component-shapes` are public
application-developer guidance, not repo-local maintenance guidance. Keep
maintainer-only details in this guide, source rustdocs, tests, or generated
fixtures.

Update them when user-facing workflows, derive syntax, generated filter
behavior, registry metadata, generated output, localization patterns, feature
flags, or recommended usage change.

## Synchronization Rules

When a substantive change modifies a public workflow, feature-flag story,
derive syntax, generated filter behavior, registry metadata shape, or other
user-visible API shape:

1. Update the root `README.md`.
2. Update the affected crate `README.md` files.
3. Update `examples/README.md` and the relevant example crates when behavior is demonstrated there.
4. Update `skills/use-gpui-table` and `skills/use-gpui-table-component-shapes`
   when public usage guidance changes.
5. Keep these surfaces aligned in the same change unless there is a documented reason not to.

`examples/some-lib` and `examples/some-lib-tables` are the canonical
end-to-end usage examples.

If inventory or codegen behavior changes, update `examples/prototyping` and
regenerate `examples/prototyping/output`.

`examples/prototyping/output` is a generated surface; regenerate it instead of
hand-editing.

When changing public APIs or behavior in a crate, update that crate's README and
crate-level or module rustdocs when they describe the affected contract.

## Workspace Map

### Main User-Facing Entry Points

- `crates/gpui-table`
  Audience: **User-facing**
  Role: workspace facade, default entry point, and home of the public feature gates. Re-exports the core and runtime namespaces and, with `derive`, the proc macros.

- `crates/gpui-table-component`
  Audience: **User-facing**
  Role: built-in GPUI filter widgets and `TableStatusBar` for teams that need direct UI composition outside fully derive-generated flows. Most users should still start with `gpui-table`.

### Public Integration Crates

- `crates/gpui-table-core`
  Audience: **Public integration**
  Role: pure filter semantics, typed filter values, and feature-gated conversion helpers. Most application users should start with `gpui-table` instead.

- `crates/gpui-table-runtime`
  Audience: **Public integration**
  Role: GPUI-facing row traits, default cell rendering, load-more support, and the stable generated-filter runtime facade. Most application users should start with `gpui-table` instead.

- `crates/gpui-table-schema`
  Audience: **Public integration**
  Role: UI-neutral filter metadata and inventory-backed table-shape registry types used by tooling and generated flows. Most application users should start with `gpui-table` instead.

- `crates/gpui-table-derive`
  Audience: **Public integration**
  Role: proc-macro crate for `GpuiTable`, `Filterable`, `TableCell`, and `gpui_table_impl`. Most users should depend on `gpui-table` instead of this crate directly.

- `crates/gpui-table-mcp`
  Audience: **Public integration**
  Role: experimental MCP query registry, `rmcp` stdio serving, and JSON filter decoding for generated table filter values. Most application users should enable it through `gpui-table/mcp` only when exposing tables to MCP clients.

- `crates/gpui-table-prototyping-core`
  Audience: **Public integration**
  Role: code-generation helpers that consume `GpuiTableShape` inventory metadata to generate GPUI table stories and scaffolding. Most application users should add this only when building tooling or prototypes.

### Internal Crates and Generated Surfaces

- `justfile`
  Audience: **Internal**
  Role: workspace maintenance commands for formatting, linting, tests, and dry-run publishing.

- `examples/prototyping/output`
  Audience: **Internal**
  Role: generated story modules emitted by `examples/prototyping`. Regenerate with `cargo run -p prototyping` instead of editing by hand.

- `crates/gpui-table/tests/ui`
  Audience: **Internal**
  Role: compile-fail coverage for derive diagnostics and feature-gated macro behavior.

- `crates/gpui-table/wip`
  Audience: **Internal**
  Role: scratch stderr artifacts for macro work. Treat this as maintenance data, not user-facing documentation.

### Examples and Supporting Surfaces

- `examples/README.md`
  Canonical index of runnable examples. Keep this aligned with the root `README.md`.

- `examples/some-lib`
  Shared domain types, derived tables, filterable enums, and package-local
  Fluent translation assets used by the example app and prototyping generator.

- `examples/some-lib-tables`
  Storybook-style GPUI app for exercising generated tables and filters. `cargo run` from the workspace root lands here by default.

- `crates/gpui-table-component` storybook
  Built-in filter and status-bar preview app. Run with `cargo run -p gpui-table-component --bin story --features story`.

- `examples/mcp-query`
  Stdio MCP proof-of-concept that exposes generated table filters as query tool arguments and returns matching in-memory rows.

- `examples/prototyping`
  Inventory-driven generator that writes story modules into `examples/prototyping/output`.

## Validation and Editing Rules

### Validation After Changes

- Validation is the default after code or workflow changes.
- Run the narrowest command that proves the edited behavior works for the
  affected crate, docs, example, or generated surface.
- Prefer targeted crate, example, docs, generated output, or UI checks before full-workspace validation.
- Use `just check`, `just test`, or a more specific `justfile` recipe when the change spans multiple surfaces.
- If validation cannot be run, state why and what remains unvalidated.
- Do not claim a change works unless it was validated, generated from a source of truth, or the remaining risk is explicitly documented.

### When Editing Docs

- Keep READMEs user-facing.
- Keep macro expansion details, generated-code internals, and subsystem design
  in rustdocs, focused tests, compile-fail fixtures, snapshots, or this guide
  when the detail changes contributor routing.
- Prefer examples over prose-only explanations.
- Sync the root `README.md`, affected crate `README.md` files,
  `examples/README.md`, and relevant `skills/*` guidance when public behavior
  changes.
- If the change affects runnable flows, update the relevant example crates too.

### When Editing Rust Crates

- Use `cargo` for build, test, and run tasks.
- Keep dependency versions in the workspace root `Cargo.toml`.
- Use `workspace = true` in member crates.
- Prefer workspace dependencies and shared feature wiring from the root `Cargo.toml`.
- Non-example crates should reference workspace crates with `workspace = true`, not explicit paths.

### When Editing Derives, Filters, or Registry Metadata

- Keep `gpui-table`, `gpui-table-derive`, `gpui-table-runtime`, and `gpui-table-schema` aligned when generated code contracts change.
- If `GpuiTableShape`, `ColumnVariant`, `FilterVariant`, or inventory registration changes, update the relevant README files and rustdocs, then regenerate `examples/prototyping/output`.
- If built-in filter behavior or query-value behavior changes, update `crates/gpui-table-component` stories and any affected example tables in `examples/some-lib-tables`.
- If `fluent` behavior changes, update the relevant `i18n/` assets in crates and examples.

### When Writing Tests

- Prefer `trybuild` for proc-macro compile errors and keep `.stderr` fixtures aligned.
- Prefer [insta](https://insta.rs/) snapshots when verifying structured table-rendering output is clearer than assertion-heavy unit tests.
- Use raw multiline strings or `quote! { ... }` for embedded Rust code in macro tests instead of heavily escaped single-line literals.
