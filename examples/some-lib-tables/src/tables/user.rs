use es_fluent::ThisFtl as _;
use fake::{Fake, Faker};
#[cfg(feature = "router")]
use gpui::InteractiveElement as _;
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement, Render, Styled,
    Subscription, Window,
};
use gpui_component::table::{DataTable, TableState};
use gpui_component::{h_flex, v_flex};
use gpui_table::filter::FilterEntitiesExt as _;
use some_lib::structs::user::*;
#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}
#[gpui_storybook::story]
pub struct UserTableStory {
    table: Entity<TableState<UserTableDelegate>>,
    filters: UserFilterEntities,
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
        let users: Vec<User> = (0..200).map(|_| Faker.fake()).collect();
        let delegate = UserTableDelegate::new(users);
        let table = cx.new(|cx| TableState::new(delegate, window, cx));
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
        if !cx.has_global::<gpui_router::RouterState>() {
            gpui_router::init(cx);
        }

        gpui_router::RouterState::global_mut(cx).with_path(action.0.clone());
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
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(self.filters.all_filters()),
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
            gpui::div().text_sm().child(format!(
                "Router location: {}",
                gpui_router::use_location(cx).pathname
            )),
        );

        root
    }
}
