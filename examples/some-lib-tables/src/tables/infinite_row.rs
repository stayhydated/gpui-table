use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use es_fluent::ThisFtl as _;
use fake::{Fake, Faker};
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, ParentElement, Render, Styled, Subscription,
    Window, prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, h_flex,
    table::{Table, TableState},
    v_flex,
};
use some_lib::structs::infinite_row::{
    InfiniteRow, InfiniteRowFilterEntities, InfiniteRowTableDelegate,
};

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story("fake")]
pub struct InfiniteRowStory {
    table: Entity<TableState<InfiniteRowTableDelegate>>,
    filters: InfiniteRowFilterEntities,
    /// The complete data pool (generated once, never changes)
    all_data: Rc<Vec<InfiniteRow>>,
    _subscription: Subscription,
}

impl gpui_storybook::Story for InfiniteRowStory {
    fn title() -> String {
        InfiniteRow::this_ftl()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for InfiniteRowStory {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}

impl InfiniteRowStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Generate complete data pool once at startup
        let all_data: Rc<Vec<InfiniteRow>> = Rc::new((0..500).map(|_| Faker.fake()).collect());

        // Initialize delegate with the full data set
        let delegate = InfiniteRowTableDelegate::with_data_pool((*all_data).clone());
        let table = cx.new(|cx| TableState::new(delegate, window, cx));

        // Create holders for the callback to access data pool and filters
        let data_pool_for_callback = all_data.clone();
        let filter_holder: Rc<RefCell<Option<InfiniteRowFilterEntities>>> =
            Rc::new(RefCell::new(None));
        let filter_holder_for_callback = filter_holder.clone();
        let table_for_reload = table.clone();

        // Build filter entities with client-side filtering callback
        let filters = InfiniteRowFilterEntities::build(
            Some(Arc::new(move |window, cx| {
                if let Some(ref filters) = *filter_holder_for_callback.borrow() {
                    // Read the current filter values from the filter entities
                    let name_filter = filters.name_value(cx);
                    let description_filter = filters.description_value(cx);

                    // Apply filters to the stable data pool
                    table_for_reload.update(cx, |table, cx| {
                        table.delegate_mut().apply_filters(
                            &data_pool_for_callback,
                            &name_filter,
                            &description_filter,
                            window,
                            cx,
                        );
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
}

impl Render for InfiniteRowStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        // Get current filter values
        let name_filter = self.filters.name_value(cx);
        let description_filter = self.filters.description_value(cx);

        let table = self.table.read(cx);
        let delegate = table.delegate();
        let row_count = delegate.rows.len();
        let total_count = self.all_data.len();

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
            // Display current filter values
            .when(!name_filter.is_empty() || !description_filter.is_empty(), |this| {
                this.child(
                    h_flex()
                        .gap_2()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .when(!name_filter.is_empty(), |c| c.child(format!("name: \"{}\"", name_filter)))
                        .when(!description_filter.is_empty(), |c| c.child(format!("description: \"{}\"", description_filter)))
                )
            })
            .child(
                h_flex()
                    .gap_4()
                    .child(format!("Showing: {} / {} rows", row_count, total_count))
                    .child("Client-side filtering"),
            )
            .child(
                Table::new(&self.table)
                    .stripe(true)
                    .scrollbar_visible(true, true),
            )
    }
}
