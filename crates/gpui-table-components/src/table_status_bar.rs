use gpui::{App, IntoElement, ParentElement as _, RenderOnce, Styled as _, Window};
use gpui_component::h_flex;

/// Configuration for the table status bar display.
#[derive(IntoElement)]
pub struct TableStatusBar {
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
            .child(format!("{}: {}", row_label, self.row_count))
            .child(if self.loading {
                loading_text.to_string()
            } else {
                idle_text.to_string()
            })
            .child(if self.eof {
                all_loaded_text.to_string()
            } else {
                more_available_text.to_string()
            })
    }
}
