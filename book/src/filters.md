# Typed filters

Generated filters keep filter values typed and apply them to the table's
in-memory rows. Enable the facade's `rust_decimal` feature before using any
numeric range filter, including one over an integer field:

```toml
[dependencies]
chrono = "0.4"
gpui-table = { version = "0.6", features = ["rust_decimal"] }
gpui-table-component = { version = "0.6", features = ["rust_decimal"] }
```

Add `#[gpui_table(filters)]` and declare each filtered field's shape:

```rust,ignore
#[derive(Clone, gpui_table::GpuiTable)]
#[gpui_table(filters)]
struct Order {
    #[gpui_table(filter(gpui_table_component::TextFilter))]
    customer: String,

    #[gpui_table(filter(gpui_table_component::NumberRangeFilter))]
    total: u32,

    #[gpui_table(filter(gpui_table_component::DateRangeFilter))]
    placed_at: chrono::DateTime<chrono::Utc>,
}
```

Built-in shapes cover text, faceted, numeric range, and date range filters.
Configured expressions such as `TextFilter.alphanumeric_only()` and
`FacetedFilter::<T>.searchable(true)` customize generated widget construction.
`NumberRangeFilter.range(...).step(...)` accepts
`rust_decimal::Decimal` bounds and steps.

## Wire and render generated filters

Build filter entities against the same table state that renders the
`DataTable`. Each widget change snapshots the generated values, updates the
delegate, and notifies the table:

```rust,ignore
let filters = OrderFilterEntities::build_for_table(table.clone(), cx);

let filter_elements = filters
    .filter_sidebar_data(cx)
    .into_groups()
    .into_iter()
    .flat_map(|group| group.into_items())
    .map(|item| item.into_element())
    .collect::<Vec<_>>();
```

The sidebar data groups nonempty filters in Text, Faceted, Number Range, and
Date Range order. Each item also carries its stable field ID, label, filter
type, and active state for custom application shells.

For loader-backed tables, use
`OrderFilterEntities::build_for_table_loader(table, window, cx)`. It clears
rows, resets `eof`, stores the new filter values, and calls `load_data` on the
initial build and after each filter change.

## Control and save filter state

Generated delegate state can add an application-owned scope to the generated
filter values:

```rust,ignore
delegate.set_filter_values(values);
delegate.clear_filter_values();
delegate.set_row_scope(|row| row.total > 0);
delegate.clear_row_scope();
```

Save and restore a complete widget snapshot through the generated values type:

```rust,ignore
let preset = filters.read_values(cx).to_preset_json();
let values = OrderFilterValues::from_preset_json(&preset)?;
filters.apply_values(values, window, cx);
```

`filters.active_filter_count(cx)` supports badges,
`filters.reset_filters(window, cx)` resets all widgets with one change
notification, and `filters.reset_button()` creates a bound reset control.

If a field-level filter produces a derive error, first confirm that the struct
has `filters`, the shape supports the field type, and the facade feature named
by the diagnostic is enabled.
