use es_fluent::EsFluent;
use gpui_kit::component::{StyledExt as _, h_flex};
use gpui_kit::{
    App, IntoElement, ParentElement as _, RenderOnce, StyleRefinement, Styled, Window, div, gpui,
};

use crate::i18n::localize_message;

#[derive(Clone, Debug, EsFluent)]
enum TableStatusBarFtl {
    ItemsLoaded { count: usize },
    Loading,
    Idle,
    AllDataLoaded,
    ScrollForMore,
}

/// Configuration for the table status bar display.
#[derive(IntoElement)]
pub struct TableStatusBar {
    style: StyleRefinement,
    row_count_style: StyleRefinement,
    activity_style: StyleRefinement,
    eof_style: StyleRefinement,
    row_count: usize,
    loading: bool,
    eof: bool,
    row_label: Option<String>,
    loading_text: Option<String>,
    idle_text: Option<String>,
    all_loaded_text: Option<String>,
    more_available_text: Option<String>,
}

impl TableStatusBar {
    /// Create a new status bar with the given state.
    pub fn new(row_count: usize, loading: bool, eof: bool) -> Self {
        Self {
            style: StyleRefinement::default(),
            row_count_style: StyleRefinement::default(),
            activity_style: StyleRefinement::default(),
            eof_style: StyleRefinement::default(),
            row_count,
            loading,
            eof,
            row_label: None,
            loading_text: None,
            idle_text: None,
            all_loaded_text: None,
            more_available_text: None,
        }
    }

    /// Set a custom label for the row count.
    pub fn row_label(mut self, label: impl Into<String>) -> Self {
        self.row_label = Some(label.into());
        self
    }

    /// Set custom text for the loading state.
    pub fn loading_text(mut self, text: impl Into<String>) -> Self {
        self.loading_text = Some(text.into());
        self
    }

    /// Set custom text for the idle state.
    pub fn idle_text(mut self, text: impl Into<String>) -> Self {
        self.idle_text = Some(text.into());
        self
    }

    /// Set custom text for when all data is loaded.
    pub fn all_loaded_text(mut self, text: impl Into<String>) -> Self {
        self.all_loaded_text = Some(text.into());
        self
    }

    /// Set custom text for when more data is available.
    pub fn more_available_text(mut self, text: impl Into<String>) -> Self {
        self.more_available_text = Some(text.into());
        self
    }

    /// Set style refinement for the row-count segment.
    pub fn row_count_style(mut self, style: StyleRefinement) -> Self {
        self.row_count_style = style;
        self
    }

    /// Set style refinement for the loading/idle segment.
    pub fn activity_style(mut self, style: StyleRefinement) -> Self {
        self.activity_style = style;
        self
    }

    /// Set style refinement for the eof/more-available segment.
    pub fn eof_style(mut self, style: StyleRefinement) -> Self {
        self.eof_style = style;
        self
    }
}

impl Styled for TableStatusBar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for TableStatusBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let row_count_text = self.row_label.map_or_else(
            || {
                localize_message(
                    cx,
                    &TableStatusBarFtl::ItemsLoaded {
                        count: self.row_count,
                    },
                )
            },
            |label| format!("{label}: {}", self.row_count),
        );
        let loading_text = self
            .loading_text
            .unwrap_or_else(|| localize_message(cx, &TableStatusBarFtl::Loading));
        let idle_text = self
            .idle_text
            .unwrap_or_else(|| localize_message(cx, &TableStatusBarFtl::Idle));
        let all_loaded_text = self
            .all_loaded_text
            .unwrap_or_else(|| localize_message(cx, &TableStatusBarFtl::AllDataLoaded));
        let more_available_text = self
            .more_available_text
            .unwrap_or_else(|| localize_message(cx, &TableStatusBarFtl::ScrollForMore));

        h_flex()
            .gap_4()
            .refine_style(&self.style)
            .child(
                div()
                    .refine_style(&self.row_count_style)
                    .child(row_count_text),
            )
            .child(
                div()
                    .refine_style(&self.activity_style)
                    .child(if self.loading {
                        loading_text
                    } else {
                        idle_text
                    }),
            )
            .child(div().refine_style(&self.eof_style).child(if self.eof {
                all_loaded_text
            } else {
                more_available_text
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::TableStatusBar;
    use gpui_kit::gpui;
    use gpui_kit::{
        Empty, IntoElement as _, RenderOnce as _, StyleRefinement, Styled as _, TestAppContext,
        VisualTestContext,
    };

    #[gpui_kit::test]
    fn status_bar_builders_and_state_combinations_render(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::component::init(cx);
            crate::i18n::init(cx).expect("table status-bar localization should initialize");
        });

        let mut custom = TableStatusBar::new(42, true, false)
            .row_label("Rows")
            .loading_text("Working")
            .idle_text("Ready")
            .all_loaded_text("Complete")
            .more_available_text("More")
            .row_count_style(StyleRefinement::default())
            .activity_style(StyleRefinement::default())
            .eof_style(StyleRefinement::default());
        *custom.style() = StyleRefinement::default();

        assert_eq!(custom.row_count, 42);
        assert!(custom.loading);
        assert!(!custom.eof);
        assert_eq!(custom.row_label.as_deref(), Some("Rows"));

        let window = cx.add_window(|_, _| Empty);
        let mut visual = VisualTestContext::from_window(window.into(), cx);
        visual.update(|window, cx| {
            let _ = custom.render(window, cx).into_any_element();
            let _ = TableStatusBar::new(0, false, true)
                .render(window, cx)
                .into_any_element();
            let _ = TableStatusBar::new(1, false, false)
                .render(window, cx)
                .into_any_element();
        });
    }
}
