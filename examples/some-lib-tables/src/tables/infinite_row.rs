use fake::{Fake, Faker};
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, InteractiveElement, ParentElement, Render,
    Styled, Subscription, Window,
};
use gpui_component::{
    h_flex,
    table::{Table, TableState},
    v_flex,
};
use some_lib::structs::infinite_row::{InfiniteRow, InfiniteRowTableDelegate};

#[gpui_storybook::story]
pub struct InfiniteScrollStory {
    table: Entity<TableState<InfiniteRowTableDelegate>>,
    _subscription: Subscription,
}

impl gpui_storybook::Story for InfiniteScrollStory {
    fn title() -> String {
        InfiniteRow::this_ftl()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for InfiniteScrollStory {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}

impl InfiniteScrollStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut delegate = InfiniteRowTableDelegate::new(vec![]);

        // Initial data
        for _ in 0..50 {
            delegate.rows.push(Faker.fake());
        }

        let table = cx.new(|cx| TableState::new(delegate, window, cx));

        let _subscription = cx.observe(&table, |_, _, cx| cx.notify());

        Self {
            table,
            _subscription,
        }
    }
}

impl Render for InfiniteScrollStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let table = self.table.read(cx);
        let delegate = table.delegate();

        v_flex()
            .size_full()
            .gap_4()
            .p_4()
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
                        "More data available"
                    }),
            )
            .child(
                Table::new(&self.table)
                    .stripe(true)
                    .scrollbar_visible(true, true),
            )
    }
}
