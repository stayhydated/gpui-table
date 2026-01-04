use some_lib::structs::item::*;
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement, Render,
    Styled, Subscription, Window,
};
use gpui_component::{
    h_flex, table::{Table, TableState},
    v_flex,
};
use es_fluent::ThisFtl as _;
#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}
#[gpui_storybook::story]
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
        table
            .update(
                cx,
                |table, cx| {
                    table.delegate_mut().load_more(window, cx);
                },
            );
        let _subscription = cx.observe(&table, |_, _, cx| cx.notify());
        Self { table, _subscription }
    }
}
impl Render for ItemTableStory {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let table = self.table.read(cx);
        let delegate = table.delegate();
        v_flex()
            .size_full()
            .gap_4()
            .p_4()
            .child(
                h_flex()
                    .gap_4()
                    .child(format!("Items Loaded: {}", delegate.rows.len()))
                    .child(if delegate.loading { "Loading..." } else { "Idle" })
                    .child(
                        if delegate.eof { "All data loaded" } else { "Scroll for more" },
                    ),
            )
            .child(Table::new(&self.table).stripe(true).scrollbar_visible(true, true))
    }
}
