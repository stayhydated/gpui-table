use gpui_kit::component::table::{TableDelegate, TableState};
use gpui_kit::{App, Context, Window};

/// Internal trait implemented by `#[gpui_table_impl]` to provide loading behavior.
///
/// This trait bridges user-defined `TableLoader` logic to the generated
/// `TableDelegate` implementation.
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

#[cfg(test)]
mod tests {
    use super::{LoadMoreDelegate, TableLoader};
    use gpui_kit::component::table::{Column, TableDelegate, TableState};
    use gpui_kit::{App, Context, IntoElement, Window, div};

    struct Delegate;

    impl TableDelegate for Delegate {
        fn columns_count(&self, _cx: &App) -> usize {
            0
        }

        fn rows_count(&self, _cx: &App) -> usize {
            0
        }

        fn column(&self, _col_ix: usize, _cx: &App) -> Column {
            unreachable!("the threshold contract does not inspect columns")
        }

        fn render_td(
            &mut self,
            _row_ix: usize,
            _col_ix: usize,
            _window: &mut Window,
            _cx: &mut Context<TableState<Self>>,
        ) -> impl IntoElement {
            div()
        }
    }

    impl LoadMoreDelegate for Delegate {
        fn has_more(&self, _app: &App) -> bool {
            true
        }

        fn load_more(&mut self, _window: &mut Window, _cx: &mut Context<TableState<Self>>) {}
    }

    impl TableLoader for Delegate {
        fn load_more(&mut self, _window: &mut Window, _cx: &mut Context<TableState<Self>>) {}
    }

    #[test]
    fn loading_traits_share_the_documented_default_threshold() {
        assert_eq!(
            <Delegate as LoadMoreDelegate>::load_more_threshold(&Delegate),
            10
        );
        assert_eq!(<Delegate as TableLoader>::THRESHOLD, 10);
    }
}
