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
use some_lib::structs::infinite_row::{
    InfiniteRow, InfiniteRowFilterEntities, InfiniteRowTableDelegate,
};

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story]
pub struct InfiniteRowStory {
    table: Entity<TableState<InfiniteRowTableDelegate>>,
    filters: InfiniteRowFilterEntities,
    _subscription: Subscription,
}

impl gpui_storybook::Story for InfiniteRowStory {
    fn title() -> String {
        InfiniteRow::this_ftl()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for InfiniteRowStory {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}

impl InfiniteRowStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = InfiniteRowTableDelegate::new(vec![]);
        let table = cx.new(|cx| TableState::new(delegate, window, cx));

        // Trigger initial load
        table.update(cx, |table, cx| {
            table.delegate_mut().load_more_data(window, cx);
        });

        // Build filter entities with reload callback for server-side filtering
        let table_for_reload = table.clone();
        let filters = InfiniteRowFilterEntities::build(
            &table,
            Some(Arc::new(move |window, cx| {
                table_for_reload.update(cx, |table, cx| {
                    // Reset and reload when filters change
                    table.delegate_mut().rows.clear();
                    table.delegate_mut().eof = false;
                    table.delegate_mut().load_more_data(window, cx);
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

impl Render for InfiniteRowStory {
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
                    .child(format!("Total Rows: {}", delegate.rows.len()))
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
