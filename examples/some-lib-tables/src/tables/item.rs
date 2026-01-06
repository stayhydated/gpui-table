use es_fluent::ThisFtl as _;
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, ParentElement, Render, Styled, Subscription,
    Window,
};
use gpui_component::{
    table::{Table, TableState},
    v_flex,
};
use gpui_table_components::TableStatusBar;
use some_lib::structs::item::{Item, ItemTableDelegate};

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story("fake infinite")]
pub struct ItemTableStory {
    table: Entity<TableState<ItemTableDelegate>>,
    _subscription: Subscription,
}

impl gpui_storybook::Story for ItemTableStory {
    fn title() -> String {
        Item::this_ftl()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for ItemTableStory {
    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}

impl ItemTableStory {
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

        let _subscription = cx.observe(&table, |_, _, cx| cx.notify());

        Self {
            table,
            _subscription,
        }
    }
}

impl Render for ItemTableStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let table = self.table.read(cx);
        let delegate = table.delegate();
        let row_count = delegate.rows.len();
        let loading = delegate.loading;
        let eof = delegate.eof;

        v_flex()
            .size_full()
            .gap_4()
            .p_4()
            .child(TableStatusBar::new(row_count, loading, eof))
            .child(
                Table::new(&self.table)
                    .stripe(true)
                    .scrollbar_visible(true, true),
            )
    }
}
