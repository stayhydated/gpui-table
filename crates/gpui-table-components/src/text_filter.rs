use crate::TableFilterComponent;
use gpui::{App, Context, Entity, IntoElement, Render, Window, prelude::*, px};
use gpui_component::{
    Icon, IconName, Sizable, h_flex,
    input::{Input, InputEvent, InputState},
};
use std::rc::Rc;

pub struct TextFilter {
    title: String,
    value: String,
    input_state: Option<Entity<InputState>>,
    on_change: Rc<dyn Fn(String, &mut Window, &mut App) + 'static>,
}

impl TableFilterComponent for TextFilter {
    type Value = String;

    const FILTER_TYPE: gpui_table_core::registry::RegistryFilterType =
        gpui_table_core::registry::RegistryFilterType::Text;

    fn build(
        title: impl Into<String>,
        value: Self::Value,
        on_change: impl Fn(Self::Value, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|_cx| Self {
            title: title.into(),
            value,
            input_state: None,
            on_change: Rc::new(on_change),
        })
    }
}

impl TextFilter {
    fn ensure_input_state(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input_state.is_none() {
            let title = self.title.clone();
            let input = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(format!("Filter {}...", title))
                    .default_value(self.value.clone())
                    .clean_on_escape()
            });

            // Subscribe to input changes - InputEvent::Change is a unit variant
            cx.subscribe(&input, |this: &mut Self, state, event: &InputEvent, cx| {
                match event {
                    InputEvent::Change => {
                        let new_value = state.read(cx).value().to_string();
                        this.value = new_value;
                        cx.notify();
                    },
                    InputEvent::PressEnter { .. } => {
                        // Apply on Enter - we need to defer this since we don't have window here
                        cx.notify();
                    },
                    _ => {},
                }
            })
            .detach();

            self.input_state = Some(input);
        }
    }

    /// Apply the current filter value via callback.
    /// Call this from parent when you want to trigger the on_change.
    pub fn apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        (self.on_change)(self.value.clone(), window, cx);
    }

    /// Get the current filter value.
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl Render for TextFilter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Ensure input state exists
        self.ensure_input_state(window, cx);

        let input_state = self.input_state.clone().unwrap();

        // Inline input without popover - similar to ts-ref data-table-filter-list.tsx
        h_flex().gap_2().items_center().child(
            Input::new(&input_state)
                .small()
                .w(px(200.))
                .prefix(Icon::new(IconName::Search).xsmall())
                .cleanable(true),
        )
    }
}
