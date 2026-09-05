# Getting started

This path produces a `DataTable` backed by typed application rows and generated
filter controls.

## Prerequisites

Initialize `gpui-kit` during application startup. If the table uses the
built-in filters from `gpui-table-component`, initialize their localization
bridge after it:

```rust,ignore
gpui_kit::component::init(cx);
gpui_table_component::i18n::init(cx)?;
```

Your crate must directly depend on `gpui`, `gpui-kit`, `gpui-table`, and
`gpui-table-component`. See [Features and integration crates](features.md) when
a row uses numeric range filters, Fluent labels, SpacetimeDB values, or MCP.

## Define the rows

Derive `GpuiTable` on a named struct. Every visible field type must implement
`TableCell`; built-in scalar and date types already do. This faceted enum
derives both `Filterable` for its filter options and `TableCell` for rendering:

```rust,ignore
use gpui_table::{Filterable, GpuiTable, TableCell};

#[derive(Clone, Eq, Filterable, Hash, PartialEq, TableCell)]
enum UserStatus {
    Active,
    Suspended,
}

#[derive(Clone, GpuiTable)]
#[gpui_table(filters)]
struct User {
    #[gpui_table(
        sortable,
        width = 160.,
        filter(gpui_table_component::TextFilter)
    )]
    name: String,

    #[gpui_table(
        width = 120.,
        filter(gpui_table_component::FacetedFilter::<UserStatus>.searchable(true))
    )]
    status: UserStatus,
}
```

## Create and render the table

Construct the generated delegate from a `Vec<User>`, put it in `TableState`,
and build the generated filters against that state. Store the table entity and
filter collection on the owning view:

```rust,ignore
use gpui_kit::component::table::{DataTable, TableState};

let delegate = UserTableDelegate::new(rows);
let table = cx.new(|cx| TableState::new(delegate, window, cx));
let filters = UserFilterEntities::build_for_table(table.clone(), cx);

let element = DataTable::new(&table)
    .stripe(true)
    .scrollbar_visible(true, true);
```

Keep the `TableState` entity on the owning view. Observe it when delegate
updates should rerender parent controls such as status bars or filter badges.
Render the entries returned by `filters.filter_sidebar_data(cx)` to make the
filter widgets visible; [Typed filters](filters.md) shows that flow.

## Verify the result

The table should display one row for each item passed to
`UserTableDelegate::new`. Clicking the `name` header should change the sort
order, and changing a rendered filter should narrow the visible rows without
replacing `delegate.rows`.

If the derive reports that a field does not implement `TableCell`, derive it on
an application value object or use a custom cell renderer. If rendering panics
because localization state is missing, run both initialization calls above
before creating or rendering the filters.
