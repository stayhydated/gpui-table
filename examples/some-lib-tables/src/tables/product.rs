use std::collections::HashSet;

use es_fluent::{ThisFtl as _, ToFluentString};
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, ParentElement, Render, Styled, Subscription,
    Window,
};
use gpui_component::{
    ActiveTheme, h_flex,
    table::{Table, TableState},
    v_flex,
};
use gpui_table::components::{FacetedFilter, TextFilter};
use gpui_table_components::TableFilterComponent;
use some_lib::structs::product::{
    Product, ProductCategory, ProductLabelKvFtl, ProductTableDelegate,
};

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story]
pub struct ProductStory {
    table: Entity<TableState<ProductTableDelegate>>,

    // Text filter - server-side search
    filter_title: Entity<TextFilter>,

    // Faceted filter - server-side category filter
    filter_category: Entity<FacetedFilter>,

    _subscription: Subscription,
}

impl gpui_storybook::Story for ProductStory {
    fn title() -> String {
        Product::this_ftl()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for ProductStory {
    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}

impl ProductStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = ProductTableDelegate::new(vec![]);
        let table = cx.new(|cx| TableState::new(delegate, window, cx));

        // Trigger initial load
        table.update(cx, |table, cx| {
            table.delegate_mut().load_more_products(window, cx);
        });

        // TextFilter: title (server-side search via /products/search?q=)
        let table_entity = table.clone();
        let filter_title = TextFilter::build(
            "Title",
            String::new(),
            move |value, window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.title = value;
                    table.delegate_mut().reset_and_reload(window, cx);
                });
            },
            cx,
        );

        // FacetedFilter: category (server-side via /products/category/{slug})
        let table_entity = table.clone();
        let filter_category = FacetedFilter::build_for::<ProductCategory>(
            || ProductLabelKvFtl::Category.to_fluent_string(),
            HashSet::new(),
            move |value, window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.category = value;
                    table.delegate_mut().reset_and_reload(window, cx);
                });
            },
            cx,
        );

        let _subscription = cx.observe(&table, |_, _, cx| cx.notify());

        Self {
            table,
            filter_title,
            filter_category,
            _subscription,
        }
    }
}

impl Render for ProductStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let table = self.table.read(cx);
        let delegate = table.delegate();

        v_flex()
            .size_full()
            .gap_4()
            .p_4()
            // Server-side filters row
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        gpui::div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Server-Side Filters"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(self.filter_title.clone())
                            .child(self.filter_category.clone()),
                    ),
            )
            // Status bar
            .child(
                h_flex()
                    .gap_4()
                    .child(format!("Products Loaded: {}", delegate.rows.len()))
                    .child(if delegate.loading {
                        "Fetching from DummyJSON..."
                    } else {
                        "Idle"
                    })
                    .child(if delegate.eof {
                        "All data loaded"
                    } else {
                        "Scroll for more"
                    }),
            )
            // Table
            .child(
                Table::new(&self.table)
                    .stripe(true)
                    .scrollbar_visible(true, true),
            )
    }
}
