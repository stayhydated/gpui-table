use some_lib::structs::user::*;
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement, Render,
    Styled, Subscription, Window,
};
use gpui_component::{h_flex, v_flex};
use gpui_component::table::{DataTable, TableDelegate as _, TableState};
#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}
#[gpui_storybook::story]
pub struct UserTableStory {
    table: Entity<TableState<UserTableDelegate>>,
    filters: UserFilterEntities,
    _subscription: Subscription,
}
impl gpui_storybook::Story for UserTableStory {
    fn title(cx: &gpui::App) -> String {
        gpui_table_component::i18n::localize_label::<User>(cx)
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
        let filters = UserFilterEntities::build_for_table(table.clone(), cx);
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
            .child(
                v_flex()
                    .gap_2()
                    .children(
                        self
                            .filters
                            .filter_sidebar_data(cx)
                            .into_groups()
                            .into_iter()
                            .map(|group| {
                                h_flex()
                                    .gap_2()
                                    .flex_wrap()
                                    .children(
                                        group
                                            .into_items()
                                            .into_iter()
                                            .map(|item| item.into_element()),
                                    )
                            }),
                    ),
            )
            .child(
                gpui_table_component::TableStatusBar::new(
                    delegate.rows.len(),
                    delegate.loading,
                    delegate.eof,
                ),
            )
            .child(
                DataTable::new(&self.table).stripe(true).scrollbar_visible(true, true),
            )
    }
}
