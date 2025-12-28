use std::sync::Arc;

use es_fluent::ThisFtl as _;
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, ParentElement, Render, Styled, Subscription,
    Window, prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, h_flex,
    table::{Table, TableState},
    v_flex,
};
use some_lib::structs::item::{Item, ItemFilterEntities, ItemTableDelegate};

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story("fake")]
pub struct ItemStory {
    table: Entity<TableState<ItemTableDelegate>>,
    filters: ItemFilterEntities,
    _subscription: Subscription,
}

impl gpui_storybook::Story for ItemStory {
    fn title() -> String {
        Item::this_ftl()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for ItemStory {
    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}

impl ItemStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = ItemTableDelegate::new(vec![]);
        let table = cx.new(|cx| TableState::new(delegate, window, cx));

        // Trigger initial load
        table.update(cx, |table, cx| {
            table.delegate_mut().load_more_items(window, cx);
        });

        // Build filter entities with reload callback
        let table_for_reload = table.clone();
        let filters = ItemFilterEntities::build(
            Some(Arc::new(move |window, cx| {
                table_for_reload.update(cx, |table, cx| {
                    // Reset and reload when filters change
                    table.delegate_mut().rows.clear();
                    table.delegate_mut().eof = false;
                    table.delegate_mut().load_more_items(window, cx);
                });
            })),
            cx,
        );

        let _subscription = cx.observe(&table, |_, _, cx| cx.notify());

        Self {
            table,
            filters,
            _subscription,
        }
    }
}

impl Render for ItemStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        // Get current filter values
        let name_filter = self.filters.name_value(cx);
        let color_filter = self.filters.color_value(cx);
        let weight_filter = self.filters.weight_value(cx);
        let acquired_on_filter = self.filters.acquired_on_value(cx);

        let table = self.table.read(cx);
        let delegate = table.delegate();
        let row_count = delegate.rows.len();
        let loading = delegate.loading;
        let eof = delegate.eof;

        let has_filters = !name_filter.is_empty()
            || !color_filter.is_empty()
            || weight_filter.0.is_some()
            || weight_filter.1.is_some()
            || acquired_on_filter.0.is_some()
            || acquired_on_filter.1.is_some();

        v_flex()
            .size_full()
            .gap_4()
            .p_4()
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(self.filters.all_filters()),
            )
            // Display current filter values
            .when(has_filters, |this| {
                this.child(
                    h_flex()
                        .gap_2()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .when(!name_filter.is_empty(), |c| c.child(format!("name: \"{}\"", name_filter)))
                        .when(!color_filter.is_empty(), |c| c.child(format!("color: \"{}\"", color_filter)))
                        .when(weight_filter.0.is_some() || weight_filter.1.is_some(), |c| {
                            c.child(format!("weight: {:?}", weight_filter))
                        })
                        .when(acquired_on_filter.0.is_some() || acquired_on_filter.1.is_some(), |c| {
                            c.child(format!("acquired_on: {:?}", acquired_on_filter))
                        })
                )
            })
            .child(
                h_flex()
                    .gap_4()
                    .child(format!("Items Loaded: {}", row_count))
                    .child(if loading {
                        "Loading..."
                    } else {
                        "Idle"
                    })
                    .child(if eof {
                        "All data loaded"
                    } else {
                        "Scroll for more"
                    }),
            )
            .child(
                Table::new(&self.table)
                    .stripe(true)
                    .scrollbar_visible(true, true),
            )
    }
}
