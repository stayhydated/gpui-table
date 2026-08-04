# Columns and rows

Each non-skipped field becomes a column in source order. Attributes control the
initial column configuration, while the generated delegate owns sorting and
maps visible rows back to the source `rows` vector.

```rust,ignore
#[derive(Clone, gpui_table::GpuiTable)]
#[gpui_table(
    id = "build-history",
    context_menu_route = "/builds/{id}",
    context_menu_label = "Open build"
)]
struct BuildRow {
    #[gpui_table(skip, context_menu_id)]
    internal_id: u64,

    #[gpui_table(width = 220., sortable)]
    package: String,

    #[gpui_table(width = 100., resizable = false)]
    status: String,

    #[gpui_table(width = 140., movable = false, sortable, ascending)]
    started_at: chrono::DateTime<chrono::Utc>,
}
```

Common field options are:

| Option | Effect |
|---|---|
| `skip` | Excludes the field from generated columns |
| `title = "Label"` | Overrides the displayed column label |
| `col = "stable-key"` | Overrides the column key |
| `width = 160.` | Sets the initial width; the default is `100.` |
| `sortable` | Enables delegate sorting for the field |
| `ascending` or `descending` | Selects the initial sort direction |
| `fixed = "left"` or `"right"` | Pins the column to that side |
| `text_right` | Right-aligns cell text |
| `resizable = false` or `movable = false` | Disables that interaction |
| `style = path::to_fn` | Replaces the default renderer for this field |

`ascending` and `descending` cannot be combined. Use `sortable` with either
initial direction so the generated delegate also handles later sort changes.

## Render application value objects

`#[derive(TableCell)]` delegates a single-field wrapper to the inner field's
existing renderer:

```rust,ignore
#[derive(Clone, gpui_table::TableCell)]
struct Revision(String);
```

Use `#[table_cell(display)]` only when the wrapper itself implements
`std::fmt::Display`. Use `#[table_cell(format = path::to::formatter)]` when a
function taking `&Revision` should produce its label. For a complete custom
GPUI element, use the `style` hook described in
[Loading and custom cells](loading.md#render-a-custom-cell).

## Keep row visibility current

The generated delegate keeps source rows in `delegate.rows`. Its visible row
view composes generated filters with an optional application-owned row scope.
Call `refresh_filtered_rows()` after in-place row mutations that may change
visibility.

The table ID defaults to the row type in snake case. Set
`#[gpui_table(id = "build-history")]` when external tools or saved state need a
stable value. Explicit IDs must be nonempty and contain only lowercase ASCII
letters, digits, `_`, or `-`.

## Add a row context-menu link

The example marks `internal_id` as the context-menu value and substitutes it
for `{id}` in the route. `context_menu_route` requires either one field marked
`context_menu_id` or the equivalent
`context_menu_row_id = "internal_id"` struct option. Use
`context_menu_route_fn` and `context_menu_label_fn` when routing or
localization is owned by helper functions. Add `custom_context_menu` and
implement `TableRowContextMenu` only when generated and application-owned menu
items must be composed.
