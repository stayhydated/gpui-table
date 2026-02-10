use std::collections::HashSet;

use gpui::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable, InteractiveElement as _,
    IntoElement, Render, StyleRefinement, Styled as _, Window, div, prelude::*, px,
};
use gpui_component::{Sizable as _, StyledExt as _, button::Button, h_flex, v_flex};

use crate::{FacetedFilterExt as _, faceted_filter::FacetedFilter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FacetedStoryMode {
    Basic,
    Searchable,
    Styled,
}

impl FacetedStoryMode {
    fn label(self) -> &'static str {
        match self {
            Self::Basic => "Basic",
            Self::Searchable => "Searchable",
            Self::Styled => "Styled",
        }
    }
}

#[gpui_storybook::story("Table Filters")]
pub struct FacetedFilterStory {
    focus_handle: FocusHandle,
    mode: FacetedStoryMode,
    faceted_filter: Entity<FacetedFilter<bool>>,
}

impl FacetedFilterStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mode = FacetedStoryMode::Searchable;
        let faceted_filter = Self::build_filter(mode, cx);

        Self {
            focus_handle: cx.focus_handle(),
            mode,
            faceted_filter,
        }
    }

    fn build_filter(mode: FacetedStoryMode, cx: &mut App) -> Entity<FacetedFilter<bool>> {
        let mut selected_values = HashSet::new();
        if mode != FacetedStoryMode::Basic {
            selected_values.insert(true);
        }

        let filter = FacetedFilter::<bool>::new_for(
            || "Archived".to_string(),
            selected_values,
            |_value, _window, _cx| {},
            cx,
        );

        let filter = if mode != FacetedStoryMode::Basic {
            filter.searchable(cx)
        } else {
            filter
        };

        if mode == FacetedStoryMode::Styled {
            filter
                .trigger_style(StyleRefinement::default().w(px(360.)), cx)
                .popover_style(StyleRefinement::default().w(px(420.)), cx)
                .search_input_style(StyleRefinement::default().w_full(), cx)
                .options_list_style(StyleRefinement::default().w_full(), cx)
                .option_button_style(StyleRefinement::default().w_full(), cx)
                .selected_tag_style(StyleRefinement::default().font_semibold(), cx)
                .clear_button_style(StyleRefinement::default().font_semibold(), cx)
        } else {
            filter
        }
    }

    fn set_mode(&mut self, mode: FacetedStoryMode, cx: &mut Context<Self>) {
        if self.mode == mode {
            return;
        }
        self.mode = mode;
        self.faceted_filter = Self::build_filter(mode, cx);
        cx.notify();
    }
}

impl Focusable for FacetedFilterStory {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui_storybook::Story for FacetedFilterStory {
    fn title() -> String {
        "Faceted Filter".into()
    }

    fn closable() -> bool {
        false
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Render for FacetedFilterStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut selected_values = self
            .faceted_filter
            .read(cx)
            .value()
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>();
        selected_values.sort();
        let selected_display = if selected_values.is_empty() {
            "(none)".to_string()
        } else {
            selected_values.join(", ")
        };

        v_flex()
            .id("faceted-filter-story")
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_3()
            .child(div().text_lg().font_semibold().child("FacetedFilter modes"))
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("faceted-story-mode-basic")
                            .outline()
                            .small()
                            .label(FacetedStoryMode::Basic.label())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.set_mode(FacetedStoryMode::Basic, cx);
                            })),
                    )
                    .child(
                        Button::new("faceted-story-mode-searchable")
                            .outline()
                            .small()
                            .label(FacetedStoryMode::Searchable.label())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.set_mode(FacetedStoryMode::Searchable, cx);
                            })),
                    )
                    .child(
                        Button::new("faceted-story-mode-styled")
                            .outline()
                            .small()
                            .label(FacetedStoryMode::Styled.label())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.set_mode(FacetedStoryMode::Styled, cx);
                            })),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .p_3()
                    .border_1()
                    .rounded(px(8.))
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .child(format!("Current mode: {}", self.mode.label())),
                    )
                    .child(self.faceted_filter.clone())
                    .child(
                        div()
                            .text_sm()
                            .child(format!("Selected values: {selected_display}")),
                    ),
            )
    }
}
