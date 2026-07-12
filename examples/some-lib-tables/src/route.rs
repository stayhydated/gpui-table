use gpui::{App, Global};
use some_lib::structs::context_menu_common::UserRoute;
use std::fmt;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum ExampleRoute {
    #[default]
    Home,
    User(UserRoute),
}

impl ExampleRoute {
    pub fn path(&self) -> String {
        match self {
            Self::Home => "/".to_string(),
            Self::User(route) => route.to_string(),
        }
    }
}

impl fmt::Display for ExampleRoute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.path())
    }
}

#[derive(Clone, Default, Eq, PartialEq)]
pub struct ExampleRouterState {
    route: ExampleRoute,
}

impl Global for ExampleRouterState {}

impl ExampleRouterState {
    pub fn init(cx: &mut App) {
        cx.set_global(Self::default());
    }

    pub fn route(&self) -> &ExampleRoute {
        &self.route
    }

    pub fn set_route(&mut self, route: ExampleRoute) {
        self.route = route;
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }
}

pub fn init(cx: &mut App) {
    ExampleRouterState::init(cx);
}
