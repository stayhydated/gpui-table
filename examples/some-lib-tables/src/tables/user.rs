use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use es_fluent::ThisFtl as _;
use fake::{Fake, Faker};
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement, Render, Styled,
    Subscription, Window,
};
use gpui_component::{
    h_flex,
    table::{Table, TableState},
    v_flex,
};
use gpui_table::filter::{FilterEntitiesExt as _, Matchable as _};
use some_lib::structs::user::*;

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story]
pub struct UserTableStory {
    table: Entity<TableState<UserTableDelegate>>,
    filters: UserFilterEntities,
    /// Store all generated data here, filter into table.rows
    all_data: Rc<RefCell<Vec<User>>>,
    _subscription: Subscription,
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
        let delegate = UserTableDelegate::new(vec![]);
        let table = cx.new(|cx| TableState::new(delegate, window, cx));

        // Shared storage for all generated data
        let all_data: Rc<RefCell<Vec<User>>> = Rc::new(RefCell::new(Vec::new()));

        // Trigger initial load
        let all_data_for_load = all_data.clone();
        let table_for_load = table.clone();
        Self::load_initial_data(table_for_load, all_data_for_load, cx);

        // Use a holder pattern to allow the callback to access filter values
        let filter_holder: Rc<RefCell<Option<UserFilterEntities>>> = Rc::new(RefCell::new(None));
        let filter_holder_for_callback = filter_holder.clone();
        let table_for_reload = table.clone();
        let all_data_for_filter = all_data.clone();

        // Build all filter entities with a callback that reads current filter values
        let filters = UserFilterEntities::build(
            Some(Rc::new(move |_window, cx| {
                if let Some(ref filters) = *filter_holder_for_callback.borrow() {
                    let filter_values = filters.read_values(cx);

                    // Filter the stored data and update table rows
                    let all = all_data_for_filter.borrow();
                    let filtered: Vec<User> = all
                        .iter()
                        .filter(|user| user.matches_filters(&filter_values))
                        .cloned()
                        .collect();

                    table_for_reload.update(cx, |table, cx| {
                        table.delegate_mut().rows = filtered;
                        cx.notify();
                    });
                }
            })),
            cx,
        );

        // Populate the holder so the callback can access filter values
        *filter_holder.borrow_mut() = Some(filters.clone());

        let _subscription = cx.observe(&table, |_, _, cx| cx.notify());
        Self {
            table,
            filters,
            all_data,
            _subscription,
        }
    }

    fn load_initial_data(
        table: Entity<TableState<UserTableDelegate>>,
        all_data: Rc<RefCell<Vec<User>>>,
        cx: &mut Context<Self>,
    ) {
        table.update(cx, |table, cx| {
            table.delegate_mut().loading = true;
            cx.notify();

            cx.spawn(async move |view, cx| {
                // Simulate network delay
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;

                // Generate fake data
                let new_rows: Vec<User> = (0..500).map(|_| Faker.fake()).collect();

                _ = cx.update(|cx| {
                    // Store in all_data
                    *all_data.borrow_mut() = new_rows.clone();

                    view.update(cx, |table, cx| {
                        let delegate = table.delegate_mut();
                        delegate.rows = new_rows;
                        delegate.loading = false;
                        delegate.eof = true;
                        cx.notify();
                    })
                    .ok();
                });
            })
            .detach();
        });
    }
}

impl Render for UserTableStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let table = self.table.read(cx);
        let delegate = table.delegate();
        v_flex()
            .size_full()
            .gap_4()
            .p_4()
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(self.filters.all_filters()),
            )
            .child(gpui_table_components::TableStatusBar::new(
                delegate.rows.len(),
                delegate.loading,
                delegate.eof,
            ))
            .child(
                Table::new(&self.table)
                    .stripe(true)
                    .scrollbar_visible(true, true),
            )
    }
}
