use crate::TableFilterComponent;
use gpui::{App, Context, Entity, IntoElement, Render, Subscription, Task, Window, prelude::*, px};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, Sizable as _,
    button::Button,
    divider::Divider,
    h_flex,
    input::{InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
    popover::Popover,
    slider::{Slider, SliderEvent, SliderState},
    v_flex,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use std::rc::Rc;
use std::time::Duration;

/// Debounce delay in milliseconds for filter changes
const DEBOUNCE_MS: u64 = 300;

/// Tracks which component changed last to determine sync direction
#[derive(Clone, Copy, PartialEq)]
enum LastChanged {
    None,
    Slider,
    MinInput,
    MaxInput,
}

pub struct NumberRangeFilter {
    title: Rc<dyn Fn() -> String>,
    min: Option<Decimal>,
    max: Option<Decimal>,
    range_min: Decimal,
    range_max: Decimal,
    step_size: Option<Decimal>,
    min_input: Option<Entity<InputState>>,
    max_input: Option<Entity<InputState>>,
    slider_state: Option<Entity<SliderState>>,
    on_change: Rc<dyn Fn((Option<Decimal>, Option<Decimal>), &mut Window, &mut App) + 'static>,
    _subscriptions: Vec<Subscription>,
    /// Flag set by debounce task to trigger apply during next render
    pending_apply: bool,
    /// Current debounce task - dropping it cancels the pending apply
    _debounce_task: Option<Task<()>>,
    /// Tracks which component was last changed for sync direction
    last_changed: LastChanged,
}

impl TableFilterComponent for NumberRangeFilter {
    type Value = (Option<Decimal>, Option<Decimal>);

    const FILTER_TYPE: gpui_table_core::registry::RegistryFilterType =
        gpui_table_core::registry::RegistryFilterType::NumberRange;

    fn new(
        title: impl Into<String>,
        value: Self::Value,
        on_change: impl Fn(Self::Value, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();
        Self::new_with_title(Rc::new(move || title.clone()), value, on_change, cx)
    }
}

impl NumberRangeFilter {
    fn new_with_title(
        title: Rc<dyn Fn() -> String>,
        value: (Option<Decimal>, Option<Decimal>),
        on_change: impl Fn((Option<Decimal>, Option<Decimal>), &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|_cx| Self {
            title,
            min: value.0,
            max: value.1,
            range_min: Decimal::ZERO,
            range_max: Decimal::ONE_HUNDRED,
            step_size: None,
            min_input: None,
            max_input: None,
            slider_state: None,
            on_change: Rc::new(on_change),
            _subscriptions: Vec::new(),
            pending_apply: false,
            _debounce_task: None,
            last_changed: LastChanged::None,
        })
    }

    /// Create a number range filter with a reactive title provider (e.g. for i18n).
    pub fn new_for(
        title: impl Fn() -> String + 'static,
        value: (Option<Decimal>, Option<Decimal>),
        on_change: impl Fn((Option<Decimal>, Option<Decimal>), &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        Self::new_with_title(Rc::new(title), value, on_change, cx)
    }

    fn ensure_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.min_input.is_none() {
            let min_val = self.min.map(format_decimal).unwrap_or_default();
            let range_min = self.range_min;
            let range_max = self.range_max;

            let input = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("Min")
                    .default_value(min_val)
                    .clean_on_escape()
            });

            // Subscribe to input text changes
            let sub1 = cx.subscribe(
                &input,
                move |this: &mut Self, state, event: &InputEvent, cx| {
                    if let InputEvent::Change = event {
                        let text = state.read(cx).value().to_string();
                        if let Ok(val) = Decimal::from_str(&text) {
                            let clamped = val.clamp(range_min, range_max);
                            this.min = Some(clamped);
                        } else if text.is_empty() {
                            this.min = None;
                        }
                        this.last_changed = LastChanged::MinInput;
                        this.schedule_debounced_apply(cx);
                    }
                },
            );

            // Subscribe to step actions
            let sub2 = cx.subscribe(
                &input,
                move |this: &mut Self, _state, event: &NumberInputEvent, cx| {
                    let NumberInputEvent::Step(action) = event;
                    let current = this.min.unwrap_or(this.range_min);
                    let step = this
                        .step_size
                        .unwrap_or((this.range_max - this.range_min) / Decimal::ONE_HUNDRED);
                    let new_val = match action {
                        StepAction::Increment => (current + step).min(this.range_max),
                        StepAction::Decrement => (current - step).max(this.range_min),
                    };
                    this.min = Some(new_val);
                    this.last_changed = LastChanged::MinInput;
                    this.schedule_debounced_apply(cx);
                },
            );

            self._subscriptions.push(sub1);
            self._subscriptions.push(sub2);
            self.min_input = Some(input);
        }

        if self.max_input.is_none() {
            let max_val = self.max.map(format_decimal).unwrap_or_default();
            let range_min = self.range_min;
            let range_max = self.range_max;

            let input = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("Max")
                    .default_value(max_val)
                    .clean_on_escape()
            });

            // Subscribe to input text changes
            let sub1 = cx.subscribe(
                &input,
                move |this: &mut Self, state, event: &InputEvent, cx| {
                    if let InputEvent::Change = event {
                        let text = state.read(cx).value().to_string();
                        if let Ok(val) = Decimal::from_str(&text) {
                            let clamped = val.clamp(range_min, range_max);
                            this.max = Some(clamped);
                        } else if text.is_empty() {
                            this.max = None;
                        }
                        this.last_changed = LastChanged::MaxInput;
                        this.schedule_debounced_apply(cx);
                    }
                },
            );

            // Subscribe to step actions
            let sub2 = cx.subscribe(
                &input,
                move |this: &mut Self, _state, event: &NumberInputEvent, cx| {
                    let NumberInputEvent::Step(action) = event;
                    let current = this.max.unwrap_or(this.range_max);
                    let step = this
                        .step_size
                        .unwrap_or((this.range_max - this.range_min) / Decimal::ONE_HUNDRED);
                    let new_val = match action {
                        StepAction::Increment => (current + step).min(this.range_max),
                        StepAction::Decrement => (current - step).max(this.range_min),
                    };
                    this.max = Some(new_val);
                    this.last_changed = LastChanged::MaxInput;
                    this.schedule_debounced_apply(cx);
                },
            );

            self._subscriptions.push(sub1);
            self._subscriptions.push(sub2);
            self.max_input = Some(input);
        }

        if self.slider_state.is_none() {
            let range_min = self.range_min.to_f32().unwrap_or(0.0);
            let range_max = self.range_max.to_f32().unwrap_or(100.0);
            let current_min = self.min.and_then(|d| d.to_f32()).unwrap_or(range_min);
            let current_max = self.max.and_then(|d| d.to_f32()).unwrap_or(range_max);

            let slider = cx.new(|_cx| {
                SliderState::new()
                    .min(range_min)
                    .max(range_max)
                    .step(1.0)
                    .default_value(current_min..current_max)
            });

            // Subscribe to slider changes
            let subscription = cx.subscribe(
                &slider,
                move |this: &mut Self, _, event: &SliderEvent, cx| {
                    let SliderEvent::Change(value) = event;
                    let start = Decimal::from_f32(value.start()).unwrap_or(Decimal::ZERO);
                    let end = Decimal::from_f32(value.end()).unwrap_or(Decimal::ONE_HUNDRED);

                    this.min = Some(start);
                    this.max = Some(end);
                    this.last_changed = LastChanged::Slider;
                    this.schedule_debounced_apply(cx);
                },
            );

            self._subscriptions.push(subscription);
            self.slider_state = Some(slider);
        }
    }

    fn schedule_debounced_apply(&mut self, cx: &mut Context<Self>) {
        // Cancel any pending debounce task and schedule a new one
        self._debounce_task = Some(cx.spawn(async move |view, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(DEBOUNCE_MS))
                .await;
            view.update(cx, |this, cx| {
                this.pending_apply = true;
                this._debounce_task = None;
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }

    fn sync_components(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.last_changed {
            LastChanged::Slider => {
                // Slider changed - update input values
                if let Some(min_input) = &self.min_input
                    && let Some(min) = self.min
                {
                    min_input.update(cx, |state, cx| {
                        state.set_value(format_decimal(min), window, cx);
                    });
                }

                if let Some(max_input) = &self.max_input
                    && let Some(max) = self.max
                {
                    max_input.update(cx, |state, cx| {
                        state.set_value(format_decimal(max), window, cx);
                    });
                }
            },
            LastChanged::MinInput | LastChanged::MaxInput => {
                // Input changed - update slider
                if let Some(slider) = &self.slider_state {
                    let min = self
                        .min
                        .and_then(|d| d.to_f32())
                        .unwrap_or(self.range_min.to_f32().unwrap_or(0.0));
                    let max = self
                        .max
                        .and_then(|d| d.to_f32())
                        .unwrap_or(self.range_max.to_f32().unwrap_or(100.0));
                    slider.update(cx, |state, cx| {
                        state.set_value(min..max, window, cx);
                    });
                }
            },
            LastChanged::None => {},
        }
        self.last_changed = LastChanged::None;
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
            let range_min = self.range_min.to_f32().unwrap_or(0.0);
            let range_max = self.range_max.to_f32().unwrap_or(100.0);
            slider.update(cx, |state, cx| {
                state.set_value(range_min..range_max, window, cx);
            });
        }
        // Clear applies immediately (no debounce for clear action)
        (self.on_change)((None, None), window, cx);
        self.last_changed = LastChanged::None;
        cx.notify();
    }

    fn has_value(&self) -> bool {
        self.min.is_some() || self.max.is_some()
    }

    /// Apply the current filter value via callback.
    /// Call this from parent when you want to trigger the on_change.
    pub fn apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        (self.on_change)((self.min, self.max), window, cx);
    }

    /// Get the current filter value.
    pub fn value(&self) -> (Option<Decimal>, Option<Decimal>) {
        (self.min, self.max)
    }

    fn format_range(&self) -> String {
        match (self.min, self.max) {
            (Some(min), Some(max)) => {
                format!("{} - {}", format_decimal(min), format_decimal(max))
            },
            (Some(min), None) => format!(">= {}", format_decimal(min)),
            (None, Some(max)) => format!("<= {}", format_decimal(max)),
            (None, None) => String::new(),
        }
    }
}

fn format_decimal(d: Decimal) -> String {
    // Normalize to remove trailing zeros, then format
    let normalized = d.normalize();
    if normalized.fract().is_zero() {
        format!("{:.0}", normalized)
    } else {
        normalized.to_string()
    }
}

impl Render for NumberRangeFilter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Ensure input states exist
        self.ensure_inputs(window, cx);

        // Sync components based on what changed last
        self.sync_components(window, cx);

        // Apply pending changes now that we have window access
        if self.pending_apply {
            self.pending_apply = false;
            (self.on_change)((self.min, self.max), window, cx);
        }

        let title = (self.title)();
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
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(NumberInput::new(&min_input).small().w_full())
                            .child(
                                gpui::div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("to"),
                            )
                            .child(NumberInput::new(&max_input).small().w_full()),
                    )
                    .child(Slider::new(&slider_state))
                    .when(has_value, |this| {
                        this.child(
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
            })
    }
}

/// Extension trait for chainable configuration on Entity<NumberRangeFilter>
pub trait NumberRangeFilterExt {
    /// Set the range bounds for the slider (chainable).
    ///
    /// # Example
    /// ```ignore
    /// NumberRangeFilter::build("Price", (None, None), on_change, cx)
    ///     .range(Decimal::ZERO, Decimal::new(1000, 0), cx)
    ///     .step(Decimal::TEN, cx)
    /// ```
    fn range(self, min: Decimal, max: Decimal, cx: &mut App) -> Self;

    /// Set the step size for increment/decrement (chainable).
    /// Default is 1% of the range.
    fn step(self, step: Decimal, cx: &mut App) -> Self;
}

impl NumberRangeFilterExt for Entity<NumberRangeFilter> {
    fn range(self, min: Decimal, max: Decimal, cx: &mut App) -> Self {
        self.update(cx, |this, _cx| {
            this.range_min = min;
            this.range_max = max;
        });
        self
    }

    fn step(self, step: Decimal, cx: &mut App) -> Self {
        self.update(cx, |this, _cx| {
            this.step_size = Some(step);
        });
        self
    }
}
