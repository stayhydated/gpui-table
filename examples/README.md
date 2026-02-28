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

That story now queries SpaceTimeDB over `POST /v1/database/sql/{database}`.
Set `GPUI_TABLE_SPACETIMEDB_DATABASE` before running.

Optional environment variables:
- `GPUI_TABLE_SPACETIMEDB_URI` (default: `http://localhost:3000`)
- `GPUI_TABLE_SPACETIMEDB_SQL` (default: query against `spacetime_event`)
- `GPUI_TABLE_SPACETIMEDB_TOKEN` (Bearer token)
- `GPUI_TABLE_SPACETIMEDB_PAGE_SIZE` (default: `50`)

`GPUI_TABLE_SPACETIMEDB_SQL` can include `{limit}` and `{offset}` placeholders.
If omitted, the loader appends `LIMIT/OFFSET`.
The query should project: `table_name`, `sender`, `connection_id`, `mutation`,
`committed_at`, and `reducer`.

## prototyping

our own prototyping tool for generating the tables, defining items that we would otherwise have to write ourselves.
Then styling's pretty much what's left to do.
