# gpui-table Application Patterns

Load this reference when implementing user-facing application tables, filters, load-more behavior, custom row rendering, row context menus, localization, or direct filter widgets.

## Feature Flags

```toml
[dependencies]
gpui-table = { version = "*", features = ["fluent", "rust_decimal"] }
```

- `derive` and `chrono` are default features.
- `rust_decimal` is required for `filter(number_range(...))`.
- `fluent` localizes table titles and faceted labels with typed `es-fluent` resources.
- `spacetimedb` enables supported temporal range filtering helpers.

## Basic Derived Table

```rust
use gpui::{Context, Window};
use gpui_component::table::TableState;
use gpui_table::runtime::TableLoader;
use gpui_table::{Filterable, GpuiTable};

#[derive(Clone, Eq, Hash, PartialEq, Filterable)]
pub enum UserStatus {
    Active,
    Suspended,
}

#[derive(Clone, GpuiTable)]
#[gpui_table(filters, load_more)]
pub struct User {
    #[gpui_table(sortable, width = 160., filter(text()))]
    pub name: String,

    #[gpui_table(width = 80., filter(number_range(min = 0, max = 120)))]
    pub age: u8,

    #[gpui_table(width = 120., filter(faceted()))]
    pub status: UserStatus,
}

#[gpui_table::gpui_table_impl]
impl TableLoader for UserTableDelegate {
    fn load_more(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        cx.notify();
    }
}
```

With `#[gpui_table(filters)]`, the derive generates:

- `<Row>TableDelegate`
- `<Row>TableColumn`
- `<Row>FilterEntities`
- `<Row>FilterValues`
- `Matchable<<Row>FilterValues>` for strongly typed client-side filtering

## Built-In Filter Syntax

```rust
#[gpui_table(filter(text()))]
name: String,

#[gpui_table(filter(number_range()))]
age: u8,

#[gpui_table(filter(number_range(min = 0, max = 120)))]
score: u8,

#[gpui_table(filter(date_range()))]
created_at: chrono::DateTime<chrono::Utc>,

#[gpui_table(filter(faceted()))]
active: bool,
```

Use `#[derive(Filterable)]` for faceted enums:

```rust
use gpui_component::IconName;
use gpui_table::Filterable;

#[derive(Clone, Eq, Hash, PartialEq, Filterable)]
pub enum Status {
    #[filter(icon = IconName::Check)]
    Active,
    #[filter(label = "Needs Review")]
    Pending,
}
```

## Fluent Labels

```rust
use es_fluent::{EsFluentLabel, EsFluentVariants};
use gpui_table::{Filterable, GpuiTable};

#[derive(Clone, Eq, Hash, PartialEq, es_fluent::EsFluent, Filterable)]
#[filter(fluent)]
pub enum UserStatus {
    Active,
    Suspended,
}

#[derive(Clone, EsFluentLabel, EsFluentVariants, GpuiTable)]
#[fluent_label(origin, variants)]
#[fluent_variants(keys = ["label"])]
#[gpui_table(fluent = "label", filters)]
pub struct User {
    #[gpui_table(filter(faceted()))]
    pub status: UserStatus,
}
```

Use the application's existing Fluent resource layout and locale-selection pattern. Keep generated table labels and faceted enum labels in the same localization system as the rest of the app.

## Custom Rendering

Use `#[gpui_table(custom_style)]`, implement `TableRowStyle`, and fall back to `default_render_cell` for standard columns.

```rust
impl gpui_table::runtime::TableRowStyle for Item {
    type ColumnId = ItemTableColumn;

    fn render_table_cell(
        &self,
        col: Self::ColumnId,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> gpui::AnyElement {
        use gpui::IntoElement as _;

        match col {
            ItemTableColumn::Weight => {
                use gpui::{ParentElement, div};
                div().child(format!("{} kg", self.weight)).into_any_element()
            },
            _ => gpui_table::runtime::default_render_cell(self, col.into(), window, cx)
                .into_any_element(),
        }
    }
}
```

## Context Menus

Generated row-context-menu links use:

- `context_menu_row_id = "field_name"` or field-level `#[gpui_table(context_menu_id)]`
- `context_menu_route = "/users/{id}"` or `context_menu_route_fn = path::to_fn`
- `context_menu_label = "Open"` or `context_menu_label_fn = path::to_fn`
- `custom_context_menu` to compose generated links with custom menu items

Prefer route and label helper functions when the app already centralizes routing or translation outside the row type.

## Direct Component Use

Use `gpui-table-component` when composing filter UI manually.

```rust
use gpui::{StyleRefinement, px};
use gpui_table_component::{TableStatusBar, TextFilter, TextFilterExt};

let filter = TextFilter::new("Name", String::new(), move |_value, _window, _cx| {}, cx)
    .alphanumeric_only(cx)
    .container_style(StyleRefinement::default().w_full(), cx)
    .input_style(StyleRefinement::default().w(px(280.)), cx);

let status = TableStatusBar::new(rows.len(), loading, eof).row_label("Rows");
```
