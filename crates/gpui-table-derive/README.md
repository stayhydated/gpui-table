# gpui-table-derive

`gpui-table-derive` provides the proc macros behind
`gpui-table`:

- `#[derive(GpuiTable)]`
- `#[derive(Filterable)]`
- `#[derive(TableCell)]`
- `#[derive(GpuiTableFilterShape)]`
- `#[derive(McpFilterShape)]` with the `mcp` feature
- `#[gpui_table_impl]` and `#[mcp_query]`

Application crates should depend on `gpui-table` and use its macro
re-exports. Depend on this proc-macro crate directly only when integrating the
macros without the facade.

- [User guide](https://stayhydated.github.io/gpui-table/book/)
- [API documentation](https://docs.rs/gpui-table-derive/)
