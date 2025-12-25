use crate::TableFilterComponent;
use gpui::{App, Context, Entity, IntoElement, Render, Window, div, prelude::*, px};
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable,
    button::{Button, ButtonVariants},
    checkbox::Checkbox,
    divider::Divider,
    input::{Input, InputState},
    popover::Popover,
    tag::Tag,
    v_flex,
};
use gpui_table_core::filter::FacetedFilterOption;
use std::collections::HashSet;
use std::rc::Rc;

pub struct FacetedFilter {
    title: String,
    options: Vec<FacetedFilterOption>,
    selected_values: HashSet<String>,
    search_state: Option<Entity<InputState>>,
    on_change: Rc<dyn Fn(HashSet<String>, &mut Window, &mut App) + 'static>,
}

impl TableFilterComponent for FacetedFilter {
    type Value = HashSet<String>;

    const FILTER_TYPE: gpui_table_core::registry::RegistryFilterType =
        gpui_table_core::registry::RegistryFilterType::Faceted;

    fn build(
        title: impl Into<String>,
        value: Self::Value,
        on_change: impl Fn(Self::Value, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();

        cx.new(|_cx| Self {
            title,
            options: Vec::new(),
            selected_values: value,
            search_state: None,
            on_change: Rc::new(on_change),
        })
    }
}

impl FacetedFilter {
    /// Build a faceted filter with options.
    ///
    /// This is the primary constructor for faceted filters since they require
    /// a list of available options.
    pub fn build_with_options(
        title: impl Into<String>,
        options: Vec<FacetedFilterOption>,
        selected_values: HashSet<String>,
        on_change: impl Fn(HashSet<String>, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();

        cx.new(|_cx| Self {
            title,
            options,
            selected_values,
            search_state: None,
            on_change: Rc::new(on_change),
        })
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
        value: String,
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

    fn clear_filters(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.selected_values.clear();
        (self.on_change)(self.selected_values.clone(), window, cx);
        cx.notify();
    }

    /// Get the labels of selected values for display.
    fn get_selected_labels(&self) -> Vec<String> {
        self.options
            .iter()
            .filter(|opt| self.selected_values.contains(&opt.value))
            .map(|opt| opt.label.clone())
            .collect()
    }
}

impl Render for FacetedFilter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Ensure search state exists
        self.ensure_search_state(window, cx);

        let title = self.title.clone();
        let selected_count = self.selected_values.len();
        let has_selection = selected_count > 0;
        let selected_labels = self.get_selected_labels();

        let view = cx.entity().clone();
        let options = self.options.clone();
        let selected_values = self.selected_values.clone();
        let search_state = self.search_state.clone().unwrap();

        // Icon: CircleX when has selection (to clear), Plus otherwise
        let trigger_icon = if has_selection {
            IconName::CircleX
        } else {
            IconName::Plus
        };

        let clear_view = view.clone();
        let trigger = Button::new("faceted-filter-trigger")
            .outline()
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
                        div().child(Tag::secondary().small().child(format!("{} selected", selected_count)))
                    } else {
                        div().flex().items_center().gap_1().children(
                            selected_labels
                                .into_iter()
                                .map(|label| Tag::secondary().small().child(label)),
                        )
                    },
                )
            });

        Popover::new("faceted-filter-popover")
            .trigger(trigger)
            .content(move |_, _window, cx| {
                let clear_view = view.clone();

                // Build options using Checkbox like the original
                let options_view = v_flex().gap_1().children(options.iter().map(|opt| {
                    let is_selected = selected_values.contains(&opt.value);
                    let val = opt.value.clone();
                    let view = view.clone();
                    let label = opt.label.clone();
                    let count = opt.count;

                    div()
                        .w_full()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            Checkbox::new(format!("opt-{}", val))
                                .label(label)
                                .checked(is_selected)
                                .on_click(move |checked, window, cx| {
                                    view.update(cx, |this, cx| {
                                        this.toggle_option(val.clone(), *checked, window, cx);
                                    });
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

                v_flex()
                    .w_56()
                    .child(
                        div().p_2().child(
                            Input::new(&search_state)
                                .small()
                                .prefix(Icon::new(IconName::Search).xsmall()),
                        ),
                    )
                    .child(Divider::horizontal())
                    .child(
                        v_flex()
                            .id("options-list")
                            .max_h_72()
                            .overflow_y_scroll()
                            .p_1()
                            .child(options_view),
                    )
                    .when(has_selection, |this| {
                        this.child(Divider::horizontal()).child(
                            div().p_1().child(
                                Button::new("clear-filters")
                                    .ghost()
                                    .w_full()
                                    .justify_center()
                                    .label("Clear filters")
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
