# gpui-table-component

`gpui-table-component` provides the built-in GPUI table filters,
adapter shapes, reset control, and `TableStatusBar`.

Use the component types in generated fields:

```rust
#[gpui_table(filter(gpui_table_component::TextFilter))]
name: String,

#[gpui_table(
    filter(gpui_table_component::FacetedFilter::<Status>.searchable(true))
)]
status: Status,
```

Use the same components directly when the application owns the filter layout.
The default features enable date and numeric range filters; enable `mcp`
when those shapes are used by MCP query tables.

- [Typed filters](https://stayhydated.github.io/gpui-table/book/filters.html)
- [Components and custom shapes](https://stayhydated.github.io/gpui-table/book/custom_filters.html)
- [API documentation](https://docs.rs/gpui-table-component/)
