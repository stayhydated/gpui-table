use crate::TableFilterComponent;
use es_fluent::{EsFluent, ToFluentString as _};
use gpui::{
    App, Context, Entity, IntoElement, Render, StyleRefinement, Task, Window, prelude::*, px,
};
use gpui_component::{
    Icon, IconName, Sizable as _, StyledExt as _, h_flex,
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

#[derive(Clone, EsFluent)]
enum TextFilterFtl {
    Placeholder { title: String },
}

pub struct TextFilter {
    title: Rc<dyn Fn() -> String>,
    value: String,
    container_style: StyleRefinement,
    input_style: StyleRefinement,
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
    /// Last placeholder applied to the input state.
    last_placeholder: Option<String>,
}

/// Extension trait for configuring TextFilter via method chaining.
pub trait TextFilterExt: Sized {
    /// Only allow alphabetic characters (a-z, A-Z) in the input.
    fn alphabetic_only(self, cx: &mut App) -> Self;
    /// Only allow numeric characters (0-9) in the input.
    fn numeric_only(self, cx: &mut App) -> Self;
    /// Only allow alphanumeric characters in the input.
    fn alphanumeric_only(self, cx: &mut App) -> Self;
    /// Set a custom validation function.
    fn validate(self, validator: impl Fn(&str) -> String + 'static, cx: &mut App) -> Self;
    /// Set style refinement for the filter container.
    fn container_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
    /// Set style refinement for the input element.
    fn input_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
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

    fn container_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _| {
            this.container_style = style;
        });
        self
    }

    fn input_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _| {
            this.input_style = style;
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
        let title = title.into();
        Self::new_with_title(Rc::new(move || title.clone()), value, on_change, cx)
    }
}

impl TextFilter {
    fn new_with_title(
        title: Rc<dyn Fn() -> String>,
        value: String,
        on_change: impl Fn(String, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|_cx| Self {
            title,
            value,
            container_style: StyleRefinement::default(),
            input_style: StyleRefinement::default().w(px(200.)),
            input_state: None,
            on_change: Rc::new(on_change),
            pending_apply: false,
            _debounce_task: None,
            validator: None,
            pending_validated_value: None,
            last_placeholder: None,
        })
    }

    /// Create a text filter with a reactive title provider (e.g. for i18n).
    pub fn new_for(
        title: impl Fn() -> String + 'static,
        value: String,
        on_change: impl Fn(String, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        Self::new_with_title(Rc::new(title), value, on_change, cx)
    }

    fn placeholder_text(&self) -> String {
        TextFilterFtl::Placeholder {
            title: (self.title)(),
        }
        .to_fluent_string()
    }

    fn ensure_input_state(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input_state.is_none() {
            let placeholder = self.placeholder_text();
            let initial_placeholder = placeholder.clone();
            let input = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(initial_placeholder)
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
            self.last_placeholder = Some(placeholder);
        }
    }

    fn reset_inner(&mut self, notify_change: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.value.clear();
        self.pending_apply = false;
        self._debounce_task = None;
        self.pending_validated_value = None;

        if let Some(input_state) = &self.input_state {
            input_state.update(cx, |input, cx| {
                input.set_value("", window, cx);
            });
        }

        if notify_change {
            (self.on_change)(self.value.clone(), window, cx);
        }

        cx.notify();
    }

    /// Apply the current filter value via callback.
    /// Call this from parent when you want to trigger the on_change.
    pub fn apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        (self.on_change)(self.value.clone(), window, cx);
    }

    /// Reset the filter value and notify via callback.
    pub fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset_inner(true, window, cx);
    }

    /// Reset the filter value without invoking callback.
    pub fn reset_silent(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset_inner(false, window, cx);
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

        // Keep placeholder reactive to title changes (e.g. locale switches).
        if let Some(input_state) = &self.input_state {
            let placeholder = self.placeholder_text();
            if self.last_placeholder.as_deref() != Some(placeholder.as_str()) {
                self.last_placeholder = Some(placeholder.clone());
                input_state.update(cx, |input, cx| {
                    input.set_placeholder(placeholder, window, cx);
                });
            }
        }

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
        h_flex()
            .gap_2()
            .items_center()
            .refine_style(&self.container_style)
            .child(
                Input::new(&input_state)
                    .small()
                    .prefix(Icon::new(IconName::Search).xsmall())
                    .cleanable(true)
                    .refine_style(&self.input_style),
            )
    }
}
