use gpui::{
    App, IntoElement, ParentElement as _, RenderOnce, StyleRefinement, Styled, Window, div,
};
use gpui_component::{StyledExt as _, h_flex};

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

    /// Set a custom label for the row count (default: "Items Loaded")
    pub fn row_label(mut self, label: impl Into<String>) -> Self {
        self.row_label = Some(label.into());
        self
    }

    /// Set custom text for the loading state (default: "Loading...")
    pub fn loading_text(mut self, text: impl Into<String>) -> Self {
        self.loading_text = Some(text.into());
        self
    }

    /// Set custom text for the idle state (default: "Idle")
    pub fn idle_text(mut self, text: impl Into<String>) -> Self {
        self.idle_text = Some(text.into());
        self
    }

    /// Set custom text for when all data is loaded (default: "All data loaded")
    pub fn all_loaded_text(mut self, text: impl Into<String>) -> Self {
        self.all_loaded_text = Some(text.into());
        self
    }

    /// Set custom text for when more data is available (default: "Scroll for more")
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
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let row_label = self.row_label.as_deref().unwrap_or("Items Loaded");
        let loading_text = self.loading_text.as_deref().unwrap_or("Loading...");
        let idle_text = self.idle_text.as_deref().unwrap_or("Idle");
        let all_loaded_text = self.all_loaded_text.as_deref().unwrap_or("All data loaded");
        let more_available_text = self
            .more_available_text
            .as_deref()
            .unwrap_or("Scroll for more");

        h_flex()
            .gap_4()
            .refine_style(&self.style)
            .child(
                div()
                    .refine_style(&self.row_count_style)
                    .child(format!("{}: {}", row_label, self.row_count)),
            )
            .child(
                div()
                    .refine_style(&self.activity_style)
                    .child(if self.loading {
                        loading_text.to_string()
                    } else {
                        idle_text.to_string()
                    }),
            )
            .child(div().refine_style(&self.eof_style).child(if self.eof {
                all_loaded_text.to_string()
            } else {
                more_available_text.to_string()
            }))
    }
}
