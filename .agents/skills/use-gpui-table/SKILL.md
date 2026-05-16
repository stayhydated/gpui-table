---
name: use-gpui-table
description: >-
  Build or extend user-facing Rust GPUI application tables with gpui-table. Use
  when application code needs #[derive(GpuiTable)], #[derive(Filterable)],
  #[derive(TableCell)], #[gpui_table_impl], generated table delegates, columns,
  filter entities, and filter values, built-in filters such as text, faceted,
  number_range, or date_range, TableStatusBar, load-more behavior, custom row
  rendering, row context menus, localized labels with fluent, or feature flags
  such as rust_decimal, chrono, or spacetimedb.
---

# Use GPUI Table

## Scope Boundary

Treat this skill as a hosted public-usage guide for `gpui-table` consumers. Use
it only for user-facing application workflows: deriving typed tables, generated
delegates and filters, built-in filter widgets, `TableStatusBar`, load-more
behavior, custom row rendering, row context menus, localization, and feature
selection.

Do not use this skill as a contributor guide for `gpui-table` repository
internals. For build, test, format, lint, maintenance, release, generated-output
regeneration, or architecture work, read the repository source, `AGENTS.md`, and
the relevant crate documentation directly.

## Core Workflow

Start from the user-facing facade. Most application code uses `gpui-table` for
derives, generated types, and runtime helpers:

1. Enable the smallest feature set needed. `derive` and `chrono` are default
   features, `rust_decimal` supports `filter(number_range(...))`, `fluent`
   supports localized labels through `es-fluent`, and `spacetimedb` supports
   SpacetimeDB temporal range filtering.
2. Define row structs with `#[derive(Clone, GpuiTable)]`.
3. Add field-level `#[gpui_table(...)]` attributes for widths, sorting,
   movement, resizing, filters, skipped fields, context menu ids, or generated
   context menu behavior.
4. Use `#[derive(Filterable)]` for faceted enums. Include
   `Clone + Eq + Hash + PartialEq`; add `#[filter(fluent)]` only when labels
   come from `es-fluent`.
5. Use `#[derive(TableCell)]` for value objects. Add `#[table_cell(display)]`
   when the wrapper should render through its own `Display` implementation, or
   `#[table_cell(format = path::to::formatter)]` for a dedicated formatter.
6. Add `#[gpui_table(filters)]` when generated filter entities and typed filter
   state are needed.
7. Add `#[gpui_table(load_more)]` plus `#[gpui_table::gpui_table_impl] impl
   TableLoader for <Row>TableDelegate` for infinite-loading tables.
8. Add `#[gpui_table(custom_style)]` and implement `TableRowStyle` when a column
   needs custom rendering. Delegate unchanged columns to
   `gpui_table::runtime::default_render_cell`.
9. Compose generated tables with `gpui_component::table::DataTable` and
   generated filter helpers, or use `gpui-table-component` directly when manual
   filter UI composition is a better fit.

## Reference Selection

Load only the reference needed for the task:

- `references/patterns.md`: table derives, filters, load-more behavior, row rendering, row context menus, localization, feature flags, and direct filter widgets.

Prefer current public docs or source examples over memory when details matter.

## Implementation Rules

Use `gpui-table` for normal strongly typed GPUI tables. It re-exports the core
and runtime namespaces and, with the default `derive` feature, the proc macros.

Use `gpui-table-component` when the app needs direct GPUI filter widget
composition, `ResetFilters`, `TableStatusBar`, or `QueryFilterValue`.

Generated names follow the row type:

- `<Row>TableDelegate`
- `<Row>TableColumn`
- `<Row>FilterEntities`
- `<Row>FilterValues`

Use built-in filters through field attributes when they match the application
workflow:

- `filter(text)` for text search.
- `filter(faceted)` for enum-like values derived with `Filterable`.
- `filter(number_range(...))` for numeric ranges.
- `filter(date_range(...))` for temporal ranges.

Keep localized labels explicit. Use `#[filter(fluent)]` or the matching table
label attributes only when the application owns an `es-fluent` localizer and the
labels are rendered through that context.
