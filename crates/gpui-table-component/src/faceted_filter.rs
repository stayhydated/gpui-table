use crate::TableFilterComponent;
use es_fluent::{EsFluent, ToFluentString as _};
use gpui::{
    App, Context, Entity, IntoElement, Render, StyleRefinement, Window, div, prelude::*, px,
};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    divider::Divider,
    h_flex,
    input::{Input, InputState},
    popover::Popover,
    tag::Tag,
    v_flex,
};
use gpui_table_core::filter::{FacetedFilterOption, FilterValue, Filterable};
use std::collections::HashSet;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone, EsFluent)]
enum FacetedFilterFtl {
    NoResultsFound,
    ClearFilters,
    SelectedCount { count: String },
}

pub struct FacetedFilter<T: FilterValue> {
    title: Rc<dyn Fn() -> String>,
    options: Rc<dyn Fn() -> Vec<FacetedFilterOption>>,
    selected_values: HashSet<T>,
    trigger_style: StyleRefinement,
    selected_tag_style: StyleRefinement,
    popover_style: StyleRefinement,
    search_input_style: StyleRefinement,
    options_list_style: StyleRefinement,
    option_button_style: StyleRefinement,
    clear_button_style: StyleRefinement,
    search_state: Option<Entity<InputState>>,
    on_change: Rc<dyn Fn(HashSet<T>, &mut Window, &mut App) + 'static>,
    /// Whether to show the search input (default: false)
    show_search: bool,
    _marker: PhantomData<T>,
}

/// Extension trait for configuring FacetedFilter via method chaining.
pub trait FacetedFilterExt: Sized {
    /// Enable search functionality for filtering options.
    fn searchable(self, cx: &mut App) -> Self;
    /// Set style refinement for the trigger button.
    fn trigger_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
    /// Set style refinement for selected-value tags in the trigger.
    fn selected_tag_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
    /// Set style refinement for the popover root content.
    fn popover_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
    /// Set style refinement for the search input.
    fn search_input_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
    /// Set style refinement for the options list container.
    fn options_list_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
    /// Set style refinement for each option button.
    fn option_button_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
    /// Set style refinement for the clear-filters button.
    fn clear_button_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
}

impl<T: FilterValue> FacetedFilterExt for Entity<FacetedFilter<T>> {
    fn searchable(self, cx: &mut App) -> Self {
        self.update(cx, |this, _| {
            this.show_search = true;
        });
        self
    }

    fn trigger_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _| {
            this.trigger_style = style;
        });
        self
    }

    fn selected_tag_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _| {
            this.selected_tag_style = style;
        });
        self
    }

    fn popover_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _| {
            this.popover_style = style;
        });
        self
    }

    fn search_input_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _| {
            this.search_input_style = style;
        });
        self
    }

    fn options_list_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _| {
            this.options_list_style = style;
        });
        self
    }

    fn option_button_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _| {
            this.option_button_style = style;
        });
        self
    }

    fn clear_button_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _| {
            this.clear_button_style = style;
        });
        self
    }
}

impl<T: FilterValue> TableFilterComponent for FacetedFilter<T> {
    type Value = HashSet<T>;

    const FILTER_TYPE: gpui_table_core::registry::RegistryFilterType =
        gpui_table_core::registry::RegistryFilterType::Faceted;

    fn new(
        title: impl Into<String>,
        value: Self::Value,
        on_change: impl Fn(Self::Value, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();

        cx.new(|_cx| Self {
            title: Rc::new(move || title.clone()),
            options: Rc::new(Vec::new),
            selected_values: value,
            trigger_style: StyleRefinement::default(),
            selected_tag_style: StyleRefinement::default(),
            popover_style: StyleRefinement::default(),
            search_input_style: StyleRefinement::default(),
            options_list_style: StyleRefinement::default(),
            option_button_style: StyleRefinement::default(),
            clear_button_style: StyleRefinement::default(),
            search_state: None,
            on_change: Rc::new(on_change),
            show_search: false,
            _marker: PhantomData,
        })
    }
}

impl<T: FilterValue> FacetedFilter<T> {
    fn ensure_search_state(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.search_state.is_none() {
            let input = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("Search...")
                    .clean_on_escape()
            });
            self.search_state = Some(input);
        }
    }

    fn toggle_option(
        &mut self,
        value: T,
        checked: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if checked {
            self.selected_values.insert(value);
        } else {
            self.selected_values.remove(&value);
        }
        (self.on_change)(self.selected_values.clone(), window, cx);
        cx.notify();
    }

    fn reset_inner(&mut self, notify_change: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.selected_values.clear();

        if notify_change {
            (self.on_change)(self.selected_values.clone(), window, cx);
        }

        cx.notify();
    }

    fn clear_filters(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset_inner(true, window, cx);
    }

    /// Reset selected values and notify via callback.
    pub fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset_inner(true, window, cx);
    }

    /// Reset selected values without invoking callback.
    pub fn reset_silent(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset_inner(false, window, cx);
    }

    /// Get the labels of selected values for display.
    fn get_selected_labels(&self) -> Vec<String> {
        let options = (self.options)();
        let selected_strings: HashSet<String> = self
            .selected_values
            .iter()
            .map(|v| v.to_filter_string())
            .collect();
        options
            .iter()
            .filter(|opt| selected_strings.contains(&opt.value))
            .map(|opt| opt.label.clone())
            .collect()
    }

    /// Get the current filter value (selected values).
    pub fn value(&self) -> &HashSet<T> {
        &self.selected_values
    }

    /// Check if a value is currently selected.
    fn is_selected(&self, value_str: &str) -> bool {
        self.selected_values
            .iter()
            .any(|v| v.to_filter_string() == value_str)
    }
}

impl<T: Filterable> FacetedFilter<T> {
    /// Build a faceted filter with options derived from a type implementing `Filterable`.
    ///
    /// This is the preferred constructor for enum-based filters. The options are
    /// automatically generated from the enum's `Filterable` implementation, which
    /// includes labels (from `#[filter(fluent)]` or `#[filter(label = "...")]`) and
    /// icons (from `#[filter(icon = IconName::...)]`).
    ///
    /// # Example
    /// ```ignore
    /// #[derive(strum::EnumIter, Filterable)]
    /// #[filter(fluent)]
    /// pub enum Priority {
    ///     #[filter(icon = IconName::ArrowDown)]
    ///     Low,
    ///     #[filter(icon = IconName::ArrowUp)]
    ///     High,
    /// }
    ///
    /// let filter = FacetedFilter::<Priority>::new_for(
    ///     || "Priority".to_string(),
    ///     HashSet::new(),
    ///     move |value, _window, cx| { /* handle change */ },
    ///     cx,
    /// );
    /// ```
    pub fn new_for(
        title: impl Fn() -> String + 'static,
        selected_values: HashSet<T>,
        on_change: impl Fn(HashSet<T>, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|_cx| Self {
            title: Rc::new(title),
            options: Rc::new(T::options),
            selected_values,
            trigger_style: StyleRefinement::default(),
            selected_tag_style: StyleRefinement::default(),
            popover_style: StyleRefinement::default(),
            search_input_style: StyleRefinement::default(),
            options_list_style: StyleRefinement::default(),
            option_button_style: StyleRefinement::default(),
            clear_button_style: StyleRefinement::default(),
            search_state: None,
            on_change: Rc::new(on_change),
            show_search: false,
            _marker: PhantomData,
        })
    }

    /// Create a faceted filter with options.
    ///
    /// Use this constructor when you need to provide options dynamically
    /// (e.g., for i18n support where labels need to update on language change).
    pub fn new_with_options(
        title: impl Fn() -> String + 'static,
        options: impl Fn() -> Vec<FacetedFilterOption> + 'static,
        selected_values: HashSet<T>,
        on_change: impl Fn(HashSet<T>, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|_cx| Self {
            title: Rc::new(title),
            options: Rc::new(options),
            selected_values,
            trigger_style: StyleRefinement::default(),
            selected_tag_style: StyleRefinement::default(),
            popover_style: StyleRefinement::default(),
            search_input_style: StyleRefinement::default(),
            options_list_style: StyleRefinement::default(),
            option_button_style: StyleRefinement::default(),
            clear_button_style: StyleRefinement::default(),
            search_state: None,
            on_change: Rc::new(on_change),
            show_search: false,
            _marker: PhantomData,
        })
    }
}

impl<T: FilterValue> Render for FacetedFilter<T> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Only create search state if searchable is enabled
        if self.show_search {
            self.ensure_search_state(window, cx);
        }

        let title = (self.title)();
        let selected_count = self.selected_values.len();
        let has_selection = selected_count > 0;
        let selected_labels = self.get_selected_labels();

        let view = cx.entity().clone();
        let options_fn = self.options.clone();
        let trigger_style = self.trigger_style.clone();
        let selected_tag_style = self.selected_tag_style.clone();
        let popover_style = self.popover_style.clone();
        let search_input_style = self.search_input_style.clone();
        let options_list_style = self.options_list_style.clone();
        let option_button_style = self.option_button_style.clone();
        let clear_button_style = self.clear_button_style.clone();
        // Convert selected values to strings for use in the closure
        let selected_strings: HashSet<String> = self
            .selected_values
            .iter()
            .map(|v| v.to_filter_string())
            .collect();
        let search_state = self.search_state.clone();

        // Icon: CircleX when has selection (to clear), Plus otherwise
        let trigger_icon = if has_selection {
            IconName::CircleX
        } else {
            IconName::Plus
        };

        let clear_view = view.clone();
        let trigger = Button::new("faceted-filter-trigger")
            .outline()
            .refine_style(&trigger_style)
            .child(
                div()
                    .id("clear-icon")
                    .when(has_selection, |this| {
                        this.cursor_pointer()
                            .rounded_sm()
                            .hover(|s| s.opacity(1.0))
                            .opacity(0.7)
                            .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                                clear_view.update(cx, |this, cx| {
                                    this.clear_filters(window, cx);
                                });
                            })
                    })
                    .child(Icon::new(trigger_icon).xsmall()),
            )
            .child(title)
            .when(has_selection, |b| {
                b.child(Divider::vertical().h(px(16.)).mx_1()).child(
                    // Show tags for selected values
                    // If more than 2 selected, show "{n} selected" tag
                    // Otherwise show individual tags for each selected value
                    if selected_count > 2 {
                        div().child(
                            Tag::secondary()
                                .small()
                                .child(
                                    FacetedFilterFtl::SelectedCount {
                                        count: selected_count.to_string(),
                                    }
                                    .to_fluent_string(),
                                )
                                .refine_style(&selected_tag_style),
                        )
                    } else {
                        div().flex().items_center().gap_1().children(
                            selected_labels.into_iter().map(|label| {
                                Tag::secondary()
                                    .small()
                                    .child(label)
                                    .refine_style(&selected_tag_style)
                            }),
                        )
                    },
                )
            });

        Popover::new("faceted-filter-popover")
            .trigger(trigger)
            .content(move |_, _window, cx| {
                let clear_view = view.clone();
                let option_button_style = option_button_style.clone();
                let search_input_style = search_input_style.clone();
                let options_list_style = options_list_style.clone();
                let popover_style = popover_style.clone();
                let clear_button_style = clear_button_style.clone();

                // Get fresh options (for i18n reactivity)
                let options = options_fn();

                // Get search query to filter options (only if search is enabled)
                let search_query = search_state
                    .as_ref()
                    .map(|s| s.read(cx).text().to_string().to_lowercase())
                    .unwrap_or_default();

                // Filter options based on search query
                let filtered_options: Vec<_> = options
                    .iter()
                    .filter(|opt| {
                        if search_query.is_empty() {
                            true
                        } else {
                            opt.label.to_lowercase().contains(&search_query)
                        }
                    })
                    .collect();

                // Build options list with icons - each option is a full-width ghost button
                let options_view = v_flex()
                    .gap_1()
                    .children(filtered_options.iter().map(|opt| {
                        let is_selected = selected_strings.contains(&opt.value);
                        let val_str = opt.value.clone();
                        let view = view.clone();
                        let label = opt.label.clone();
                        let count = opt.count;
                        let icon = opt.icon.clone();

                        div()
                            .w_full()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                Button::new(format!("opt-btn-{}", val_str))
                                    .ghost()
                                    .flex_1()
                                    .justify_start()
                                    .refine_style(&option_button_style)
                                    .child(
                                        h_flex()
                                            .w_full()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                Checkbox::new(format!("opt-{}", val_str))
                                                    .checked(is_selected),
                                            )
                                            .when_some(icon, |this, icon_name| {
                                                this.child(
                                                    Icon::new(icon_name)
                                                        .xsmall()
                                                        .text_color(cx.theme().muted_foreground),
                                                )
                                            })
                                            .child(label),
                                    )
                                    .on_click({
                                        let view = view.clone();
                                        let val_str = val_str.clone();
                                        move |_, window, cx| {
                                            view.update(cx, |this, cx| {
                                                // Toggle: if selected, deselect; if not, select
                                                let is_currently_selected =
                                                    this.is_selected(&val_str);
                                                if is_currently_selected {
                                                    // Remove: find and remove the matching value
                                                    this.selected_values.retain(|v| {
                                                        v.to_filter_string() != val_str
                                                    });
                                                    (this.on_change)(
                                                        this.selected_values.clone(),
                                                        window,
                                                        cx,
                                                    );
                                                    cx.notify();
                                                } else {
                                                    // Add: parse the string back to T
                                                    if let Some(typed_val) =
                                                        T::from_filter_string(&val_str)
                                                    {
                                                        this.toggle_option(
                                                            typed_val, true, window, cx,
                                                        );
                                                    }
                                                }
                                            });
                                        }
                                    }),
                            )
                            .when(count.is_some(), |d| {
                                d.child(
                                    div()
                                        .text_xs()
                                        .font_family("monospace")
                                        .text_color(cx.theme().muted_foreground)
                                        .child(count.unwrap().to_string()),
                                )
                            })
                    }));

                // Show "No results" message if search yields nothing
                let has_results = !filtered_options.is_empty();

                v_flex()
                    .w_56()
                    .refine_style(&popover_style)
                    .when_some(search_state.clone(), |this, search_state| {
                        this.child(
                            div().p_2().child(
                                Input::new(&search_state)
                                    .small()
                                    .prefix(Icon::new(IconName::Search).xsmall())
                                    .refine_style(&search_input_style),
                            ),
                        )
                        .child(Divider::horizontal())
                    })
                    .child(
                        v_flex()
                            .id("options-list")
                            .max_h_72()
                            .overflow_y_scroll()
                            .p_1()
                            .refine_style(&options_list_style)
                            .when(has_results, |this| this.child(options_view))
                            .when(!has_results, |this| {
                                this.child(
                                    div()
                                        .py_4()
                                        .w_full()
                                        .flex()
                                        .justify_center()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(FacetedFilterFtl::NoResultsFound.to_fluent_string()),
                                )
                            }),
                    )
                    .when(has_selection, |this| {
                        this.child(Divider::horizontal()).child(
                            div().p_1().child(
                                Button::new("clear-filters")
                                    .ghost()
                                    .w_full()
                                    .justify_center()
                                    .label(FacetedFilterFtl::ClearFilters.to_fluent_string())
                                    .refine_style(&clear_button_style)
                                    .on_click(move |_, window, cx| {
                                        clear_view.update(cx, |this, cx| {
                                            this.clear_filters(window, cx);
                                        });
                                    }),
                            ),
                        )
                    })
            })
    }
}
