use some_lib::structs::user::*;
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
pub struct UserTableStory {
    table: Entity<TableState<UserTableDelegate>>,
    filter_name: String,
    filter_age: (Option<f64>, Option<f64>),
    filter_debt: (Option<f64>, Option<f64>),
    filter_email: String,
    filter_active: std::collections::HashSet<String>,
    filter_status: std::collections::HashSet<String>,
    filter_created_at: (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>),
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
        let mut delegate = UserTableDelegate::new(vec![]);
        for _ in 0..100 {
            delegate.rows.push(fake::Faker.fake());
        }
        let table = cx.new(|cx| TableState::new(delegate, window, cx));
        Self {
            table,
            filter_name: String::new(),
            filter_age: (None, None),
            filter_debt: (None, None),
            filter_email: String::new(),
            filter_active: Default::default(),
            filter_status: Default::default(),
            filter_created_at: (None, None),
        }
    }
}
impl Render for UserTableStory {
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
                        gpui_table_components::number_range_filter::NumberRangeFilter::build(
                            "Age",
                            self.filter_age,
                            move |new_val, window, cx| {
                                view.update(
                                    cx,
                                    |this, cx| {
                                        this.filter_age = new_val;
                                        cx.notify();
                                    },
                                );
                            },
                            cx,
                        ),
                    )
                    .child(
                        gpui_table_components::number_range_filter::NumberRangeFilter::build(
                            "Debt",
                            self.filter_debt,
                            move |new_val, window, cx| {
                                view.update(
                                    cx,
                                    |this, cx| {
                                        this.filter_debt = new_val;
                                        cx.notify();
                                    },
                                );
                            },
                            cx,
                        ),
                    )
                    .child(
                        gpui_table_components::text_filter::TextFilter::build(
                            "Email",
                            self.filter_email.clone(),
                            move |new_val, window, cx| {
                                view.update(
                                    cx,
                                    |this, cx| {
                                        this.filter_email = new_val;
                                        cx.notify();
                                    },
                                );
                            },
                            cx,
                        ),
                    )
                    .child(
                        gpui_table_components::faceted_filter::FacetedFilter::build(
                            "Active",
                            <bool as gpui_table::filter::Filterable>::options(),
                            self.filter_active.clone(),
                            move |new_val, window, cx| {
                                view.update(
                                    cx,
                                    |this, cx| {
                                        this.filter_active = new_val;
                                        cx.notify();
                                    },
                                );
                            },
                            cx,
                        ),
                    )
                    .child(
                        gpui_table_components::faceted_filter::FacetedFilter::build(
                            "Status",
                            <UserStatus as gpui_table::filter::Filterable>::options(),
                            self.filter_status.clone(),
                            move |new_val, window, cx| {
                                view.update(
                                    cx,
                                    |this, cx| {
                                        this.filter_status = new_val;
                                        cx.notify();
                                    },
                                );
                            },
                            cx,
                        ),
                    )
                    .child(
                        gpui_table_components::date_range_filter::DateRangeFilter::build(
                            "Created At",
                            self.filter_created_at,
                            move |new_val, window, cx| {
                                view.update(
                                    cx,
                                    |this, cx| {
                                        this.filter_created_at = new_val;
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
