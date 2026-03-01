tldr

```sh
cargo run
```

## i18n

i18n resources

## some-lib

our crate types

## some-lib-tables

crate to hold the gpui tables, also has a main.rs defining a storybook-like gpui app, showcasing the tables

Includes a `SpacetimeEvent` table story that demonstrates `spacetimedb` types
(`Timestamp`, `Identity`, `ConnectionId`) in `gpui-table`.

The `SpacetimeEvent` loader is feature-gated and can query via generated
SpaceTimeDB client bindings:

- run `examples/some-lib/setup.sh` (builds and publishes with `--features db`, then generates bindings)
- seed `spacetime_event` rows with `cargo run -p some-lib --features seed-bin --bin seed_spacetime_events -- 10000`
- run the story app with `cargo run -p some-lib-tables`
- `some-lib-tables` auto-initializes `DbConnection` via `some_lib::client_connection::init_from_env()`
  (`SPACETIMEDB_URI` defaults to `http://127.0.0.1:3000`, `SPACETIMEDB_DB_NAME` defaults to `gpui-table-some-lib`)

`module_bindings` are generated into `examples/some-lib/src/module_bindings/`.

## prototyping

our own prototyping tool for generating the tables, defining items that we would otherwise have to write ourselves.
Then styling's pretty much what's left to do.
