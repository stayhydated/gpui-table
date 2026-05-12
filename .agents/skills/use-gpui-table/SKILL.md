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

# Use gpui-table

## Start Here

Use this skill from the perspective of an application developer adding typed tables to a GPUI app.

Prefer the `gpui-table` facade for derives, generated types, and runtime helpers. Reach for `gpui-table-component` only when the application needs direct control over built-in filter widgets or `TableStatusBar`.

Read `references/patterns.md` when implementing table derives, filters, load-more behavior, row rendering, row context menus, localization, or direct filter widgets.

## User-Facing Entry Points

- Use `gpui-table` for normal strongly typed GPUI tables. It re-exports the core and runtime namespaces and, with the default `derive` feature, the proc macros.
- Use `gpui-table-component` when the app needs direct GPUI filter widget composition, `ResetFilters`, `TableStatusBar`, or `QueryFilterValue`.

## Application Workflow

1. Enable the smallest feature set needed:
   - `derive` and `chrono` are default features.
   - Add `rust_decimal` for `filter(number_range(...))`.
   - Add `fluent` for localized table labels and faceted labels through `es-fluent`.
   - Add `spacetimedb` for supported SpacetimeDB temporal range filtering.
2. Define row structs with `#[derive(Clone, GpuiTable)]`.
3. Add field-level `#[gpui_table(...)]` attributes for widths, sorting, movement/resizing, filters, skipped fields, context menu ids, or generated context menu behavior.
4. Use `#[derive(Filterable)]` for faceted enums. Include `Clone + Eq + Hash + PartialEq`; add `#[filter(fluent)]` only when labels come from `es-fluent`.
5. Add `#[gpui_table(filters)]` when generated filter entities and typed filter state are needed. Generated names follow the row type, such as `UserTableDelegate`, `UserTableColumn`, `UserFilterEntities`, and `UserFilterValues`.
6. Add `#[gpui_table(load_more)]` plus `#[gpui_table::gpui_table_impl] impl TableLoader for <Row>TableDelegate` for infinite-loading tables.
7. Add `#[gpui_table(custom_style)]` and implement `TableRowStyle` when a column needs custom rendering; delegate unchanged columns to `gpui_table::runtime::default_render_cell`.
8. Compose generated tables with `gpui_component::table::DataTable` and generated filter helpers, or use `gpui-table-component` directly when manual filter UI composition is a better fit.
