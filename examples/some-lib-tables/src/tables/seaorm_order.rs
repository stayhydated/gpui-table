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
use some_lib::structs::seaorm_order::{Model, ModelFilterEntities, ModelTableDelegate};

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story("seaorm")]
pub struct SeaormOrderStory {
    table: Entity<TableState<ModelTableDelegate>>,
    filters: ModelFilterEntities,
    _subscription: Subscription,
}

impl gpui_storybook::Story for SeaormOrderStory {
    fn title() -> String {
        Model::this_ftl()
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
        let delegate = ModelTableDelegate::new(vec![]);
        let table = cx.new(|cx| TableState::new(delegate, window, cx));

        // Trigger initial load
        table.update(cx, |table, cx| {
            table.delegate_mut().load_more(window, cx);
        });

        // Use holder pattern for filter callback
        let filter_holder: Rc<RefCell<Option<ModelFilterEntities>>> = Rc::new(RefCell::new(None));
        let filter_holder_for_callback = filter_holder.clone();
        let table_for_reload = table.clone();

        // Build filters with callback
        let filters = ModelFilterEntities::build(
            Some(Arc::new(move |window, cx| {
                if let Some(ref filters) = *filter_holder_for_callback.borrow() {
                    let filter_values = filters.read_values(cx);

                    table_for_reload.update(cx, |table, cx| {
                        table.delegate_mut().reset_and_reload_with_filters(
                            filter_values,
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
        let filters = self.filters.read_values(cx);

        let table = self.table.read(cx);
        let delegate = table.delegate();
        let row_count = delegate.rows.len();
        let loading = delegate.loading;
        let eof = delegate.eof;

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
            .when(filters.has_active_filters(), |this| {
                this.child(
                    h_flex()
                        .flex_wrap()
                        .gap_2()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("Query:")
                        .when(filters.customer_name.is_active(), |c| {
                            c.child(format!(".filter(customer LIKE '%{}%')", filters.customer_name))
                        })
                        .when(filters.total_amount.is_active(), |c| {
                            let min = filters.total_amount.min().map_or("*".to_string(), |v| format!("${:.2}", v));
                            let max = filters.total_amount.max().map_or("*".to_string(), |v| format!("${:.2}", v));
                            c.child(format!(".filter(amount: {}..{})", min, max))
                        })
                        .when(filters.status.is_active(), |c| {
                            c.child(format!(".filter(status IN {:?})", filters.status))
                        })
                        .when(filters.shipping_method.is_active(), |c| {
                            c.child(format!(".filter(shipping IN {:?})", filters.shipping_method))
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
