# Loading and custom cells

`#[gpui_table(load_more)]` lets `DataTable` request more rows as the viewport
approaches the end. The application implements the fetch and must keep the
generated delegate's `loading` and `eof` flags accurate.

## Implement load-more behavior

```rust,ignore
use gpui::{Context, Window};
use gpui_component::table::TableState;
use gpui_table::runtime::TableLoader;

#[derive(Clone, gpui_table::GpuiTable)]
#[gpui_table(load_more)]
struct EventRow {
    message: String,
}

#[gpui_table::gpui_table_impl]
impl TableLoader for EventRowTableDelegate {
    const THRESHOLD: usize = 20;

    fn load_more(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) {
        if self.loading || self.eof {
            return;
        }

        self.loading = true;
        // Start the application request. When it completes, append rows,
        // clear loading, set eof if no more pages exist, and notify again.
        cx.notify();
    }
}
```

`THRESHOLD` defaults to `10`. The guard prevents duplicate requests while a
load is active and stops requests after the backend reaches the end.

Trigger the initial request after constructing `TableState`:

```rust,ignore
table.update(cx, |table, cx| {
    use gpui_table::runtime::TableDataLoader as _;
    table.delegate_mut().load_data(window, cx);
});
```

When generated filters should reload backend data, build them with
`<Row>FilterEntities::build_for_table_loader(...)` instead. That method also
performs the initial load and resets `rows` and `eof` before a filter-driven
reload.

`TableStatusBar::new(delegate.rows.len(), delegate.loading, delegate.eof)`
provides a matching row-count and loading indicator.

## Render a custom cell

Use field-level `style = path::to_fn` when one column needs a custom GPUI
element:

```rust,ignore
#[gpui_table(width = 120., style = render_duration)]
duration_ms: u64,

fn render_duration(
    _row: &EventRow,
    value: &u64,
    _window: &mut gpui::Window,
    _cx: &mut gpui::App,
) -> impl gpui::IntoElement {
    use gpui::{ParentElement as _, Styled as _};
    gpui::div().px_2().child(format!("{value} ms"))
}
```

The style function receives the full row, the field value, and the active GPUI
contexts. Keep renderer-specific presentation in this hook while the generated
delegate continues to own sorting, filtering, and row access.

If loading repeatedly fires without adding rows, confirm that `loading` is set
before starting asynchronous work and remains true until the completion update.
If it never fires again after a request, confirm that completion clears
`loading`; set `eof` only when no later page exists.
