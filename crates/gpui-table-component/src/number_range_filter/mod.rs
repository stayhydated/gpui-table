use crate::TableFilterComponent;
use es_fluent::EsFluent;
use gpui::{
    App, Context, Entity, IntoElement, Render, StyleRefinement, Subscription, Task, Window,
    prelude::*, px,
};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _,
    button::Button,
    h_flex,
    input::{InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
    popover::Popover,
    separator::Separator,
    slider::{Slider, SliderEvent, SliderState},
    v_flex,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use std::borrow::Borrow;
use std::rc::Rc;
use std::time::Duration;

mod ext;
mod model;
mod render;

use model::{BoundTextChange, StepDirection};

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

#[derive(Clone, Copy)]
enum BoundInput {
    Min,
    Max,
}

#[derive(Clone, Copy, EsFluent)]
enum NumberRangeFilterFtl {
    MinPlaceholder,
    MaxPlaceholder,
    Between,
}

pub struct NumberRangeFilter {
    title: Rc<dyn Fn(&App) -> String>,
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

impl component_shape::ComponentShapeMetadata for NumberRangeFilter {
    const MCP_INPUT: component_shape::McpInput = component_shape::McpInput::decimal_range();
}
impl component_shape::DeclaredComponentShape for NumberRangeFilter {}

macro_rules! impl_number_range_component_shape_for {
    ($($ty:ty),* $(,)?) => {
        $(
            impl component_shape::ComponentShapeFor<$ty> for NumberRangeFilter {}
            impl component_shape::ComponentShapeFor<Option<$ty>> for NumberRangeFilter {}
        )*
    };
}

impl_number_range_component_shape_for!(
    f32, f64, i8, i16, i32, i64, isize, Decimal, u8, u16, u32, u64, usize,
);

#[cfg(feature = "spacetimedb")]
impl_number_range_component_shape_for!(spacetimedb_lib::Timestamp, spacetimedb_lib::TimeDuration);

impl TableFilterComponent for NumberRangeFilter {
    type Value = (Option<Decimal>, Option<Decimal>);

    const FILTER_TYPE: gpui_table_schema::registry::RegistryFilterType =
        gpui_table_schema::registry::RegistryFilterType::NumberRange;

    fn new(
        title: impl Into<String>,
        value: Self::Value,
        on_change: impl Fn(Self::Value, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();
        Self::new_with_title(Rc::new(move |_| title.clone()), value, on_change, cx)
    }
}

impl NumberRangeFilter {
    /// Create a number range filter with a fixed title.
    pub fn new(
        title: impl Into<String>,
        value: (Option<Decimal>, Option<Decimal>),
        on_change: impl Fn((Option<Decimal>, Option<Decimal>), &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();
        Self::new_with_title(Rc::new(move |_| title.clone()), value, on_change, cx)
    }

    fn new_with_title(
        title: Rc<dyn Fn(&App) -> String>,
        value: (Option<Decimal>, Option<Decimal>),
        on_change: impl Fn((Option<Decimal>, Option<Decimal>), &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let (range_min, range_max) = model::dynamic_range(value.0, value.1);

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
        title: impl Fn(&App) -> String + 'static,
        value: (Option<Decimal>, Option<Decimal>),
        on_change: impl Fn((Option<Decimal>, Option<Decimal>), &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        Self::new_with_title(Rc::new(title), value, on_change, cx)
    }

    fn min_placeholder_text(cx: &impl Borrow<App>) -> String {
        crate::i18n::localize_message(cx, &NumberRangeFilterFtl::MinPlaceholder)
    }

    fn max_placeholder_text(cx: &impl Borrow<App>) -> String {
        crate::i18n::localize_message(cx, &NumberRangeFilterFtl::MaxPlaceholder)
    }

    fn between_text(cx: &impl Borrow<App>) -> String {
        crate::i18n::localize_message(cx, &NumberRangeFilterFtl::Between)
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

        (self.range_min, self.range_max) = model::dynamic_range(self.min, self.max);
    }

    fn slider_values(&self) -> (f32, f32, f32, f32) {
        model::slider_values(self.min, self.max, self.range_min, self.range_max)
    }

    fn bound_value(&self, bound: BoundInput) -> Option<Decimal> {
        match bound {
            BoundInput::Min => self.min,
            BoundInput::Max => self.max,
        }
    }

    fn set_bound_value(&mut self, bound: BoundInput, value: Option<Decimal>) {
        match bound {
            BoundInput::Min => self.min = value,
            BoundInput::Max => self.max = value,
        }
    }

    fn last_changed_for(bound: BoundInput) -> LastChanged {
        match bound {
            BoundInput::Min => LastChanged::MinInput,
            BoundInput::Max => LastChanged::MaxInput,
        }
    }

    fn update_bound_from_text(&mut self, bound: BoundInput, text: &str, cx: &mut Context<Self>) {
        match model::bound_text_change(text, self.range_is_explicit, self.range_min, self.range_max)
        {
            BoundTextChange::Set(value) => self.set_bound_value(bound, Some(value)),
            BoundTextChange::Clear => self.set_bound_value(bound, None),
            BoundTextChange::Unchanged => {},
        }
        if !self.range_is_explicit {
            self.recompute_dynamic_range_from_values();
        }

        self.last_changed = Self::last_changed_for(bound);
        self.schedule_debounced_apply(cx);
    }

    fn step_amount(&self) -> Decimal {
        model::step_amount(self.step_size, self.range_min, self.range_max)
    }

    fn step_bound(&mut self, bound: BoundInput, action: &StepAction, cx: &mut Context<Self>) {
        let fallback = match bound {
            BoundInput::Min => self.range_min,
            BoundInput::Max => self.range_max,
        };
        let direction = match action {
            StepAction::Increment => StepDirection::Increment,
            StepAction::Decrement => StepDirection::Decrement,
        };
        let explicit_range = self
            .range_is_explicit
            .then_some((self.range_min, self.range_max));
        let next = model::stepped_value(
            self.bound_value(bound),
            fallback,
            self.step_amount(),
            direction,
            explicit_range,
        );

        self.set_bound_value(bound, Some(next));
        self.recompute_dynamic_range_from_values();
        self.last_changed = Self::last_changed_for(bound);
        self.schedule_debounced_apply(cx);
    }

    fn sync_slider_state(&self, slider: &Entity<SliderState>, cx: &mut Context<Self>) {
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

    fn ensure_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.min_input.is_none() {
            let min_val = self.min.map(format_decimal).unwrap_or_default();
            let min_placeholder = Self::min_placeholder_text(cx);
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
                        this.update_bound_from_text(BoundInput::Min, &text, cx);
                    }
                },
            );

            // Subscribe to step actions
            let sub2 = cx.subscribe(
                &input,
                move |this: &mut Self, _state, event: &NumberInputEvent, cx| {
                    let NumberInputEvent::Step(action) = event;
                    this.step_bound(BoundInput::Min, action, cx);
                },
            );

            self._subscriptions.push(sub1);
            self._subscriptions.push(sub2);
            self.min_input = Some(input);
            self.last_min_placeholder = Some(min_placeholder);
        }

        if self.max_input.is_none() {
            let max_val = self.max.map(format_decimal).unwrap_or_default();
            let max_placeholder = Self::max_placeholder_text(cx);
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
                        this.update_bound_from_text(BoundInput::Max, &text, cx);
                    }
                },
            );

            // Subscribe to step actions
            let sub2 = cx.subscribe(
                &input,
                move |this: &mut Self, _state, event: &NumberInputEvent, cx| {
                    let NumberInputEvent::Step(action) = event;
                    this.step_bound(BoundInput::Max, action, cx);
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
                move |this: &mut Self, _, event: &SliderEvent, cx| match event {
                    SliderEvent::Change(value) => {
                        let start = Decimal::from_f32(value.start()).unwrap_or(Decimal::ZERO);
                        let end = Decimal::from_f32(value.end()).unwrap_or(Decimal::ONE_HUNDRED);

                        this.min = Some(start);
                        this.max = Some(end);
                        this.last_changed = LastChanged::Slider;
                        this.schedule_debounced_apply(cx);
                    },
                    SliderEvent::Release(_) => {},
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
                    self.sync_slider_state(slider, cx);
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
            self.sync_slider_state(slider, cx);
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

    /// Replace range bounds without invoking the change callback.
    pub fn set_silent(
        &mut self,
        value: (Option<Decimal>, Option<Decimal>),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.min = value.0;
        self.max = value.1;
        self.recompute_dynamic_range_from_values();
        self.pending_apply = false;
        self._debounce_task = None;
        self.last_changed = LastChanged::Slider;
        self.sync_components(window, cx);
        cx.notify();
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

#[cfg(test)]
mod tests {
    use super::{
        BoundInput, LastChanged, NumberRangeFilter, NumberRangeFilterExt as _, format_decimal,
    };
    use gpui::{Empty, StyleRefinement, TestAppContext, VisualTestContext};
    use rust_decimal::Decimal;
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn decimal_and_layout_helpers_are_stable_for_localized_content() {
        assert_eq!(format_decimal(Decimal::new(1000, 2)), "10");
        assert_eq!(format_decimal(Decimal::new(1234, 2)), "12.34");

        assert_eq!(NumberRangeFilter::between_width_px("to"), 40.0);
        assert!(NumberRangeFilter::between_width_px("through") > 40.0);

        let input_width = NumberRangeFilter::input_width_px(40.0, "Min", "Maximum");
        assert!(input_width >= 132.0);
        let row_width = NumberRangeFilter::row_width_px(input_width, 40.0);
        assert_eq!(row_width, input_width * 2.0 + 56.0);
        assert_eq!(
            NumberRangeFilter::popover_width_px(row_width),
            row_width + 24.0
        );
    }

    #[gpui::test]
    fn constructors_ranges_steps_and_styles_preserve_numeric_state(cx: &mut TestAppContext) {
        let filter = cx.update(|cx| {
            NumberRangeFilter::new(
                "Amount",
                (Some(Decimal::from(-5)), Some(Decimal::from(150))),
                |_, _, _| {},
                cx,
            )
            .range(Decimal::from(-10), Decimal::from(10), cx)
            .step(Decimal::new(5, 1), cx)
            .trigger_style(StyleRefinement::default(), cx)
            .popover_style(StyleRefinement::default(), cx)
            .inputs_row_style(StyleRefinement::default(), cx)
            .min_input_style(StyleRefinement::default(), cx)
            .max_input_style(StyleRefinement::default(), cx)
            .between_style(StyleRefinement::default(), cx)
            .slider_style(StyleRefinement::default(), cx)
            .clear_button_style(StyleRefinement::default(), cx)
        });

        filter.read_with(cx, |filter, cx| {
            assert_eq!((filter.title)(cx), "Amount");
            assert_eq!(
                filter.value(),
                (Some(Decimal::from(-5)), Some(Decimal::from(10)))
            );
            assert_eq!(filter.range_min, Decimal::from(-10));
            assert_eq!(filter.range_max, Decimal::from(10));
            assert!(filter.range_is_explicit);
            assert_eq!(filter.step_amount(), Decimal::new(5, 1));
            assert!(filter.has_value());
            assert_eq!(filter.format_range(), "-5 - 10");
            assert_eq!(filter.slider_values(), (-10.0, 10.0, -5.0, 10.0));
            assert!(filter.min_input.is_none());
            assert!(filter.slider_state.is_none());
        });

        filter.update(cx, |filter, _| {
            assert_eq!(filter.bound_value(BoundInput::Min), Some(Decimal::from(-5)));
            assert_eq!(filter.bound_value(BoundInput::Max), Some(Decimal::from(10)));
            filter.set_bound_value(BoundInput::Min, None);
            filter.set_bound_value(BoundInput::Max, Some(Decimal::from(5)));
            assert!(matches!(
                NumberRangeFilter::last_changed_for(BoundInput::Min),
                LastChanged::MinInput
            ));
            assert!(matches!(
                NumberRangeFilter::last_changed_for(BoundInput::Max),
                LastChanged::MaxInput
            ));
        });
        filter.read_with(cx, |filter, _| assert_eq!(filter.format_range(), "<= 5"));

        let dynamic = cx.update(|cx| {
            NumberRangeFilter::new_for(
                |_| "Dynamic".into(),
                (Some(Decimal::from(-20)), Some(Decimal::from(200))),
                |_, _, _| {},
                cx,
            )
        });
        dynamic.update(cx, |filter, _| {
            assert_eq!(filter.range_min, Decimal::from(-20));
            assert_eq!(filter.range_max, Decimal::from(200));
            assert_eq!(filter.step_amount(), Decimal::new(22, 1));

            filter.min = Some(Decimal::from(20));
            filter.max = None;
            filter.recompute_dynamic_range_from_values();
            assert_eq!(filter.range_min, Decimal::ZERO);
            assert_eq!(filter.range_max, Decimal::ONE_HUNDRED);
            assert_eq!(filter.format_range(), ">= 20");

            filter.min = None;
            assert_eq!(filter.format_range(), "");
            assert!(!filter.has_value());
        });
    }

    #[gpui::test]
    fn number_filter_apply_and_reset_paths_use_window_context(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let changes = Rc::new(RefCell::new(Vec::new()));
        let changes_for_callback = changes.clone();
        let filter = cx.update(|cx| {
            NumberRangeFilter::new(
                "Amount",
                (Some(Decimal::from(10)), Some(Decimal::from(20))),
                move |value, _, _| changes_for_callback.borrow_mut().push(value),
                cx,
            )
        });
        let window = cx.add_window(|_, _| Empty);
        let mut visual = VisualTestContext::from_window(window.into(), cx);

        visual.update(|window, cx| {
            filter.update(cx, |filter, cx| {
                filter.apply(window, cx);
                filter.reset(window, cx);
            });
        });
        assert_eq!(changes.borrow().len(), 2);
        assert_eq!(
            changes.borrow()[0],
            (Some(Decimal::from(10)), Some(Decimal::from(20)))
        );
        assert_eq!(changes.borrow()[1], (None, None));

        visual.update(|window, cx| {
            filter.update(cx, |filter, cx| {
                filter.min = Some(Decimal::ONE);
                filter.reset_silent(window, cx);
            });
        });
        assert_eq!(changes.borrow().len(), 2);
        filter.read_with(&visual.cx, |filter, _| {
            assert_eq!(filter.value(), (None, None))
        });
        drop(filter);
        drop(visual);
        cx.run_until_parked();
    }
}
