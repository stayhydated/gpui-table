# gpui-table custom shape patterns

Use the narrowest section that matches the requested integration.

## Supported built-in fields

- `TextFilter`: `String` and
  `Option<String>`
- `FacetedFilter<T>`: `T`,
  `Option<T>`, `Vec<T>`, and
  `Option<Vec<T>>` where `T: Filterable`
- `NumberRangeFilter`: supported numeric types and optional variants
  with `rust_decimal`
- `DateRangeFilter`: supported date-like types and optional variants
  with `chrono`
- SpacetimeDB temporal support: matching facade and component
  `spacetimedb` features

Configured field expressions use a builder for the same base shape. Built-ins
include text validation, numeric `range(...).step(...)`, and faceted
`searchable(true)`.

## Adapt a domain value type

Implement the field conversion trait and select the matching adapter:

```rust
#[derive(Clone, gpui_table::TableCell)]
struct AccountCode(String);

impl gpui_table_component::TextFilterField for AccountCode {
    fn to_filter_text(&self) -> String {
        self.0.clone()
    }
}

#[derive(Clone, gpui_table::GpuiTable)]
#[gpui_table(filters)]
struct Account {
    #[gpui_table(filter(gpui_table_component::TextFilterAdapter))]
    code: AccountCode,
}
```

The adapter supports both `T` and `Option<T>`. Number and
date adapters use `NumberRangeFilterField` and
`DateRangeFilterField`.

## Derive an adapter over a base shape

```rust
#[derive(Clone, Debug, Default, PartialEq)]
struct PrefixText(String);

#[derive(gpui_table::GpuiTableFilterShape)]
#[gpui_table_filter_shape(
    base = gpui_table_component::TextFilter,
    raw_value = PrefixText,
    field = String,
    into_base = |value: PrefixText| value.0,
    from_base = PrefixText
)]
struct PrefixTextFilter;
```

The derive delegates the component, typed matching value, reset behavior, and
preset application to the base shape while converting its raw value. Use
`fields(A, B, ...)` for several supported field types. Omit
`raw_value` and conversions when identity conversion is correct.

With `gpui-table/mcp`, the derive emits the default MCP shape and
validation bridge when its raw value implements `McpToolValue`.

Use `koruma_newtype` when `field` is a Koruma newtype and
the base shape supports its `NewtypeValue::Inner`. The generated
matching and MCP validation use the inner value.

## Implement a new runtime shape

Implement these contracts:

1. `ComponentShapeMetadata` and
   `DeclaredComponentShape` for stable component identity.
2. `GpuiTableFilterShape` for the component, raw value, typed filter
   value, semantic filter category, construction, reading, wrapping, silent
   reset, and optional preset application.
3. `DeclaredGpuiTableFilterShape` to opt the shape into table use.
4. `ComponentShapeFor<Field>` and
   `GpuiTableFilterShapeFor<Field>` for each supported field and its
   matching behavior.

`RawValue` must be defaultable, cloneable, sendable, and
`'static`. `FilterValue` must be cloneable, sendable, and
`'static`. Return `true` from matching when the filter is
inactive.

A configured custom expression must produce
`GpuiTableFilterShapeBuilder<Shape>`. Its `build` method
constructs the same component while applying the configuration.

## Support saved presets

Generated filter values call
`gpui_table::FilterPresetValue` for each typed filter value. Implement
that codec for a new value type.

Override `GpuiTableFilterShape::unwrap_value` and
`set_silent` so `FilterEntities::apply_values(...)` can
restore a complete snapshot without firing individual callbacks. The default
implementation rejects non-default application and resets the component.

## Support MCP arguments

Choose one path:

- Derive `gpui_table::McpFilterShape` when the raw value implements
  `McpToolValue`.
- Implement `McpFilterShape` directly when the input needs a custom
  schema or decoder.
- Use `McpRange<T>` for a typed
  `{ "min": ..., "max": ... }` raw value.
- Use `McpAny` only when unconstrained JSON is intentional.

Add `McpFilterShapeValidation` when field-level
`#[koruma(...)]` validators must run for the shape. Add
`McpKorumaNewtypeFilterValidation<Field>` for a manual Koruma
newtype adapter.

## Preserve registry metadata

Inventory-enabled table derives record the field, Rust field type, semantic
filter category, and resolved base shape path through
`ComponentShapeUse`. Keep component identities and base paths stable
when tooling or generated stories consume that metadata. Configured builders
change component construction, not the registered base shape.
