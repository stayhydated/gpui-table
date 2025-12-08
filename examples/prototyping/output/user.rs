use es_fluent::ToFluentString as _;
use fake::Fake;
use gpui::{
    App, AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Styled, Window,
};
use gpui_component::{
    table::{Table, TableState},
    v_flex,
};
use some_lib::structs::user::*;
#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}
#[gpui_storybook::story]
pub struct UserTableStory {
    table: Entity<TableState<UserTableDelegate>>,
}
impl gpui_storybook::Story for UserTableStory {
    fn title() -> String {
        UserLabelKvFtl::this_ftl()
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
        let mut delegate = UserTableDelegate::new(vec![]);
        for _ in 0..100 {
            delegate.rows.push(fake::Faker.fake());
        }
        let table = cx.new(|cx| TableState::new(delegate, window, cx));
        Self { table }
    }
}
impl Render for UserTableStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let table = &self.table.read(cx);
        let delegate = table.delegate();
        let rows_count = delegate.rows_count(cx);
        v_flex()
            .size_full()
            .text_sm()
            .gap_4()
            .child(format!("Total Rows: {}", rows_count))
            .child(Table::new(&self.table))
    }
}
