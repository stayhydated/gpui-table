use crate::TableFilterComponent;
use gpui::{App, Context, Entity, IntoElement, Render, Subscription, Window, prelude::*, px};
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable,
    button::Button,
    divider::Divider,
    h_flex,
    input::{Input, InputState},
    popover::Popover,
    slider::{Slider, SliderEvent, SliderState},
    v_flex,
};
use std::rc::Rc;

pub struct NumberRangeFilter {
    title: String,
    min: Option<f64>,
    max: Option<f64>,
    range_min: f64,
    range_max: f64,
    min_input: Option<Entity<InputState>>,
    max_input: Option<Entity<InputState>>,
    slider_state: Option<Entity<SliderState>>,
    on_change: Rc<dyn Fn((Option<f64>, Option<f64>), &mut Window, &mut App) + 'static>,
    _subscriptions: Vec<Subscription>,
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
            range_min: 0.0,
            range_max: 100.0,
            min_input: None,
            max_input: None,
            slider_state: None,
            on_change: Rc::new(on_change),
            _subscriptions: Vec::new(),
        })
    }
}

impl NumberRangeFilter {
    /// Set the range bounds for the slider.
    pub fn set_range(entity: Entity<Self>, min: f64, max: f64, cx: &mut App) {
        entity.update(cx, |this, _cx| {
            this.range_min = min;
            this.range_max = max;
        });
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
        if self.slider_state.is_none() {
            let range_min = self.range_min;
            let range_max = self.range_max;
            let current_min = self.min.unwrap_or(range_min);
            let current_max = self.max.unwrap_or(range_max);

            let slider = cx.new(|_cx| {
                SliderState::new()
                    .min(range_min as f32)
                    .max(range_max as f32)
                    .step(1.0)
                    .default_value(current_min as f32..current_max as f32)
            });

            let subscription = cx.subscribe(
                &slider,
                move |this: &mut Self, _, event: &SliderEvent, cx| {
                    let SliderEvent::Change(value) = event;
                    let start = value.start() as f64;
                    let end = value.end() as f64;

                    this.min = Some(start);
                    this.max = Some(end);
                    cx.notify();
                },
            );

            self._subscriptions.push(subscription);
            self.slider_state = Some(slider);
        }
    }

    fn sync_inputs_from_slider(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let (Some(min_input), Some(max_input)) = (&self.min_input, &self.max_input) {
            if let Some(min) = self.min {
                min_input.update(cx, |state, cx| {
                    state.set_value(format_number(min), window, cx);
                });
            }
            if let Some(max) = self.max {
                max_input.update(cx, |state, cx| {
                    state.set_value(format_number(max), window, cx);
                });
            }
        }
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
        // Reset slider to full range
        if let Some(slider) = &self.slider_state {
            let range_min = self.range_min as f32;
            let range_max = self.range_max as f32;
            slider.update(cx, |state, cx| {
                state.set_value(range_min..range_max, window, cx);
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

        // Sync inputs from slider values (in case slider changed)
        self.sync_inputs_from_slider(window, cx);

        let title = self.title.clone();
        let has_value = self.has_value();
        let range_display = self.format_range();
        let view = cx.entity().clone();
        let min_input = self.min_input.clone().unwrap();
        let max_input = self.max_input.clone().unwrap();
        let slider_state = self.slider_state.clone().unwrap();

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
                gpui::div()
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
                let clear_view_inner = view.clone();
                v_flex()
                    .p_3()
                    .gap_3()
                    .w_64()
                    .child(
                        gpui::div()
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
                                gpui::div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("to"),
                            )
                            .child(Input::new(&max_input).small().w_full()),
                    )
                    .child(Slider::new(&slider_state))
                    .child(
                        Button::new("clear-btn")
                            .outline()
                            .small()
                            .w_full()
                            .label("Clear")
                            .on_click(move |_, window, cx| {
                                clear_view_inner.update(cx, |this, cx| {
                                    this.clear(window, cx);
                                });
                            }),
                    )
            })
    }
}
