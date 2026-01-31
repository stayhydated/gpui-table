use some_lib::structs::user::*;
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement, Render,
    Styled, Subscription, Window,
};
use gpui_table::filter::{FilterEntitiesExt as _, Matchable as _};
use gpui_component::{
    h_flex, table::{Table, TableState, TableDelegate as _},
    v_flex,
};
use es_fluent::ThisFtl as _;
#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}
#[gpui_storybook::story]
pub struct UserTableStory {
    table: Entity<TableState<UserTableDelegate>>,
    filters: UserFilterEntities,
    _subscription: Subscription,
}
impl gpui_storybook::Story for UserTableStory {
    fn title() -> String {
        User::this_ftl()
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}
impl Focusable for UserTableStory {
    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}
impl UserTableStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = UserTableDelegate::new(vec![]);
        let table = cx.new(|cx| TableState::new(delegate, window, cx));
        let filters = UserFilterEntities::build(None, cx);
        let _subscription = cx.observe(&table, |_, _, cx| cx.notify());
        Self {
            table,
            filters,
            _subscription,
        }
    }
}
impl Render for UserTableStory {
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
            .child(h_flex().gap_2().flex_wrap().child(self.filters.all_filters()))
            .child(
                gpui_table_component::TableStatusBar::new(
                    delegate.rows.len(),
                    delegate.loading,
                    delegate.eof,
                ),
            )
            .child(Table::new(&self.table).stripe(true).scrollbar_visible(true, true))
    }
}
