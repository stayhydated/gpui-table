use crate::TableFilterComponent;
use gpui::{App, Context, Entity, IntoElement, Render, Window, div, prelude::*, px};
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable,
    button::{Button, ButtonVariants},
    divider::Divider,
    h_flex,
    input::{Input, InputState},
    popover::Popover,
    v_flex,
};
use std::rc::Rc;

pub struct NumberRangeFilter {
    title: String,
    min: Option<f64>,
    max: Option<f64>,
    min_input: Option<Entity<InputState>>,
    max_input: Option<Entity<InputState>>,
    on_change: Rc<dyn Fn((Option<f64>, Option<f64>), &mut Window, &mut App) + 'static>,
}

impl TableFilterComponent for NumberRangeFilter {
    type Value = (Option<f64>, Option<f64>);

    const FILTER_TYPE: gpui_table_core::registry::RegistryFilterType =
        gpui_table_core::registry::RegistryFilterType::NumberRange;

    fn build(
        title: impl Into<String>,
        value: Self::Value,
        on_change: impl Fn(Self::Value, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();

        cx.new(|_cx| Self {
            title,
            min: value.0,
            max: value.1,
            min_input: None,
            max_input: None,
            on_change: Rc::new(on_change),
        })
    }
}

impl NumberRangeFilter {
    /// Legacy builder method for backwards compatibility.
    #[deprecated(note = "Use TableFilterComponent::build instead")]
    pub fn new(
        title: impl Into<String>,
        range: (Option<f64>, Option<f64>),
        on_change: impl Fn((Option<f64>, Option<f64>), &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        <Self as TableFilterComponent>::build(title, range, on_change, cx)
    }

    fn ensure_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.min_input.is_none() {
            let min_val = self.min.map(|v| format_number(v)).unwrap_or_default();
            let input = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("Min")
                    .default_value(min_val)
                    .clean_on_escape()
            });
            self.min_input = Some(input);
        }
        if self.max_input.is_none() {
            let max_val = self.max.map(|v| format_number(v)).unwrap_or_default();
            let input = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("Max")
                    .default_value(max_val)
                    .clean_on_escape()
            });
            self.max_input = Some(input);
        }
    }

    fn apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(min_input) = &self.min_input {
            let min_str = min_input.read(cx).value().to_string();
            self.min = min_str.parse::<f64>().ok();
        }
        if let Some(max_input) = &self.max_input {
            let max_str = max_input.read(cx).value().to_string();
            self.max = max_str.parse::<f64>().ok();
        }

        (self.on_change)((self.min, self.max), window, cx);
        cx.notify();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.min = None;
        self.max = None;
        if let Some(input) = &self.min_input {
            input.update(cx, |state, cx| {
                state.set_value("", window, cx);
            });
        }
        if let Some(input) = &self.max_input {
            input.update(cx, |state, cx| {
                state.set_value("", window, cx);
            });
        }
        (self.on_change)((None, None), window, cx);
        cx.notify();
    }

    fn has_value(&self) -> bool {
        self.min.is_some() || self.max.is_some()
    }

    fn format_range(&self) -> String {
        match (self.min, self.max) {
            (Some(min), Some(max)) => format!("{} - {}", format_number(min), format_number(max)),
            (Some(min), None) => format!(">= {}", format_number(min)),
            (None, Some(max)) => format!("<= {}", format_number(max)),
            (None, None) => String::new(),
        }
    }
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{:.0}", n)
    } else {
        format!("{}", n)
    }
}

impl Render for NumberRangeFilter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Ensure input states exist
        self.ensure_inputs(window, cx);

        let title = self.title.clone();
        let has_value = self.has_value();
        let range_display = self.format_range();
        let view = cx.entity().clone();
        let min_input = self.min_input.clone().unwrap();
        let max_input = self.max_input.clone().unwrap();

        // Icon: CircleX when has value (to clear), Plus otherwise
        let trigger_icon = if has_value {
            IconName::CircleX
        } else {
            IconName::Plus
        };

        let clear_view = view.clone();
        let trigger = Button::new("number-range-trigger")
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
                    .child(range_display)
            });

        Popover::new("number-range-popover")
            .trigger(trigger)
            .content(move |_, _window, cx| {
                let apply_view = view.clone();
                v_flex()
                    .p_3()
                    .gap_3()
                    .w_56()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(title.clone()),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(Input::new(&min_input).small().w_full())
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("to"),
                            )
                            .child(Input::new(&max_input).small().w_full()),
                    )
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
