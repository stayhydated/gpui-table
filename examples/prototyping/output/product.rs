use some_lib::structs::product::*;
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
pub struct ProductTableStory {
    table: Entity<TableState<ProductTableDelegate>>,
    filters: ProductFilterEntities,
    _subscription: Subscription,
}
impl gpui_storybook::Story for ProductTableStory {
    fn title() -> String {
        Product::this_ftl()
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}
impl Focusable for ProductTableStory {
    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}
impl ProductTableStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = ProductTableDelegate::new(vec![]);
        let table = cx.new(|cx| TableState::new(delegate, window, cx));
        table
            .update(
                cx,
                |table, cx| {
                    use gpui_table::TableDataLoader as _;
                    table.delegate_mut().load_data(window, cx);
                },
            );
        let table_for_reload = table.clone();
        let filters = ProductFilterEntities::build(
            Some(
                std::rc::Rc::new(move |window, cx| {
                    table_for_reload
                        .update(
                            cx,
                            |table, cx| {
                                table.delegate_mut().rows.clear();
                                table.delegate_mut().eof = false;
                                use gpui_table::TableDataLoader as _;
                                table.delegate_mut().load_data(window, cx);
                            },
                        );
                }),
            ),
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
impl Render for ProductTableStory {
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
