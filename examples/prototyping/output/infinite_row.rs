use some_lib::structs::infinite_row::*;
use fake::Fake;
use gpui::{
    App, AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render,
    Styled, Window,
};
use gpui_component::{
    table::{Table, TableState, TableDelegate as _},
    v_flex,
};
use es_fluent::ToFluentString as _;
#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}
#[gpui_storybook::story]
pub struct InfiniteRowTableStory {
    table: Entity<TableState<InfiniteRowTableDelegate>>,
    filter_name: String,
    filter_description: String,
}
impl gpui_storybook::Story for InfiniteRowTableStory {
    fn title() -> String {
        InfiniteRow::this_ftl()
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}
impl Focusable for InfiniteRowTableStory {
    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}
impl InfiniteRowTableStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut delegate = InfiniteRowTableDelegate::new(vec![]);
        for _ in 0..100 {
            delegate.rows.push(fake::Faker.fake());
        }
        let table = cx.new(|cx| TableState::new(delegate, window, cx));
        Self {
            table,
            filter_name: String::new(),
            filter_description: String::new(),
        }
    }
}
impl Render for InfiniteRowTableStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let table = &self.table.read(cx);
        let delegate = table.delegate();
        let rows_count = delegate.rows_count(cx);
        let view = cx.entity().clone();
        v_flex()
            .size_full()
            .text_sm()
            .gap_4()
            .child(
                gpui_component::h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(
                        gpui_table_components::text_filter::TextFilter::build(
                            "Name",
                            self.filter_name.clone(),
                            move |new_val, window, cx| {
                                view.update(
                                    cx,
                                    |this, cx| {
                                        this.filter_name = new_val;
                                        cx.notify();
                                    },
                                );
                            },
                            cx,
                        ),
                    )
                    .child(
                        gpui_table_components::text_filter::TextFilter::build(
                            "Description",
                            self.filter_description.clone(),
                            move |new_val, window, cx| {
                                view.update(
                                    cx,
                                    |this, cx| {
                                        this.filter_description = new_val;
                                        cx.notify();
                                    },
                                );
                            },
                            cx,
                        ),
                    ),
            )
            .child(format!("Total Rows: {}", rows_count))
            .child(Table::new(&self.table))
    }
}
