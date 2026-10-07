# Order rows and calculated keys

Use one `SortOrder` for table controls, saved preferences, and query handlers.
Clauses are evaluated in priority order. Each clause names a stable column key,
its direction, and null placement; null placement is independent of direction.

## Declare sortable columns and row identity

```rust
use gpui_table::GpuiTable;

#[derive(Clone, GpuiTable)]
#[gpui_table(row_id = "id")]
struct Record {
    #[gpui_table(skip)]
    id: u64,
    #[gpui_table(sortable, col = "category_key")]
    category: String,
    #[gpui_table(sortable)]
    score: Option<i64>,
}
```

`row_id` names a unique, stable field. It supplies ascending tie-breaking and
lets selection follow records across sorting and refreshes. The generated
column enum's `key()` returns the public key, including a `col` override.
Registry metadata keeps `field_name` for Rust access and `key` for ordering.

When `row_id` is omitted, ties retain source order and the delegate uses source
positions for selection identity. Use explicit IDs when replacing, inserting,
or removing source rows. `ascending` and `descending` declare initial clauses
in field order and also enable sorting for those columns.

## Apply ordered clauses

```rust
use gpui_table::sort::{NullPlacement, SortClause, SortDirection, SortOrder};

let ordering = SortOrder::new(vec![
    SortClause::new("category_key", SortDirection::Ascending),
    SortClause::new("score", SortDirection::Descending)
        .with_nulls(NullPlacement::Last),
])?;
# Ok::<(), gpui_table::sort::SortError>(())
```

Call `gpui_table::runtime::set_table_ordering(table, ordering, cx)` inside a
`TableState` update to retain row or cell selection. A standalone delegate
supports `set_ordering(ordering)` and `ordering()`.

The delegate orders its **loaded rows** and keeps `rows` in source order.
`visible_row_indices()` is the ordered, filtered source-index view. For remote
pagination, send the same order to the backend and execute it before selecting
pages. Sorting a partial loaded buffer does not order the complete result.

`SortOrder` serializes as an array of clauses. Deserialization rejects duplicate
keys, unknown clause fields, invalid directions/null policies and more than 32
clauses. Row validation also rejects unknown or unsortable column keys, even
for an empty result. Comparison errors leave a requested order unapplied.

## Share calculated keys with cell presentation

Declare `#[gpui_table(sortable, sort_key = score_key, style = score_cell)]`.
The key function accepts `&Record` and returns either `Option<Key>` or
`Result<Option<Key>, SortError>`, where `Key: PartialOrd`. Use that same function
from the cell renderer so the displayed calculation and its ordering agree.
Keep display formatting separate from the comparable typed value.

Null results use the clause's null policy. Failed calculations and unordered
values such as NaN produce `SortError`; they are not silently converted to ties.
The row owner defines expression vocabulary, units, evaluation limits and
backend translation. The table API accepts typed keys without imposing a
formula language or report engine.

After an in-place row change, call `refresh_filtered_rows()` to recompute the
ordered view. Check `sort_error()` if changed inputs can fail. A refresh failure
exposes the error and the filtered source order until the owner repairs the
input or changes the ordering. Capture `TableRowSelection::capture(table)`
before replacing or editing source rows, refresh, then call
`selection.restore(table, cx)` to retain selection by ID; removed records clear
selection.

The `ordered_sort` example uses real SpacetimeDB `Timestamp` and `TimeDuration`
values, a calculated key shared with a cell renderer, and an MCP query. Enable
`spacetimedb`, `mcp`, and `rust_decimal` to run it. Duration cells use the SDK's
`Display` representation, while comparisons use the typed duration.

## Compose sort controls

`gpui-table-component::SortEditor` takes a caller-owned `SortOrder`,
`SortColumnOption` values with localized titles, and an `on_change` callback.
It provides key selection, direction/null policy, priority changes and removal.
The owner applies each requested value and notifies its view. Programmatic
controlled updates do not emit a user-change callback.

Use a stable editor ID. Clause controls use stable column keys, so changing
priority retains their identity. `disabled(true)` prevents editing. Place the
component in the existing toolbar, panel or sheet; it owns no query, dialog,
persistence or authorization. Initialize the component's i18n as for filters.

## Declare source capabilities

Call `delegate.set_allowed_sort_columns(["name", "created_at"])` before
presenting a generated delegate when the source supports a subset of its sortable
keys. Excluded headers become unsortable and direct order changes reject excluded
keys. An empty list disables ordering. Unknown keys, duplicate keys, and a
restriction excluding the current order fail atomically.

## Resolve application-owned columns

Use `sort::ResolvedSortColumn::new(key, extract)` for a source-resolved key.
The extractor returns `Result<Option<Key>, SortError>` with `Key: PartialOrd`.
Keep the same extractor available to the owning cell renderer. The application
resolves the definition, source readiness, units and permissions before supplying
this context.

```rust
use gpui_table::sort::{ResolvedSortColumn, ResolvedSortColumns, SortError};

fn scaled_score(row: &Record) -> Result<Option<i64>, SortError> {
    row.score.map(|score| score.checked_mul(2).ok_or_else(|| SortError::Calculation {
        column: "scaled_score".into(),
        message: "overflow".into(),
    })).transpose()
}

let columns = ResolvedSortColumns::new([
    ResolvedSortColumn::new("scaled_score", scaled_score)?,
])?;
let mut delegate = RecordTableDelegate::new(Vec::new());
delegate.set_resolved_sort_columns(columns)?;
delegate.set_allowed_sort_columns(["category_key", "scaled_score"])?;
# Ok::<(), SortError>(())
```

Resolved keys are distinct from native keys and from each other. The retained
context validates empty sources, preflights single-row errors and compares only
included rows. Native and resolved clauses share null placement, direction,
identity tie-breaking and atomic error handling. `ResolvedSortColumns` also
provides complete-source `sorted_indices`, `sort_indices` and `sort_rows` for
application query owners.

Use `runtime::set_table_resolved_sort_columns(table, columns, cx)` for a generated
table state. It preserves selection by stable row identity. Context replacement
checks the current order, source capability restriction and visible values before
applying the change. Clear an order and its capability restriction before retiring
keys that it uses. Remote query owners execute the same resolved calculation over
the complete source before pagination.
