use crate::TableFilterComponent;
use gpui::{App, Context, Entity, IntoElement, Render, Task, Timer, Window, prelude::*, px};
use gpui_component::{
    Icon, IconName, Sizable as _, h_flex,
    input::{Input, InputEvent, InputState},
};
use std::rc::Rc;
use std::time::Duration;

/// Debounce delay in milliseconds for filter changes
const DEBOUNCE_MS: u64 = 300;

/// Text validation function type
pub type TextValidator = Rc<dyn Fn(&str) -> String>;

/// Built-in validators for common text filtering patterns
pub mod validators {
    /// Only allow ASCII characters
    pub fn ascii_only(s: &str) -> String {
        s.chars().filter(|c| c.is_ascii()).collect()
    }

    /// Only allow numeric characters (0-9)
    pub fn numeric_only(s: &str) -> String {
        s.chars().filter(|c| c.is_ascii_digit()).collect()
    }

    /// Only allow alphanumeric characters
    pub fn alphanumeric_only(s: &str) -> String {
        s.chars().filter(|c| c.is_alphanumeric()).collect()
    }
}

pub struct TextFilter {
    title: String,
    value: String,
    input_state: Option<Entity<InputState>>,
    on_change: Rc<dyn Fn(String, &mut Window, &mut App) + 'static>,
    /// Flag set by debounce task to trigger apply during next render
    pending_apply: bool,
    /// Current debounce task - dropping it cancels the pending apply
    _debounce_task: Option<Task<()>>,
    /// Optional validator function to filter input
    validator: Option<TextValidator>,
    /// Pending validated value to apply to input during render
    pending_validated_value: Option<String>,
}

/// Extension trait for configuring TextFilter via method chaining.
pub trait TextFilterExt {
    /// Only allow alphabetic characters (a-z, A-Z) in the input.
    fn alphabetic_only(self, cx: &mut App) -> Self;
    /// Only allow numeric characters (0-9) in the input.
    fn numeric_only(self, cx: &mut App) -> Self;
    /// Only allow alphanumeric characters in the input.
    fn alphanumeric_only(self, cx: &mut App) -> Self;
    /// Set a custom validation function.
    fn validate(self, validator: impl Fn(&str) -> String + 'static, cx: &mut App) -> Self;
}

impl TextFilterExt for Entity<TextFilter> {
    fn alphabetic_only(self, cx: &mut App) -> Self {
        self.validate(
            |text| text.chars().filter(|c| c.is_alphabetic()).collect(),
            cx,
        )
    }

    fn numeric_only(self, cx: &mut App) -> Self {
        self.validate(|text| text.chars().filter(|c| c.is_numeric()).collect(), cx)
    }

    fn alphanumeric_only(self, cx: &mut App) -> Self {
        self.validate(
            |text| text.chars().filter(|c| c.is_alphanumeric()).collect(),
            cx,
        )
    }

    fn validate(self, validator: impl Fn(&str) -> String + 'static, cx: &mut App) -> Self {
        self.update(cx, |this, _| {
            this.validator = Some(Rc::new(validator));
        });
        self
    }
}

impl TableFilterComponent for TextFilter {
    type Value = String;

    const FILTER_TYPE: gpui_table_core::registry::RegistryFilterType =
        gpui_table_core::registry::RegistryFilterType::Text;

    fn new(
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
            pending_apply: false,
            _debounce_task: None,
            validator: None,
            pending_validated_value: None,
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

            // Subscribe to input changes with debounce
            cx.subscribe(
                &input,
                |this: &mut Self, state, event: &InputEvent, cx| match event {
                    InputEvent::Change => {
                        let raw_value = state.read(cx).value().to_string();

                        // Apply validator if set
                        let new_value = if let Some(ref validator) = this.validator {
                            let validated = validator(&raw_value);
                            // If validation changed the value, schedule update for next render
                            if validated != raw_value {
                                this.pending_validated_value = Some(validated.clone());
                                cx.notify();
                            }
                            validated
                        } else {
                            raw_value
                        };

                        this.value = new_value;

                        // Cancel any pending debounce task and schedule a new one
                        this._debounce_task = Some(cx.spawn(async move |view, cx| {
                            Timer::after(Duration::from_millis(DEBOUNCE_MS)).await;
                            view.update(cx, |this, cx| {
                                this.pending_apply = true;
                                this._debounce_task = None;
                                cx.notify();
                            })
                            .ok();
                        }));
                    },
                    InputEvent::PressEnter { .. } => {
                        // On enter, apply immediately without debounce
                        this._debounce_task = None;
                        this.pending_apply = true;
                        cx.notify();
                    },
                    _ => {},
                },
            )
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

        // Apply pending validated value if any
        if let Some(validated) = self.pending_validated_value.take()
            && let Some(input_state) = &self.input_state
        {
            input_state.update(cx, |input, cx| {
                input.set_value(validated, window, cx);
            });
        }

        // Apply pending changes now that we have window access
        if self.pending_apply {
            self.pending_apply = false;
            (self.on_change)(self.value.clone(), window, cx);
        }

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
