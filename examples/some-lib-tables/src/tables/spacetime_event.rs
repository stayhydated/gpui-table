use es_fluent::ThisFtl as _;
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement, Render, Styled,
    Subscription, Window,
};
use gpui_component::table::{DataTable, TableState};
use gpui_component::{h_flex, v_flex};
use gpui_table::filter::FilterEntitiesExt as _;
use some_lib::structs::spacetime_event::*;

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story]
pub struct SpacetimeEventTableStory {
    table: Entity<TableState<SpacetimeEventTableDelegate>>,
    filters: SpacetimeEventFilterEntities,
    _subscription: Subscription,
}

impl gpui_storybook::Story for SpacetimeEventTableStory {
    fn title() -> String {
        SpacetimeEvent::this_ftl()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for SpacetimeEventTableStory {
    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}

impl SpacetimeEventTableStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = SpacetimeEventTableDelegate::new(vec![]);
        let table = cx.new(|cx| TableState::new(delegate, window, cx));

        let filters_slot: std::rc::Rc<std::cell::RefCell<Option<SpacetimeEventFilterEntities>>> =
            std::rc::Rc::new(std::cell::RefCell::new(None));
        let filters_slot_for_change = filters_slot.clone();
        let table_for_reload = table.clone();

        let on_filter_change: std::rc::Rc<dyn Fn(&mut Window, &mut App) + 'static> =
            std::rc::Rc::new(move |window, cx| {
                let next_values = {
                    let filters = filters_slot_for_change.borrow();
                    filters.as_ref().map(|filters| filters.read_values(cx))
                };

                if let Some(values) = next_values {
                    table_for_reload.update(cx, |table, cx| {
                        table.delegate_mut().set_filter_values(values);
                        table.delegate_mut().rows.clear();
                        table.delegate_mut().eof = false;
                        use gpui_table::TableDataLoader as _;
                        table.delegate_mut().load_data(window, cx);
                    });
                }
            });

        let filters = SpacetimeEventFilterEntities::build(Some(on_filter_change), cx);
        *filters_slot.borrow_mut() = Some(filters.clone());

        let initial_values = filters.read_values(cx);
        table.update(cx, |table, cx| {
            table.delegate_mut().set_filter_values(initial_values);
            use gpui_table::TableDataLoader as _;
            table.delegate_mut().load_data(window, cx);
        });

        let table_for_live_reload = table.clone();
        let mut live_change_rx = some_lib::client_connection::subscribe_spacetime_event_changes();
        cx.spawn(async move |view, cx| {
            loop {
                match live_change_rx.recv().await {
                    Ok(()) => {},
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {},
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }

                let updated = cx.update(|cx| {
                    view.update(cx, |_, cx| {
                        table_for_live_reload.update(cx, |table, cx| {
                            table.delegate_mut().reload_visible_rows(cx);
                        })
                    })
                });

                let should_continue = updated.is_ok();
                if !should_continue {
                    break;
                }
            }
        })
        .detach();

        let _subscription = cx.observe(&table, |_, _, cx| cx.notify());

        Self {
            table,
            filters,
            _subscription,
        }
    }
}

impl Render for SpacetimeEventTableStory {
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
                    .child(self.filters.all_filters_with_reset()),
            )
            .child(gpui_table_component::TableStatusBar::new(
                delegate.rows.len(),
                delegate.loading,
                delegate.eof,
            ))
            .child(
                DataTable::new(&self.table)
                    .stripe(true)
                    .scrollbar_visible(true, true),
            )
    }
}
