# Examples

```sh
cargo run
```

From the workspace root, `cargo run` launches `examples/some-lib-tables`,
which is configured as the default workspace member.

## Layout

- `examples/i18n`: shared Fluent resources used by the example crates
- `examples/some-lib`: shared domain types, derived tables, and filterable enums
- `examples/some-lib-tables`: storybook-style GPUI app that renders the example tables
- `examples/prototyping`: generator that iterates `GpuiTableShape` inventory
  entries and writes stories to `examples/prototyping/output`

## Useful commands

```sh
cargo run
cargo run -p prototyping
```

Use the second command after changing inventory-registered table shapes or the
prototyping layout.
