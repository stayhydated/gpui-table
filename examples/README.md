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
- custom row rendering and load-more behavior
- optional row-context-menu routing when `some-lib-tables` is run with `--features router`

### Regenerate prototyping output

```sh
cargo run -p prototyping
```

This iterates the inventory-registered `GpuiTableShape` values and rewrites
`examples/prototyping/output`.

Do not hand-edit `examples/prototyping/output`; it is generated output.

## Workspace Layout

- `examples/i18n`
  Shared Fluent resources used by the example crates.
- `examples/some-lib`
  Domain types, `#[derive(GpuiTable)]` rows, `#[derive(Filterable)]` enums, and shared i18n setup.
- `examples/some-lib-tables`
  Storybook-style GPUI app that renders the generated tables and filters.
- `examples/prototyping`
  Inventory-driven generator that writes story modules into `examples/prototyping/output`.

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
