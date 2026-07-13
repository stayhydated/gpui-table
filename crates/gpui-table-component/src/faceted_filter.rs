use crate::TableFilterComponent;
use es_fluent::EsFluent;
use gpui::{
    App, Context, Entity, IntoElement, Render, StyleRefinement, Window, div, prelude::*, px,
};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex,
    input::{Input, InputState},
    popover::Popover,
    separator::Separator,
    tag::Tag,
    v_flex,
};
use gpui_table_core::filter::{FacetedFilterOption, FilterValue, Filterable};
use std::borrow::Borrow as _;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::rc::Rc;

fn app<'a, T>(cx: &'a Context<'_, T>) -> &'a App {
    cx.borrow()
}

#[derive(Clone, EsFluent)]
enum FacetedFilterFtl {
    NoResultsFound,
    ClearFilters,
    SelectedCount { count: String },
}

pub struct FacetedFilter<T: FilterValue> {
    title: Rc<dyn Fn(&App) -> String>,
    options: Rc<dyn Fn(&App) -> Vec<FacetedFilterOption>>,
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

impl<T: FilterValue> component_shape::ComponentShapeMetadata for FacetedFilter<T> {
    const MCP_INPUT: component_shape::McpInput = component_shape::McpInput::string_set();
}
impl<T: Filterable> component_shape::DeclaredComponentShape for FacetedFilter<T> {}
impl<T: Filterable> component_shape::ComponentShapeFor<T> for FacetedFilter<T> {}
impl<T: Filterable> component_shape::ComponentShapeFor<Option<T>> for FacetedFilter<T> {}
impl<T: Filterable> component_shape::ComponentShapeFor<Vec<T>> for FacetedFilter<T> {}
impl<T: Filterable> component_shape::ComponentShapeFor<Option<Vec<T>>> for FacetedFilter<T> {}

#[derive(Clone)]
struct FacetedOptionGroup {
    title: Option<String>,
    options: Vec<FacetedFilterOption>,
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

    const FILTER_TYPE: gpui_table_schema::registry::RegistryFilterType =
        gpui_table_schema::registry::RegistryFilterType::Faceted;

    fn new(
        title: impl Into<String>,
        value: Self::Value,
        on_change: impl Fn(Self::Value, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();
        Self::new_inner(
            Rc::new(move |_| title.clone()),
            Rc::new(|_| Vec::new()),
            value,
            Rc::new(on_change),
            cx,
        )
    }
}

impl<T: FilterValue> FacetedFilter<T> {
    fn new_inner(
        title: Rc<dyn Fn(&App) -> String>,
        options: Rc<dyn Fn(&App) -> Vec<FacetedFilterOption>>,
        selected_values: HashSet<T>,
        on_change: Rc<dyn Fn(HashSet<T>, &mut Window, &mut App) + 'static>,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|_cx| Self {
            title,
            options,
            selected_values,
            trigger_style: StyleRefinement::default(),
            selected_tag_style: StyleRefinement::default(),
            popover_style: StyleRefinement::default(),
            search_input_style: StyleRefinement::default(),
            options_list_style: StyleRefinement::default(),
            option_button_style: StyleRefinement::default(),
            clear_button_style: StyleRefinement::default(),
            search_state: None,
            on_change,
            show_search: false,
            _marker: PhantomData,
        })
    }

    /// Create a faceted filter with a fixed title.
    pub fn new(
        title: impl Into<String>,
        selected_values: HashSet<T>,
        on_change: impl Fn(HashSet<T>, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();
        Self::new_inner(
            Rc::new(move |_| title.clone()),
            Rc::new(|_| Vec::new()),
            selected_values,
            Rc::new(on_change),
            cx,
        )
    }

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

    /// Replace selected facets without invoking the change callback.
    pub fn set_silent(&mut self, value: HashSet<T>, _window: &mut Window, cx: &mut Context<Self>) {
        self.selected_values = value;
        cx.notify();
    }

    /// Get the selected options for trigger-tag display.
    fn get_selected_options(&self, cx: &App) -> Vec<FacetedFilterOption> {
        let options = (self.options)(cx);
        let selected_strings: HashSet<String> = self
            .selected_values
            .iter()
            .map(|v| v.to_filter_string())
            .collect();
        options
            .iter()
            .filter(|opt| selected_strings.contains(&opt.value))
            .cloned()
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
    ///     |_| "Priority".to_string(),
    ///     HashSet::new(),
    ///     move |value, _window, cx| { /* handle change */ },
    ///     cx,
    /// );
    /// ```
    pub fn new_for(
        title: impl Fn(&App) -> String + 'static,
        selected_values: HashSet<T>,
        on_change: impl Fn(HashSet<T>, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        Self::new_inner(
            Rc::new(title),
            Rc::new(|_| T::options()),
            selected_values,
            Rc::new(on_change),
            cx,
        )
    }

    /// Create a faceted filter with options.
    ///
    /// Use this constructor when you need to provide options dynamically
    /// (e.g., for i18n support where labels need to update on language change).
    pub fn new_with_options(
        title: impl Fn(&App) -> String + 'static,
        options: impl Fn(&App) -> Vec<FacetedFilterOption> + 'static,
        selected_values: HashSet<T>,
        on_change: impl Fn(HashSet<T>, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        Self::new_inner(
            Rc::new(title),
            Rc::new(options),
            selected_values,
            Rc::new(on_change),
            cx,
        )
    }
}

impl<T: FilterValue> Render for FacetedFilter<T> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        crate::i18n::sync_locale(cx);

        let should_show_search = self.show_search
            || (self.options)(app(cx))
                .iter()
                .any(|opt| opt.group.is_some());

        // Only create search state if searchable is enabled or grouping benefits from search.
        if should_show_search {
            self.ensure_search_state(window, cx);
        }

        let title = (self.title)(app(cx));
        let selected_count = self.selected_values.len();
        let has_selection = selected_count > 0;
        let selected_options = self.get_selected_options(app(cx));

        let view = cx.entity();
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
                b.child(Separator::vertical().h(px(16.)).mx_1()).child(
                    // Show tags for selected values
                    // If more than 2 selected, show "{n} selected" tag
                    // Otherwise show individual tags for each selected value
                    if selected_count > 2 {
                        div().child(
                            Tag::secondary()
                                .small()
                                .child(crate::i18n::localize_message(
                                    cx,
                                    &FacetedFilterFtl::SelectedCount {
                                        count: selected_count.to_string(),
                                    },
                                ))
                                .refine_style(&selected_tag_style),
                        )
                    } else {
                        div().flex().items_center().gap_1().children(
                            selected_options.into_iter().map(|option| {
                                let label = display_option_label(&option);
                                Tag::secondary()
                                    .small()
                                    .child(
                                        h_flex()
                                            .gap_1()
                                            .when_some(option.icon, |this, icon_name| {
                                                this.child(
                                                    Icon::default()
                                                        .path(icon_name.path().to_string())
                                                        .xsmall(),
                                                )
                                            })
                                            .child(label),
                                    )
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
                let options = options_fn(app(cx));

                // Get search query to filter options (only if search is enabled)
                let search_query = search_state
                    .as_ref()
                    .map(|s| s.read(cx).text().to_string().to_lowercase())
                    .unwrap_or_default();

                // Filter options based on search query
                let grouped_options = group_options(&options, &search_query);

                // Build options list with icons - each option is a full-width ghost button
                let options_view = v_flex()
                    .gap_2()
                    .children(grouped_options.iter().map(|group| {
                        v_flex()
                            .gap_1()
                            .when_some(group.title.clone(), |this, title| {
                                this.child(
                                    div()
                                        .px_2()
                                        .pt_1()
                                        .text_xs()
                                        .font_semibold()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(title),
                                )
                            })
                            .children(group.options.iter().map(|opt| {
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
                                                        let icon_path =
                                                            icon_name.path().to_string();
                                                        this.child(
                                                            Icon::default()
                                                                .path(icon_path)
                                                                .xsmall()
                                                                .text_color(
                                                                    cx.theme().muted_foreground,
                                                                ),
                                                        )
                                                    })
                                                    .child(label),
                                            )
                                            .on_click(move |_, window, cx| {
                                                view.update(cx, |this, cx| {
                                                    let is_currently_selected =
                                                        this.is_selected(&val_str);
                                                    if is_currently_selected {
                                                        this.selected_values.retain(|v| {
                                                            v.to_filter_string() != val_str
                                                        });
                                                        (this.on_change)(
                                                            this.selected_values.clone(),
                                                            window,
                                                            cx,
                                                        );
                                                        cx.notify();
                                                    } else if let Some(typed_val) =
                                                        T::from_filter_string(&val_str)
                                                    {
                                                        this.toggle_option(
                                                            typed_val, true, window, cx,
                                                        );
                                                    }
                                                });
                                            }),
                                    )
                                    .when_some(count, |d, count| {
                                        d.child(
                                            div()
                                                .text_xs()
                                                .font_family("monospace")
                                                .text_color(cx.theme().muted_foreground)
                                                .child(count.to_string()),
                                        )
                                    })
                            }))
                    }));

                // Show "No results" message if search yields nothing
                let has_results = grouped_options
                    .iter()
                    .any(|group| !group.options.is_empty());

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
                        .child(Separator::horizontal())
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
                                        .child(crate::i18n::localize_message(
                                            cx,
                                            &FacetedFilterFtl::NoResultsFound,
                                        )),
                                )
                            }),
                    )
                    .when(has_selection, |this| {
                        this.child(Separator::horizontal()).child(
                            div().p_1().child(
                                Button::new("clear-filters")
                                    .ghost()
                                    .w_full()
                                    .justify_center()
                                    .label(crate::i18n::localize_message(
                                        cx,
                                        &FacetedFilterFtl::ClearFilters,
                                    ))
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

fn display_option_label(option: &FacetedFilterOption) -> String {
    option
        .group
        .as_ref()
        .map(|group| format!("{group}: {}", option.label))
        .unwrap_or_else(|| option.label.clone())
}

fn group_options(options: &[FacetedFilterOption], search_query: &str) -> Vec<FacetedOptionGroup> {
    let mut groups: Vec<FacetedOptionGroup> = Vec::new();
    let normalized_query = search_query.trim().to_lowercase();

    for option in options.iter().filter(|option| {
        normalized_query.is_empty()
            || option.label.to_lowercase().contains(&normalized_query)
            || option
                .group
                .as_ref()
                .is_some_and(|group| group.to_lowercase().contains(&normalized_query))
    }) {
        if let Some(group) = groups
            .iter_mut()
            .find(|group| group.title.as_ref() == option.group.as_ref())
        {
            group.options.push(option.clone());
        } else {
            groups.push(FacetedOptionGroup {
                title: option.group.clone(),
                options: vec![option.clone()],
            });
        }
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::{FacetedFilter, FacetedFilterExt as _, display_option_label, group_options};
    use gpui::{Empty, StyleRefinement, TestAppContext, VisualTestContext};
    use gpui_table_core::filter::{FacetedFilterOption, FilterValue, Filterable};
    use std::{cell::RefCell, collections::HashSet, rc::Rc};

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    enum Status {
        Active,
        Pending,
        Disabled,
    }

    impl FilterValue for Status {
        fn to_filter_string(&self) -> String {
            match self {
                Self::Active => "active",
                Self::Pending => "pending",
                Self::Disabled => "disabled",
            }
            .into()
        }

        fn from_filter_string(value: &str) -> Option<Self> {
            match value {
                "active" => Some(Self::Active),
                "pending" => Some(Self::Pending),
                "disabled" => Some(Self::Disabled),
                _ => None,
            }
        }
    }

    impl Filterable for Status {
        fn options() -> Vec<FacetedFilterOption> {
            vec![
                FacetedFilterOption {
                    group: Some("Enabled".into()),
                    label: "Active".into(),
                    value: "active".into(),
                    count: Some(2),
                    icon: Some(gpui_table_core::filter::FacetedFilterIcon::from_path(
                        "icons/check.svg",
                    )),
                },
                FacetedFilterOption {
                    group: Some("Enabled".into()),
                    label: "Pending".into(),
                    value: "pending".into(),
                    count: None,
                    icon: None,
                },
                FacetedFilterOption {
                    group: None,
                    label: "Disabled".into(),
                    value: "disabled".into(),
                    count: None,
                    icon: None,
                },
            ]
        }
    }

    #[test]
    fn option_grouping_preserves_order_groups_and_search_contracts() {
        let options = Status::options();
        assert_eq!(display_option_label(&options[0]), "Enabled: Active");
        assert_eq!(display_option_label(&options[2]), "Disabled");

        let groups = group_options(&options, "");
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].title.as_deref(), Some("Enabled"));
        assert_eq!(groups[0].options.len(), 2);
        assert_eq!(groups[1].title, None);
        assert_eq!(groups[1].options[0].value, "disabled");

        let label_match = group_options(&options, " active ");
        assert_eq!(label_match.len(), 1);
        assert_eq!(label_match[0].options[0].value, "active");

        let group_match = group_options(&options, "enabled");
        assert_eq!(group_match[0].options.len(), 2);
        assert!(group_options(&options, "missing").is_empty());
    }

    #[gpui::test]
    fn constructors_selection_options_and_styles_preserve_state(cx: &mut TestAppContext) {
        let selected = HashSet::from([Status::Active]);
        let filter = cx.update(|cx| {
            FacetedFilter::<Status>::new_for(
                |_| "Status".into(),
                selected.clone(),
                |_, _, _| {},
                cx,
            )
            .searchable(cx)
            .trigger_style(StyleRefinement::default(), cx)
            .selected_tag_style(StyleRefinement::default(), cx)
            .popover_style(StyleRefinement::default(), cx)
            .search_input_style(StyleRefinement::default(), cx)
            .options_list_style(StyleRefinement::default(), cx)
            .option_button_style(StyleRefinement::default(), cx)
            .clear_button_style(StyleRefinement::default(), cx)
        });

        filter.read_with(cx, |filter, cx| {
            assert_eq!((filter.title)(cx), "Status");
            assert_eq!(filter.value(), &selected);
            assert!(filter.show_search);
            assert!(filter.is_selected("active"));
            assert!(!filter.is_selected("pending"));
            let selected_options = filter.get_selected_options(cx);
            assert_eq!(selected_options.len(), 1);
            assert_eq!(
                display_option_label(&selected_options[0]),
                "Enabled: Active"
            );
            assert_eq!(
                selected_options[0].icon.as_ref().map(|icon| icon.path()),
                Some("icons/check.svg")
            );
            assert!(filter.search_state.is_none());
        });

        let manual = cx.update(|cx| {
            FacetedFilter::<Status>::new_with_options(
                |_| "Manual".into(),
                |_| Status::options(),
                HashSet::new(),
                |_, _, _| {},
                cx,
            )
        });
        manual.read_with(cx, |filter, cx| {
            assert_eq!((filter.options)(cx).len(), 3);
            assert!(filter.value().is_empty());
        });

        let fixed =
            cx.update(|cx| FacetedFilter::<Status>::new("Fixed", HashSet::new(), |_, _, _| {}, cx));
        fixed.read_with(cx, |filter, cx| {
            assert_eq!((filter.title)(cx), "Fixed");
            assert!((filter.options)(cx).is_empty());
        });
    }

    #[gpui::test]
    fn faceted_filter_toggle_and_reset_paths_use_window_context(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let changes = Rc::new(RefCell::new(Vec::new()));
        let changes_for_callback = changes.clone();
        let filter = cx.update(|cx| {
            FacetedFilter::<Status>::new_for(
                |_| "Status".into(),
                HashSet::new(),
                move |value, _, _| changes_for_callback.borrow_mut().push(value),
                cx,
            )
        });
        let window = cx.add_window(|_, _| Empty);
        let mut visual = VisualTestContext::from_window(window.into(), cx);

        visual.update(|window, cx| {
            filter.update(cx, |filter, cx| {
                filter.toggle_option(Status::Active, true, window, cx);
                filter.toggle_option(Status::Active, false, window, cx);
                filter.toggle_option(Status::Pending, true, window, cx);
                filter.reset(window, cx);
            });
        });
        assert_eq!(changes.borrow().len(), 4);
        assert_eq!(changes.borrow()[0], HashSet::from([Status::Active]));
        assert!(changes.borrow()[1].is_empty());
        assert_eq!(changes.borrow()[2], HashSet::from([Status::Pending]));
        assert!(changes.borrow()[3].is_empty());

        visual.update(|window, cx| {
            filter.update(cx, |filter, cx| {
                filter.selected_values.insert(Status::Disabled);
                filter.reset_silent(window, cx);
            });
        });
        assert_eq!(changes.borrow().len(), 4);
        filter.read_with(&visual.cx, |filter, _| assert!(filter.value().is_empty()));
        drop(filter);
        drop(visual);
        cx.run_until_parked();
    }
}
