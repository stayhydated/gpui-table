use some_lib::structs::item::*;
use gpui_kit::{
    App, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement, Render,
    Styled, Subscription, Window,
};
use gpui_kit::component::v_flex;
use gpui_kit::component::table::{DataTable, TableDelegate as _, TableState};
#[gpui_storybook::story]
#[derive(gpui_storybook::StoryControls)]
pub struct ItemTableStory {
    table: Entity<TableState<ItemTableDelegate>>,
    _subscription: Subscription,
}
impl gpui_storybook::Story for ItemTableStory {
    fn title(cx: &gpui_kit::App) -> String {
        gpui_table_component::i18n::localize_label::<Item>(cx)
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        Self::view(window, cx)
    }
}
impl Focusable for ItemTableStory {
    fn focus_handle(&self, cx: &gpui_kit::App) -> gpui_kit::FocusHandle {
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
                    use gpui_table::runtime::TableDataLoader as _;
                    table.delegate_mut().load_data(window, cx);
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
