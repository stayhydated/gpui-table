//! SeaORM Order Table Story
//!
//! Demonstrates using gpui-table with SeaORM for database-backed order management.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use es_fluent::ThisFtl as _;
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, ParentElement, Render, Styled, Subscription,
    Window, prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, h_flex,
    table::{Table, TableState},
    v_flex,
};
use some_lib::structs::seaorm_order::{
    SeaormOrder, SeaormOrderFilterEntities, SeaormOrderTableDelegate,
};

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story("seaorm")]
pub struct SeaormOrderStory {
    table: Entity<TableState<SeaormOrderTableDelegate>>,
    filters: SeaormOrderFilterEntities,
    _subscription: Subscription,
}

impl gpui_storybook::Story for SeaormOrderStory {
    fn title() -> String {
        SeaormOrder::this_ftl()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for SeaormOrderStory {
    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}

impl SeaormOrderStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = SeaormOrderTableDelegate::new(vec![]);
        let table = cx.new(|cx| TableState::new(delegate, window, cx));

        // Trigger initial load
        table.update(cx, |table, cx| {
            table.delegate_mut().load_more(window, cx);
        });

        // Use holder pattern for filter callback
        let filter_holder: Rc<RefCell<Option<SeaormOrderFilterEntities>>> =
            Rc::new(RefCell::new(None));
        let filter_holder_for_callback = filter_holder.clone();
        let table_for_reload = table.clone();

        // Build filters with callback
        let filters = SeaormOrderFilterEntities::build(
            Some(Arc::new(move |window, cx| {
                if let Some(ref filters) = *filter_holder_for_callback.borrow() {
                    let customer_filter = filters.customer_name_value(cx);
                    let email_filter = filters.customer_email_value(cx);
                    let amount_range = filters.total_amount_value(cx);
                    let status_filter = filters.status_value(cx);
                    let shipping_filter = filters.shipping_method_value(cx);
                    let date_range = filters.created_at_value(cx);

                    table_for_reload.update(cx, |table, cx| {
                        table.delegate_mut().reset_and_reload_with_filters(
                            customer_filter,
                            email_filter,
                            amount_range,
                            status_filter,
                            shipping_filter,
                            date_range,
                            window,
                            cx,
                        );
                    });
                }
            })),
            cx,
        );

        *filter_holder.borrow_mut() = Some(filters.clone());

        let _subscription = cx.observe(&table, |_, _, cx| cx.notify());

        Self {
            table,
            filters,
            _subscription,
        }
    }
}

impl Render for SeaormOrderStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let customer_filter = self.filters.customer_name_value(cx);
        let amount_range = self.filters.total_amount_value(cx);
        let status_filter = self.filters.status_value(cx);
        let shipping_filter = self.filters.shipping_method_value(cx);
        let date_range = self.filters.created_at_value(cx);

        let table = self.table.read(cx);
        let delegate = table.delegate();
        let row_count = delegate.rows.len();
        let loading = delegate.loading;
        let eof = delegate.eof;

        let has_filters = !customer_filter.is_empty()
            || amount_range.0.is_some()
            || amount_range.1.is_some()
            || !status_filter.is_empty()
            || !shipping_filter.is_empty()
            || date_range.0.is_some()
            || date_range.1.is_some();

        v_flex()
            .size_full()
            .gap_4()
            .p_4()
            // Header
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        gpui::div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("SeaORM Order Management"),
                    )
                    .child(
                        gpui::div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Database-backed orders with SeaORM pagination and filtering"),
                    ),
            )
            // Filters
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        gpui::div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("SeaORM Query Filters"),
                    )
                    .child(self.filters.all_filters()),
            )
            // Active filter display
            .when(has_filters, |this| {
                this.child(
                    h_flex()
                        .flex_wrap()
                        .gap_2()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("Query:")
                        .when(!customer_filter.is_empty(), |c| {
                            c.child(format!(".filter(customer LIKE '%{}%')", customer_filter))
                        })
                        .when(amount_range.0.is_some() || amount_range.1.is_some(), |c| {
                            let min = amount_range.0.map_or("*".to_string(), |v| format!("${:.2}", v));
                            let max = amount_range.1.map_or("*".to_string(), |v| format!("${:.2}", v));
                            c.child(format!(".filter(amount: {}..{})", min, max))
                        })
                        .when(!status_filter.is_empty(), |c| {
                            c.child(format!(".filter(status IN {:?})", status_filter))
                        })
                        .when(!shipping_filter.is_empty(), |c| {
                            c.child(format!(".filter(shipping IN {:?})", shipping_filter))
                        }),
                )
            })
            // Status bar
            .child(
                h_flex()
                    .gap_4()
                    .text_sm()
                    .child(format!("Orders: {}", row_count))
                    .child(if loading {
                        "Fetching page..."
                    } else {
                        "Ready"
                    })
                    .child(if eof {
                        "All pages loaded"
                    } else {
                        "Scroll for more"
                    }),
            )
            // Table
            .child(
                Table::new(&self.table)
                    .stripe(true)
                    .scrollbar_visible(true, true),
            )
    }
}
