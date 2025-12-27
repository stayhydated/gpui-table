use std::sync::Arc;

use es_fluent::ThisFtl as _;
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, ParentElement, Render, Styled, Subscription,
    Window,
};
use gpui_component::{
    h_flex,
    table::{Table, TableState},
    v_flex,
};
use some_lib::structs::item::{Item, ItemFilterEntities, ItemTableDelegate};

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story]
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
            &table,
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
        let table = self.table.read(cx);
        let delegate = table.delegate();

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
            .child(
                h_flex()
                    .gap_4()
                    .child(format!("Items Loaded: {}", delegate.rows.len()))
                    .child(if delegate.loading {
                        "Loading..."
                    } else {
                        "Idle"
                    })
                    .child(if delegate.eof {
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
