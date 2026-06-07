# AGENTS.md

This is the working guide for contributors and coding agents in the
`gpui-table` workspace.

Use it to decide:

1. where documentation belongs,
2. whether a crate or surface is user-facing, public integration, or internal,
3. which related docs, examples, and skills must change together,
4. which validation command should run before handoff.

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
2. **Place documentation by content, not by crate audience.** README files are
   always user-facing. Internal design belongs in the matching
   `docs/ARCHITECTURE.md`.
3. **Sync public workflow changes.** If derive syntax, generated filter
   behavior, registry metadata shape, feature flags, generated output,
   localization, or recommended usage changes, update the relevant README,
   example, generated surface, architecture note, and public `skills/*`
   guidance in the same change when applicable.
4. **Validate narrowly.** Run the smallest command that proves the edited
   behavior or documentation surface is still sound.

## Audience Labels

These labels describe the crate or surface itself, not the documentation file
being edited:

- **User-facing**: normal entry points for application developers.
- **Public integration**: public crates meant for extensions, tooling, or deeper customization. These are usually not the default starting point.
- **Internal**: workspace plumbing, generated outputs, examples-as-infrastructure, and maintenance tooling.

## Documentation Placement

### User-Facing Documentation

Treat these surfaces as user-facing:

- the root `README.md`,
- `examples/README.md`,
- crate-level `README.md` files.

Even README files for public-integration crates should explain:

- who the crate is for,
- what it does,
- what most users should use instead.

Keep user-facing documentation example-first. Prefer Rust snippets over
prose-only explanations when showing derive, filter, loader, or registry
behavior.

### Internal Documentation

Use the relevant `docs/ARCHITECTURE.md` file for internal documentation, such
as the crate-level paths listed in the workspace map.

Keep these topics in architecture documents, not in READMEs:

- implementation details,
- macro expansion and parsing details,
- subsystem boundaries,
- generated-code and runtime contracts,
- data flow,
- design rationale,
- inter-crate relationships.

### Skill Guidance

Skill directories are split by audience and intended distribution:

- `.agents/skills/*-dev` contains repo-scoped development skills for Codex and
  local agent use inside this repository. These skills may include internal
  wording, repo-specific assumptions, implementation details, maintainer
  workflows, and development-only instructions. Each directory and its
  `SKILL.md` `name` field must use the same `-dev` suffix.
- `skills/*` contains public, user-facing skills intended to be reusable outside
  this repository or distributed as part of a skills catalog/plugin. These
  skills must not include internal wording, maintainer-only language,
  repo-private assumptions, or implementation details that belong only in a
  corresponding `*-dev` skill.

Do not assume root-level `skills/*` are auto-loaded as repo-local Codex skills.
Use `.agents/skills` for auto-discovered repo-local skills; use `skills/*` as
the source location for public/reusable skills.

`skills/use-gpui-table` is the public `gpui-table` usage skill for application
developers.

Update relevant public `skills/*` guidance when a code change alters
user-facing workflows, derive syntax, generated filter behavior, registry
metadata, generated output, localization patterns, feature flags, or recommended
usage.

## Synchronization Rules

When a substantive change modifies a public workflow, feature-flag story,
derive syntax, generated filter behavior, registry metadata shape, or other
user-visible API shape:

1. Update the root `README.md`.
2. Update the affected crate `README.md` files.
3. Update `examples/README.md` and the relevant example crates when behavior is demonstrated there.
4. Update relevant public `skills/*` guidance.
5. Keep these surfaces aligned in the same change unless there is a documented reason not to.

`examples/some-lib` and `examples/some-lib-tables` are the canonical
end-to-end usage examples.

If inventory or codegen behavior changes, update `examples/prototyping` and
regenerate `examples/prototyping/output`.

`examples/prototyping/output` is a generated surface; regenerate it instead of
hand-editing.

When changing public APIs or behavior in a crate, update that crate's
`docs/ARCHITECTURE.md`.

## Workspace Map

### Main User-Facing Entry Points

- `crates/gpui-table`
  Audience: **User-facing**
  Docs: [Architecture](crates/gpui-table/docs/ARCHITECTURE.md)
  Role: workspace facade, default entry point, and home of the public feature gates. Re-exports the core and runtime namespaces and, with `derive`, the proc macros.

- `crates/gpui-table-component`
  Audience: **User-facing**
  Docs: [Architecture](crates/gpui-table-component/docs/ARCHITECTURE.md)
  Role: built-in GPUI filter widgets and `TableStatusBar` for teams that need direct UI composition outside fully derive-generated flows. Most users should still start with `gpui-table`.

### Public Integration Crates

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
- Move macro expansion details, generated-code internals, and subsystem design into `docs/ARCHITECTURE.md`.
- Prefer examples over prose-only explanations.
- Sync the root `README.md`, affected crate `README.md` files, `examples/README.md`, and public `skills/*` guidance in the same change when public behavior changes.
- If the change affects runnable flows, update the relevant example crates too.

### When Editing Rust Crates

- Use `cargo` for build, test, and run tasks.
- Keep dependency versions in the workspace root `Cargo.toml`.
- Use `workspace = true` in member crates.
- Prefer workspace dependencies and shared feature wiring from the root `Cargo.toml`.
- Non-example crates should reference workspace crates with `workspace = true`, not explicit paths.

### When Editing Derives, Filters, or Registry Metadata

- Keep `gpui-table`, `gpui-table-derive`, `gpui-table-runtime`, and `gpui-table-schema` aligned when generated code contracts change.
- If `GpuiTableShape`, `ColumnVariant`, `FilterVariant`, or inventory registration changes, update the relevant README files, architecture docs, and regenerate `examples/prototyping/output`.
- If built-in filter behavior or query-value behavior changes, update `crates/gpui-table-component` stories and any affected example tables in `examples/some-lib-tables`.
- If `fluent` behavior changes, update the relevant `i18n/` assets in crates and examples.

### When Writing Tests

- Prefer `trybuild` for proc-macro compile errors and keep `.stderr` fixtures aligned.
- Prefer [insta](https://insta.rs/) snapshots when verifying structured table-rendering output is clearer than assertion-heavy unit tests.
- Use raw multiline strings or `quote! { ... }` for embedded Rust code in macro tests instead of heavily escaped single-line literals.
