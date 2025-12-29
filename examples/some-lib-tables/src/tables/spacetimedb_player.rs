//! SpacetimeDB Player Table Story
//!
//! Demonstrates using gpui-table with SpacetimeDB for real-time game data.

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
use some_lib::structs::spacetimedb_player::{
    SpacetimedbPlayer, SpacetimedbPlayerFilterEntities, SpacetimedbPlayerTableDelegate,
};

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story("spacetimedb")]
pub struct SpacetimedbPlayerStory {
    table: Entity<TableState<SpacetimedbPlayerTableDelegate>>,
    filters: SpacetimedbPlayerFilterEntities,
    _subscription: Subscription,
}

impl gpui_storybook::Story for SpacetimedbPlayerStory {
    fn title() -> String {
        SpacetimedbPlayer::this_ftl()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for SpacetimedbPlayerStory {
    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}

impl SpacetimedbPlayerStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = SpacetimedbPlayerTableDelegate::new(vec![]);
        let table = cx.new(|cx| TableState::new(delegate, window, cx));

        // Trigger initial load
        table.update(cx, |table, cx| {
            table.delegate_mut().load_more(window, cx);
        });

        // Use holder pattern for filter callback
        let filter_holder: Rc<RefCell<Option<SpacetimedbPlayerFilterEntities>>> =
            Rc::new(RefCell::new(None));
        let filter_holder_for_callback = filter_holder.clone();
        let table_for_reload = table.clone();

        // Build filters with callback
        let filters = SpacetimedbPlayerFilterEntities::build(
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

impl Render for SpacetimedbPlayerStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let filters = self.filters.read_values(cx);

        let table = self.table.read(cx);
        let delegate = table.delegate();
        let row_count = delegate.rows.len();
        let loading = delegate.loading;
        let eof = delegate.eof;

        let has_filters = !filters.username.is_empty()
            || filters.level.0.is_some()
            || filters.level.1.is_some()
            || !filters.guild.is_empty()
            || !filters.status.is_empty();

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
                            .child("SpacetimeDB Player Management"),
                    )
                    .child(
                        gpui::div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Real-time multiplayer game data with server-side filtering"),
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
                            .child("SpacetimeDB Subscription Filters"),
                    )
                    .child(self.filters.all_filters()),
            )
            // Active filter display
            .when(has_filters, |this| {
                this.child(
                    h_flex()
                        .gap_2()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("Active filters:")
                        .when(!filters.username.is_empty(), |c| {
                            c.child(format!("username LIKE '%{}%'", filters.username))
                        })
                        .when(filters.level.0.is_some() || filters.level.1.is_some(), |c| {
                            let min = filters.level.0.map_or("*".to_string(), |v| v.to_string());
                            let max = filters.level.1.map_or("*".to_string(), |v| v.to_string());
                            c.child(format!("level: {}..{}", min, max))
                        })
                        .when(!filters.guild.is_empty(), |c| {
                            c.child(format!("guild IN {:?}", filters.guild))
                        })
                        .when(!filters.status.is_empty(), |c| {
                            c.child(format!("status IN {:?}", filters.status))
                        }),
                )
            })
            // Status bar
            .child(
                h_flex()
                    .gap_4()
                    .text_sm()
                    .child(format!("Players: {}", row_count))
                    .child(if loading {
                        "Receiving from SpacetimeDB..."
                    } else {
                        "Subscribed"
                    })
                    .child(if eof {
                        "All players loaded"
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
