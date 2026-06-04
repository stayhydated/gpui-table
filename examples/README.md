# Examples

`examples/some-lib` and `examples/some-lib-tables` are the canonical end-to-end
examples for the workspace. They show the derive flow that most application
code should start with.

## Recommended Runs

### Launch the storybook app

```sh
cargo run
```

From the workspace root, this starts `examples/some-lib-tables`, which is also
the default workspace member.

Use it to see:

- derived columns rendered in `gpui_component::table::DataTable`
- generated built-in filters wired with `UserFilterEntities::build_for_table(...)`
- `TableStatusBar` and filter layout composition
- typed `es-fluent` titles, descriptions, and faceted labels
- custom row rendering and load-more behavior
- optional row-context-menu routing when `some-lib-tables` is run with `--features router`

### Regenerate prototyping output

```sh
cargo run -p prototyping
```

This iterates the inventory-registered `GpuiTableShape` values, including
`ComponentShapeUse` filter metadata, and rewrites
`examples/prototyping/output`. Generated Storybook table titles use the active
example app locale.

Do not hand-edit `examples/prototyping/output`; it is generated output.

## Workspace Layout

- `examples/i18n`
  Shared Fluent resources used by the example crates.
- `examples/some-lib`
  Domain types, `#[derive(GpuiTable)]` rows, `#[derive(Filterable)]` enums, and embedded i18n setup.
- `examples/some-lib-tables`
  Storybook-style GPUI app that renders the generated tables and filters.
- `examples/prototyping`
  Inventory-driven generator that writes story modules into `examples/prototyping/output`.

## Fluent Setup

The examples use `es-fluent` typed messages directly: row structs derive
`EsFluentLabel`/`EsFluentVariants`, faceted enums derive `EsFluent`, and table
attributes opt into localized labels.

```rs
#[derive(Clone, Eq, Hash, PartialEq, es_fluent::EsFluent, gpui_table::Filterable)]
#[filter(fluent)]
pub enum UserStatus {
    Active,
    Suspended,
}
```

`examples/some-lib/src/i18n.rs` declares the embedded resources, while
`examples/some-lib-tables/src/i18n.rs` declares the app language enum with
`#[es_fluent_language]`. The binary imports that language enum and selects the
storybook locale before rendering.

## Files To Read First

- `examples/some-lib/src/structs/user.rs`
  Generated filters, localized titles, faceted enums, and custom context-menu composition.
- `examples/some-lib/src/structs/item.rs`
  Load-more wiring via `#[gpui_table_impl]` plus custom `TableRowStyle`.
- `examples/some-lib-tables/src/tables/user.rs`
  How generated filters are composed into a screen with `DataTable`.
- `examples/prototyping/src/main.rs`
  A complete generator built on `TableShapeAdapter`, `TableLayout`, and `TableParts`.

## Notes

- If you change derive behavior, keep these examples aligned with the public README surfaces in the same change.
- If inventory or codegen behavior changes, rerun `cargo run -p prototyping` so `examples/prototyping/output` stays in sync.
