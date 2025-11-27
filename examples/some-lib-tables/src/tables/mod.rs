pub mod fruit;
pub mod user;

use gpui::Action;
use gpui_component::Size;
use serde::Deserialize;

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = table_story, no_json)]
pub struct ChangeSize(Size);

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = table_story, no_json)]
pub struct OpenDetail(usize);
