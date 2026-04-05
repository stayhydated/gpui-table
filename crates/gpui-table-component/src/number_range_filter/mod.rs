use crate::TableFilterComponent;
use es_fluent::{EsFluent, ToFluentString as _};
use gpui::{
    App, Context, Entity, IntoElement, Render, StyleRefinement, Subscription, Task, Window,
    prelude::*, px,
};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _,
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

mod ext;
mod render;

pub use ext::NumberRangeFilterExt;

/// Debounce delay in milliseconds for filter changes
const DEBOUNCE_MS: u64 = 300;
/// Min number of characters used when sizing the localized "between" label.
const BETWEEN_MIN_CHARS: usize = 2;
/// Estimated width per character for the localized "between" label.
const BETWEEN_CHAR_WIDTH_PX: f32 = 10.0;
/// Extra horizontal padding applied to the "between" label width estimate.
const BETWEEN_BASE_PADDING_PX: f32 = 20.0;
/// Minimum width for the "between" label container.
const BETWEEN_MIN_WIDTH_PX: f32 = 32.0;
/// Base width for each NumberInput before locale-driven expansion.
const INPUT_BASE_WIDTH_PX: f32 = 108.0;
/// Extra expansion multiplier applied to locale-driven width growth.
const INPUT_EXPANSION_FACTOR: f32 = 1.15;
/// Min number of placeholder chars used for width heuristics.
const PLACEHOLDER_MIN_CHARS: usize = 3;
/// Estimated width per placeholder character.
const PLACEHOLDER_CHAR_WIDTH_PX: f32 = 7.5;
/// Baseline width budget for NumberInput chrome (buttons, paddings, etc.).
const PLACEHOLDER_BASE_WIDTH_PX: f32 = 72.0;
/// Total horizontal gap used in the min-max row (two `gap_2` slots).
const ROW_GAP_TOTAL_PX: f32 = 16.0;
/// Total horizontal padding from `v_flex().p_3()` (left + right).
const POPOVER_HORIZONTAL_PADDING_PX: f32 = 24.0;
/// Number of inputs in the min-max row.
const ROW_INPUT_COUNT: f32 = 2.0;
/// Default slider minimum when decimal conversion fails.
const DEFAULT_RANGE_MIN_F32: f32 = 0.0;
/// Default slider maximum when decimal conversion fails.
const DEFAULT_RANGE_MAX_F32: f32 = 100.0;
/// Slider step size.
const DEFAULT_SLIDER_STEP_F32: f32 = 1.0;

/// Tracks which component changed last to determine sync direction
#[derive(Clone, Copy, PartialEq)]
enum LastChanged {
    None,
    Slider,
    MinInput,
    MaxInput,
}

#[derive(Clone, Copy, EsFluent)]
enum NumberRangeFilterFtl {
    MinPlaceholder,
    MaxPlaceholder,
    Between,
}

pub struct NumberRangeFilter {
    title: Rc<dyn Fn() -> String>,
    min: Option<Decimal>,
    max: Option<Decimal>,
    range_min: Decimal,
    range_max: Decimal,
    range_is_explicit: bool,
    step_size: Option<Decimal>,
    trigger_style: StyleRefinement,
    popover_style: StyleRefinement,
    inputs_row_style: StyleRefinement,
    min_input_style: StyleRefinement,
    max_input_style: StyleRefinement,
    between_style: StyleRefinement,
    slider_style: StyleRefinement,
    clear_button_style: StyleRefinement,
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
    /// Last placeholder applied to the min input.
    last_min_placeholder: Option<String>,
    /// Last placeholder applied to the max input.
    last_max_placeholder: Option<String>,
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
        let mut range_min = Decimal::ZERO;
        let mut range_max = Decimal::ONE_HUNDRED;
        if let Some(min) = value.0 {
            if min < range_min {
                range_min = min;
            }
            if min > range_max {
                range_max = min;
            }
        }
        if let Some(max) = value.1 {
            if max < range_min {
                range_min = max;
            }
            if max > range_max {
                range_max = max;
            }
        }

        cx.new(|_cx| Self {
            title,
            min: value.0,
            max: value.1,
            range_min,
            range_max,
            range_is_explicit: false,
            step_size: None,
            trigger_style: StyleRefinement::default(),
            popover_style: StyleRefinement::default(),
            inputs_row_style: StyleRefinement::default(),
            min_input_style: StyleRefinement::default(),
            max_input_style: StyleRefinement::default(),
            between_style: StyleRefinement::default(),
            slider_style: StyleRefinement::default(),
            clear_button_style: StyleRefinement::default(),
            min_input: None,
            max_input: None,
            slider_state: None,
            on_change: Rc::new(on_change),
            _subscriptions: Vec::new(),
            pending_apply: false,
            _debounce_task: None,
            last_changed: LastChanged::None,
            last_min_placeholder: None,
            last_max_placeholder: None,
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

    fn min_placeholder_text() -> String {
        NumberRangeFilterFtl::MinPlaceholder.to_fluent_string()
    }

    fn max_placeholder_text() -> String {
        NumberRangeFilterFtl::MaxPlaceholder.to_fluent_string()
    }

    fn between_text() -> String {
        NumberRangeFilterFtl::Between.to_fluent_string()
    }

    fn between_width_px(between: &str) -> f32 {
        // Leave generous room for longer localized joiners (e.g. some Romance languages).
        let char_count = between.trim().chars().count().max(BETWEEN_MIN_CHARS) as f32;
        (char_count * BETWEEN_CHAR_WIDTH_PX + BETWEEN_BASE_PADDING_PX).max(BETWEEN_MIN_WIDTH_PX)
    }

    fn input_width_px(between_width_px: f32, min_placeholder: &str, max_placeholder: &str) -> f32 {
        // Base input width is tuned for the original "to" layout.
        // Expand each input as localized joiner/placeholder strings get longer.
        let between_delta = (between_width_px - BETWEEN_MIN_WIDTH_PX).max(0.0);
        let placeholder_chars = min_placeholder
            .trim()
            .chars()
            .count()
            .max(max_placeholder.trim().chars().count())
            .max(PLACEHOLDER_MIN_CHARS) as f32;
        let between_target = INPUT_BASE_WIDTH_PX + between_delta * INPUT_EXPANSION_FACTOR;
        let placeholder_target = placeholder_chars
            * (PLACEHOLDER_CHAR_WIDTH_PX * INPUT_EXPANSION_FACTOR)
            + PLACEHOLDER_BASE_WIDTH_PX;

        between_target.max(placeholder_target)
    }

    fn row_width_px(input_width_px: f32, between_width_px: f32) -> f32 {
        // h_flex().gap_2() creates two gaps between the three row children.
        input_width_px * ROW_INPUT_COUNT + between_width_px + ROW_GAP_TOTAL_PX
    }

    fn popover_width_px(row_width_px: f32) -> f32 {
        // v_flex().p_3() contributes 12px padding on left and right.
        row_width_px + POPOVER_HORIZONTAL_PADDING_PX
    }

    fn recompute_dynamic_range_from_values(&mut self) {
        if self.range_is_explicit {
            return;
        }

        let mut range_min = Decimal::ZERO;
        let mut range_max = Decimal::ONE_HUNDRED;
        if let Some(min) = self.min {
            if min < range_min {
                range_min = min;
            }
            if min > range_max {
                range_max = min;
            }
        }
        if let Some(max) = self.max {
            if max < range_min {
                range_min = max;
            }
            if max > range_max {
                range_max = max;
            }
        }

        self.range_min = range_min;
        self.range_max = range_max;
    }

    fn slider_values(&self) -> (f32, f32, f32, f32) {
        let range_min = self.range_min.to_f32().unwrap_or(DEFAULT_RANGE_MIN_F32);
        let range_max = self.range_max.to_f32().unwrap_or(DEFAULT_RANGE_MAX_F32);
        let current_min = self
            .min
            .and_then(|d| d.to_f32())
            .unwrap_or(range_min)
            .clamp(range_min, range_max);
        let current_max = self
            .max
            .and_then(|d| d.to_f32())
            .unwrap_or(range_max)
            .clamp(range_min, range_max);

        (range_min, range_max, current_min, current_max)
    }

    fn ensure_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.min_input.is_none() {
            let min_val = self.min.map(format_decimal).unwrap_or_default();
            let min_placeholder = Self::min_placeholder_text();
            let initial_min_placeholder = min_placeholder.clone();

            let input = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(initial_min_placeholder)
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
                            this.min = Some(val);
                            if this.range_is_explicit {
                                this.min = Some(val.clamp(this.range_min, this.range_max));
                            } else {
                                this.recompute_dynamic_range_from_values();
                            }
                        } else if text.is_empty() {
                            this.min = None;
                            this.recompute_dynamic_range_from_values();
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
                    let mut new_val = match action {
                        StepAction::Increment => current + step,
                        StepAction::Decrement => current - step,
                    };
                    if this.range_is_explicit {
                        new_val = new_val.clamp(this.range_min, this.range_max);
                    }
                    this.min = Some(new_val);
                    this.recompute_dynamic_range_from_values();
                    this.last_changed = LastChanged::MinInput;
                    this.schedule_debounced_apply(cx);
                },
            );

            self._subscriptions.push(sub1);
            self._subscriptions.push(sub2);
            self.min_input = Some(input);
            self.last_min_placeholder = Some(min_placeholder);
        }

        if self.max_input.is_none() {
            let max_val = self.max.map(format_decimal).unwrap_or_default();
            let max_placeholder = Self::max_placeholder_text();
            let initial_max_placeholder = max_placeholder.clone();

            let input = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(initial_max_placeholder)
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
                            this.max = Some(val);
                            if this.range_is_explicit {
                                this.max = Some(val.clamp(this.range_min, this.range_max));
                            } else {
                                this.recompute_dynamic_range_from_values();
                            }
                        } else if text.is_empty() {
                            this.max = None;
                            this.recompute_dynamic_range_from_values();
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
                    let mut new_val = match action {
                        StepAction::Increment => current + step,
                        StepAction::Decrement => current - step,
                    };
                    if this.range_is_explicit {
                        new_val = new_val.clamp(this.range_min, this.range_max);
                    }
                    this.max = Some(new_val);
                    this.recompute_dynamic_range_from_values();
                    this.last_changed = LastChanged::MaxInput;
                    this.schedule_debounced_apply(cx);
                },
            );

            self._subscriptions.push(sub1);
            self._subscriptions.push(sub2);
            self.max_input = Some(input);
            self.last_max_placeholder = Some(max_placeholder);
        }

        if self.slider_state.is_none() {
            let (range_min, range_max, current_min, current_max) = self.slider_values();

            let slider = cx.new(|_cx| {
                SliderState::new()
                    .min(range_min)
                    .max(range_max)
                    .step(DEFAULT_SLIDER_STEP_F32)
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
                    let (range_min, range_max, current_min, current_max) = self.slider_values();
                    slider.update(cx, |state, cx| {
                        *state = SliderState::new()
                            .min(range_min)
                            .max(range_max)
                            .step(DEFAULT_SLIDER_STEP_F32)
                            .default_value(current_min..current_max);
                        cx.notify();
                    });
                }
            },
            LastChanged::None => {},
        }
        self.last_changed = LastChanged::None;
    }

    fn reset_inner(&mut self, notify_change: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.min = None;
        self.max = None;
        self.recompute_dynamic_range_from_values();
        self.pending_apply = false;
        self._debounce_task = None;

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
            let (range_min, range_max, current_min, current_max) = self.slider_values();
            slider.update(cx, |state, cx| {
                *state = SliderState::new()
                    .min(range_min)
                    .max(range_max)
                    .step(DEFAULT_SLIDER_STEP_F32)
                    .default_value(current_min..current_max);
                cx.notify();
            });
        }

        if notify_change {
            // Reset applies immediately (no debounce for clear action)
            (self.on_change)((None, None), window, cx);
        }

        self.last_changed = LastChanged::None;
        cx.notify();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset_inner(true, window, cx);
    }

    fn has_value(&self) -> bool {
        self.min.is_some() || self.max.is_some()
    }

    /// Apply the current filter value via callback.
    /// Call this from parent when you want to trigger the on_change.
    pub fn apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        (self.on_change)((self.min, self.max), window, cx);
    }

    /// Reset the range value and notify via callback.
    pub fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset_inner(true, window, cx);
    }

    /// Reset the range value without invoking callback.
    pub fn reset_silent(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset_inner(false, window, cx);
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
