use crate::TableFilterComponent;
use es_fluent::EsFluent;
use gpui_kit::component::{
    Icon, IconName, Sizable as _, StyledExt as _, h_flex,
    input::{Input, InputEvent, InputState},
};
use gpui_kit::{
    App, Context, Entity, IntoElement, Render, StyleRefinement, Task, Window, prelude::*, px,
};
use std::borrow::Borrow;
use std::rc::Rc;
use std::time::Duration;

/// Debounce delay in milliseconds for filter changes
const DEBOUNCE_MS: u64 = 300;

/// Text validation function type
pub type TextValidator = Rc<dyn Fn(&str) -> String>;
type TextInputValidator = Rc<dyn Fn(&str, &str) -> String>;

/// Built-in validators for common text filtering patterns
pub mod validators {
    use regex::Regex;

    /// Only allow alphabetic characters.
    pub fn alphabetic_only(s: &str) -> String {
        s.chars().filter(|c| c.is_alphabetic()).collect()
    }

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

    /// Compile a regex that must match the complete candidate value.
    ///
    /// The pattern is wrapped with `\A(?:...)\z`; include partial quantifiers
    /// when normal typing should accept incomplete values.
    pub fn full_match_regex(pattern: &str) -> Result<Regex, regex::Error> {
        Regex::new(&format!(r"\A(?:{pattern})\z"))
    }

    /// Accept regex-matching candidate values and otherwise keep the previous value.
    pub fn matching_regex(regex: Regex) -> impl Fn(&str, &str) -> String {
        move |candidate, previous| {
            if regex.is_match(candidate) {
                candidate.to_string()
            } else {
                previous.to_string()
            }
        }
    }

    /// Build a validator from a regex pattern that must match the full value.
    pub fn matching_regex_pattern(
        pattern: &str,
    ) -> Result<impl Fn(&str, &str) -> String + 'static, regex::Error> {
        full_match_regex(pattern).map(matching_regex)
    }
}

#[derive(Clone, EsFluent)]
enum TextFilterFtl {
    Placeholder { title: String },
}

pub struct TextFilter {
    title: Rc<dyn Fn(&App) -> String>,
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
    validator: Option<TextInputValidator>,
    /// Pending validated value to apply to input during render
    pending_validated_value: Option<String>,
    /// Last placeholder applied to the input state.
    last_placeholder: Option<String>,
}

impl component_shape::ComponentShapeMetadata for TextFilter {
    const MCP_INPUT: component_shape::McpInput = component_shape::McpInput::string();
}
impl component_shape::DeclaredComponentShape for TextFilter {}
impl component_shape::ComponentShapeFor<String> for TextFilter {}
impl component_shape::ComponentShapeFor<Option<String>> for TextFilter {}

/// Extension trait for configuring TextFilter via method chaining.
pub trait TextFilterExt: Sized {
    /// Only allow alphabetic characters (a-z, A-Z) in the input.
    fn alphabetic_only(self, cx: &mut App) -> Self;
    /// Only allow numeric characters (0-9) in the input.
    fn numeric_only(self, cx: &mut App) -> Self;
    /// Only allow alphanumeric characters in the input.
    fn alphanumeric_only(self, cx: &mut App) -> Self;
    /// Accept only candidate values that fully match the regex pattern.
    ///
    /// Include partial quantifiers when normal typing should accept incomplete
    /// values, such as `[A-Z0-9-]*` for an uppercase identifier in progress.
    fn matching_regex(self, pattern: impl AsRef<str>, cx: &mut App) -> Self;
    /// Try to configure a full-value regex validator.
    fn try_matching_regex(
        self,
        pattern: impl AsRef<str>,
        cx: &mut App,
    ) -> Result<Self, regex::Error>;
    /// Set a custom validation function.
    fn validate(self, validator: impl Fn(&str) -> String + 'static, cx: &mut App) -> Self;
    /// Set a custom validation function that can keep the previous accepted value.
    fn validate_with_previous(
        self,
        validator: impl Fn(&str, &str) -> String + 'static,
        cx: &mut App,
    ) -> Self;
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
        self.validate(validators::alphabetic_only, cx)
    }

    fn numeric_only(self, cx: &mut App) -> Self {
        self.validate(validators::numeric_only, cx)
    }

    fn alphanumeric_only(self, cx: &mut App) -> Self {
        self.validate(validators::alphanumeric_only, cx)
    }

    fn matching_regex(self, pattern: impl AsRef<str>, cx: &mut App) -> Self {
        let pattern = pattern.as_ref();
        self.try_matching_regex(pattern, cx)
            .unwrap_or_else(|error| panic!("invalid TextFilter regex `{pattern}`: {error}"))
    }

    fn try_matching_regex(
        self,
        pattern: impl AsRef<str>,
        cx: &mut App,
    ) -> Result<Self, regex::Error> {
        let validator = validators::matching_regex_pattern(pattern.as_ref())?;
        Ok(self.validate_with_previous(validator, cx))
    }

    fn validate(self, validator: impl Fn(&str) -> String + 'static, cx: &mut App) -> Self {
        self.validate_with_previous(move |value, _previous| validator(value), cx)
    }

    fn validate_with_previous(
        self,
        validator: impl Fn(&str, &str) -> String + 'static,
        cx: &mut App,
    ) -> Self {
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

    const FILTER_TYPE: gpui_table_schema::registry::RegistryFilterType =
        gpui_table_schema::registry::RegistryFilterType::Text;

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

impl TextFilter {
    /// Create a text filter with a fixed title.
    pub fn new(
        title: impl Into<String>,
        value: String,
        on_change: impl Fn(String, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();
        Self::new_with_title(Rc::new(move |_| title.clone()), value, on_change, cx)
    }

    fn new_with_title(
        title: Rc<dyn Fn(&App) -> String>,
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
        title: impl Fn(&App) -> String + 'static,
        value: String,
        on_change: impl Fn(String, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        Self::new_with_title(Rc::new(title), value, on_change, cx)
    }

    fn placeholder_text(&self, cx: &impl Borrow<App>) -> String {
        crate::i18n::localize_message(
            cx,
            &TextFilterFtl::Placeholder {
                title: (self.title)(cx.borrow()),
            },
        )
    }

    fn ensure_input_state(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input_state.is_none() {
            let placeholder = self.placeholder_text(cx);
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
                            let previous_value = this.value.clone();
                            let validated = validator(&raw_value, &previous_value);
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

    /// Replace the filter value without invoking the change callback.
    pub fn set_silent(&mut self, value: String, window: &mut Window, cx: &mut Context<Self>) {
        self.value = value.clone();
        self.pending_apply = false;
        self._debounce_task = None;
        self.pending_validated_value = None;
        if let Some(input) = &self.input_state {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        } else {
            self.pending_validated_value = Some(value);
        }
        cx.notify();
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
            let placeholder = self.placeholder_text(cx);
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

        let Some(input_state) = self.input_state.clone() else {
            return h_flex()
                .gap_2()
                .items_center()
                .refine_style(&self.container_style)
                .into_any_element();
        };

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
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{TextFilter, TextFilterExt as _, validators};
    use gpui_kit::gpui;
    use gpui_kit::{Empty, StyleRefinement, TestAppContext, VisualTestContext};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn matching_regex_accepts_complete_matches() {
        let validator = validators::matching_regex_pattern(r"[A-Z]{0,3}-?[0-9]{0,3}")
            .expect("regex should compile");

        assert_eq!(validator("ABC-123", "ABC-12"), "ABC-123");
        assert_eq!(validator("ABC-123x", "ABC-123"), "ABC-123");
    }

    #[test]
    fn matching_regex_rejects_substring_only_matches() {
        let validator =
            validators::matching_regex_pattern(r"[0-9]*").expect("regex should compile");

        assert_eq!(validator("123", ""), "123");
        assert_eq!(validator("abc123", "123"), "123");
    }

    #[test]
    fn built_in_character_validators_preserve_only_their_allowed_classes() {
        assert_eq!(validators::alphabetic_only("Ab 12-é"), "Abé");
        assert_eq!(validators::ascii_only("ASCII-é🙂"), "ASCII-");
        assert_eq!(validators::numeric_only("a1２3"), "13");
        assert_eq!(validators::alphanumeric_only("A-1_é🙂"), "A1é");
    }

    #[test]
    fn regex_builders_report_invalid_patterns_and_keep_previous_values() {
        assert!(validators::full_match_regex("[").is_err());
        assert!(validators::matching_regex_pattern("[").is_err());

        let regex = validators::full_match_regex("[A-Z]+").unwrap();
        let validator = validators::matching_regex(regex);
        assert_eq!(validator("ABC", "OLD"), "ABC");
        assert_eq!(validator("ABC1", "OLD"), "OLD");
    }

    #[gpui_kit::test]
    fn text_filter_constructors_and_configuration_preserve_state(cx: &mut TestAppContext) {
        let filter = cx.update(|cx| {
            TextFilter::new("Name", "Alice".into(), |_, _, _| {}, cx)
                .numeric_only(cx)
                .alphabetic_only(cx)
                .alphanumeric_only(cx)
                .matching_regex("[A-Za-z]*", cx)
                .validate(|value| value.trim().to_string(), cx)
                .validate_with_previous(
                    |value, previous| {
                        if value.is_empty() {
                            previous.to_string()
                        } else {
                            value.to_string()
                        }
                    },
                    cx,
                )
                .container_style(StyleRefinement::default(), cx)
                .input_style(StyleRefinement::default(), cx)
        });

        filter.read_with(cx, |filter, cx| {
            assert_eq!(filter.value(), "Alice");
            assert_eq!((filter.title)(cx), "Name");
            assert!(filter.validator.is_some());
            assert!(filter.input_state.is_none());
            assert!(!filter.pending_apply);
        });

        let reactive = cx.update(|cx| {
            TextFilter::new_for(|_| "Reactive".into(), String::new(), |_, _, _| {}, cx)
        });
        reactive.read_with(cx, |filter, cx| {
            assert_eq!((filter.title)(cx), "Reactive");
            assert_eq!(filter.value(), "");
        });

        assert!(
            cx.update(|cx| reactive.clone().try_matching_regex("[", cx))
                .is_err()
        );
    }

    #[gpui_kit::test]
    fn text_filter_apply_and_reset_paths_use_window_context(cx: &mut TestAppContext) {
        cx.update(gpui_kit::component::init);
        let changes = Rc::new(RefCell::new(Vec::new()));
        let changes_for_callback = changes.clone();
        let filter = cx.update(|cx| {
            TextFilter::new(
                "Name",
                "Alice".into(),
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
        assert_eq!(&*changes.borrow(), &["Alice", ""]);

        visual.update(|window, cx| {
            filter.update(cx, |filter, cx| {
                filter.value = "silent".into();
                filter.reset_silent(window, cx);
            });
        });
        assert_eq!(&*changes.borrow(), &["Alice", ""]);
        filter.read_with(&visual.cx, |filter, _| assert_eq!(filter.value(), ""));
        drop(filter);
        drop(visual);
        cx.run_until_parked();
    }
}
