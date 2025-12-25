use crate::TableFilterComponent;
use gpui::{App, Context, Entity, IntoElement, Render, Window, div, prelude::*, px};
use gpui_component::{
    Icon, IconName, Sizable,
    button::{Button, ButtonVariants},
    divider::Divider,
    input::{Input, InputState},
    popover::Popover,
    v_flex,
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
    /// Legacy builder method for backwards compatibility.
    #[deprecated(note = "Use TableFilterComponent::build instead")]
    pub fn new(
        title: impl Into<String>,
        value: String,
        on_change: impl Fn(String, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        <Self as TableFilterComponent>::build(title, value, on_change, cx)
    }

    fn ensure_input_state(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input_state.is_none() {
            let input = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("Filter...")
                    .default_value(self.value.clone())
                    .clean_on_escape()
            });
            self.input_state = Some(input);
        }
    }

    fn apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(input) = &self.input_state {
            let new_value = input.read(cx).value().to_string();
            self.value = new_value.clone();
            (self.on_change)(new_value, window, cx);
            cx.notify();
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.value.clear();
        if let Some(input) = &self.input_state {
            input.update(cx, |state, cx| {
                state.set_value("", window, cx);
            });
        }
        (self.on_change)(String::new(), window, cx);
        cx.notify();
    }
}

impl Render for TextFilter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Ensure input state exists
        self.ensure_input_state(window, cx);

        let title = self.title.clone();
        let has_value = !self.value.is_empty();
        let view = cx.entity().clone();
        let input_state = self.input_state.clone().unwrap();

        // Icon: CircleX when has value (to clear), Search otherwise
        let trigger_icon = if has_value {
            IconName::CircleX
        } else {
            IconName::Search
        };

        let clear_view = view.clone();
        let trigger = Button::new("text-filter-trigger")
            .outline()
            .child(
                div()
                    .id("clear-icon")
                    .when(has_value, |this| {
                        this.cursor_pointer()
                            .rounded_sm()
                            .hover(|s| s.opacity(1.0))
                            .opacity(0.7)
                            .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                                clear_view.update(cx, |this, cx| {
                                    this.clear(window, cx);
                                });
                            })
                    })
                    .child(Icon::new(trigger_icon).xsmall()),
            )
            .child(title.clone())
            .when(has_value, |b| {
                b.child(Divider::vertical().h(px(16.)).mx_1())
                    .child(self.value.clone())
            });

        Popover::new("text-filter-popover")
            .trigger(trigger)
            .content(move |_, _window, _cx| {
                let apply_view = view.clone();
                v_flex()
                    .p_2()
                    .w_48()
                    .gap_2()
                    .child(Input::new(&input_state).cleanable(true).small())
                    .child(
                        Button::new("apply-btn")
                            .primary()
                            .small()
                            .w_full()
                            .label("Apply")
                            .on_click(move |_, window, cx| {
                                apply_view.update(cx, |this, cx| {
                                    this.apply(window, cx);
                                });
                            }),
                    )
            })
    }
}
