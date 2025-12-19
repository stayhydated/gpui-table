use some_lib::structs::item::*;
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
pub struct ItemTableStory {
    table: Entity<TableState<ItemTableDelegate>>,
    filter_name: String,
    filter_color: String,
    filter_weight: (Option<f64>, Option<f64>),
    filter_acquired_on: (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>),
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
        let mut delegate = ItemTableDelegate::new(vec![]);
        for _ in 0..100 {
            delegate.rows.push(fake::Faker.fake());
        }
        let table = cx.new(|cx| TableState::new(delegate, window, cx));
        Self {
            table,
            filter_name: String::new(),
            filter_color: String::new(),
            filter_weight: (None, None),
            filter_acquired_on: (None, None),
        }
    }
}
impl Render for ItemTableStory {
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
                            "Color",
                            self.filter_color.clone(),
                            move |new_val, window, cx| {
                                view.update(
                                    cx,
                                    |this, cx| {
                                        this.filter_color = new_val;
                                        cx.notify();
                                    },
                                );
                            },
                            cx,
                        ),
                    )
                    .child(
                        gpui_table_components::number_range_filter::NumberRangeFilter::build(
                            "Weight",
                            self.filter_weight,
                            move |new_val, window, cx| {
                                view.update(
                                    cx,
                                    |this, cx| {
                                        this.filter_weight = new_val;
                                        cx.notify();
                                    },
                                );
                            },
                            cx,
                        ),
                    )
                    .child(
                        gpui_table_components::date_range_filter::DateRangeFilter::build(
                            "Acquired On",
                            self.filter_acquired_on,
                            move |new_val, window, cx| {
                                view.update(
                                    cx,
                                    |this, cx| {
                                        this.filter_acquired_on = new_val;
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
