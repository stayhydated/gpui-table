#[cfg(feature = "router")]
use crate::route::{self, ExampleRoute, ExampleRouterState};
#[cfg(feature = "router")]
use gpui_kit::InteractiveElement as _;
use gpui_kit::component::table::{DataTable, TableState};
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::{
    App, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement as _, Render,
    Styled as _, Subscription, Window,
};
use some_lib::structs::user::*;
#[gpui_storybook::story]
#[derive(gpui_storybook::StoryControls)]
pub struct UserTableStory {
    table: Entity<TableState<UserTableDelegate>>,
    filters: UserFilterEntities,
    _subscription: Subscription,
}
impl gpui_storybook::Story for UserTableStory {
    fn title(cx: &gpui_kit::App) -> String {
        gpui_table_component::i18n::localize_label::<User>(cx)
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        Self::view(window, cx)
    }
}
impl Focusable for UserTableStory {
    fn focus_handle(&self, cx: &gpui_kit::App) -> gpui_kit::FocusHandle {
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
        table.update(cx, |table, cx| {
            use gpui_table::runtime::TableDataLoader as _;
            table.delegate_mut().load_data(window, cx);
        });
        let filters = UserFilterEntities::build_for_table(table.clone(), cx);
        let _subscription = cx.observe(&table, |_, _, cx| cx.notify());
        Self {
            table,
            filters,
            _subscription,
        }
    }

    #[cfg(feature = "router")]
    fn on_open_user_route(
        &mut self,
        action: &some_lib::structs::context_menu_common::OpenUserRoute,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !cx.has_global::<ExampleRouterState>() {
            route::init(cx);
        }

        ExampleRouterState::global_mut(cx).set_route(ExampleRoute::User(action.0.clone()));
        cx.notify();
    }
}
impl Render for UserTableStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let table = self.table.read(cx);
        let delegate = table.delegate();
        let root = v_flex()
            .size_full()
            .gap_4()
            .p_4()
            .child(
                v_flex().gap_2().children(
                    self.filters
                        .filter_sidebar_data(cx)
                        .into_groups()
                        .into_iter()
                        .map(|group| {
                            h_flex().gap_2().flex_wrap().children(
                                group
                                    .into_items()
                                    .into_iter()
                                    .map(|item| item.into_element()),
                            )
                        }),
                ),
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
            );

        #[cfg(feature = "router")]
        let root = root.on_action(cx.listener(Self::on_open_user_route)).child(
            gpui_kit::div().text_sm().child(format!(
                "Router location: {}",
                ExampleRouterState::global(cx).route()
            )),
        );

        root
    }
}
