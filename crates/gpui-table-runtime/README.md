# gpui-table-runtime

GPUI-facing runtime traits and helpers for `gpui-table`.

## What it provides

- `TableCell` and built-in default renderers for common scalar/date/time values
- Row/runtime traits: `TableRowMeta`, `TableRowStyle`, `TableRowContextMenu`
- Load-more traits: `TableLoader`, `TableDataLoader`
- Generated-filter runtime facade via `generated_filters`

## Notes

- This crate is the GPUI runtime layer for the workspace.
- Static metadata lives in `gpui-table-schema`.
- Filter matching and typed filter values live in `gpui-table-core`.
