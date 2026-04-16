use gpui::{App, Context, Window};
use gpui_component::table::{TableDelegate, TableState};

/// Internal trait implemented by `#[gpui_table_impl]` to provide loading behavior.
///
/// This trait bridges user-defined loading logic (via `TableLoader` trait or
/// freestanding `#[load_more]` methods) to the generated `TableDelegate` implementation.
pub trait LoadMoreDelegate: TableDelegate {
    /// Check if there is more data to load.
    fn has_more(&self, app: &App) -> bool;

    /// Threshold of rows from bottom to trigger load_more.
    fn load_more_threshold(&self) -> usize {
        10
    }

    /// Load more data into the table.
    fn load_more(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>);
}

/// Trait for table delegates that support loading data.
pub trait TableDataLoader: TableDelegate {
    /// Load data into the table.
    fn load_data(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>);
}

/// Trait for defining table loading behavior.
pub trait TableLoader: TableDelegate {
    /// Number of rows from the bottom at which to trigger loading more data.
    const THRESHOLD: usize = 10;

    /// Load more data into the table.
    fn load_more(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>);
}
