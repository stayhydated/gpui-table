use gpui::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable, InteractiveElement as _,
    IntoElement, Render, StyleRefinement, Styled as _, Window, div, prelude::*, px,
};
use gpui_component::{Sizable as _, StyledExt as _, button::Button, h_flex, v_flex};

use crate::TableStatusBar;

#[gpui_storybook::story("Table Filters")]
pub struct TableStatusBarStory {
    focus_handle: FocusHandle,
    row_count: usize,
    loading: bool,
    eof: bool,
}

impl TableStatusBarStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            row_count: 125,
            loading: false,
            eof: false,
        }
    }
}

impl Focusable for TableStatusBarStory {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui_storybook::Story for TableStatusBarStory {
    fn title() -> String {
        "Table Status Bar".into()
    }

    fn closable() -> bool {
        false
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Render for TableStatusBarStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("table-status-bar-story")
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_3()
            .child(
                div()
                    .text_lg()
                    .font_semibold()
                    .child("TableStatusBar modes"),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("status-bar-rows-plus-25")
                            .outline()
                            .small()
                            .label("+25 rows")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.row_count += 25;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("status-bar-toggle-loading")
                            .outline()
                            .small()
                            .label(if self.loading {
                                "Stop Loading"
                            } else {
                                "Start Loading"
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.loading = !this.loading;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("status-bar-toggle-eof")
                            .outline()
                            .small()
                            .label(if self.eof { "Set More" } else { "Set EOF" })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.eof = !this.eof;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("status-bar-reset")
                            .outline()
                            .small()
                            .label("Reset")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.row_count = 125;
                                this.loading = false;
                                this.eof = false;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .p_3()
                    .border_1()
                    .rounded(px(8.))
                    .child(div().text_sm().font_semibold().child("Live state"))
                    .child(
                        TableStatusBar::new(self.row_count, self.loading, self.eof)
                            .row_label("Rows loaded")
                            .loading_text("Loading...")
                            .idle_text("Idle")
                            .all_loaded_text("All data loaded")
                            .more_available_text("Scroll for more")
                            .row_count_style(StyleRefinement::default().font_semibold())
                            .activity_style(StyleRefinement::default().font_semibold())
                            .eof_style(StyleRefinement::default().font_semibold()),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .p_3()
                    .border_1()
                    .rounded(px(8.))
                    .child(div().text_sm().font_semibold().child("Static previews"))
                    .child(
                        TableStatusBar::new(self.row_count, true, false)
                            .row_label("Preview loading")
                            .loading_text("Loading...")
                            .idle_text("Idle")
                            .all_loaded_text("All data loaded")
                            .more_available_text("Scroll for more"),
                    )
                    .child(
                        TableStatusBar::new(self.row_count, false, true)
                            .row_label("Preview complete")
                            .loading_text("Loading...")
                            .idle_text("Idle")
                            .all_loaded_text("All data loaded")
                            .more_available_text("Scroll for more"),
                    ),
            )
    }
}
