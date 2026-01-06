use gpui::{
    AnyElement, App, Context, Div, InteractiveElement as _, IntoElement, Stateful, Window, div,
};
use gpui_component::table::{Column, TableDelegate, TableState};

pub mod filter;
pub mod registry;

/// Private module for macro internals. Not part of public API.
#[doc(hidden)]
pub mod __private {
    use gpui::{App, Context, Window};
    use gpui_component::table::TableState;

    /// Marker trait indicating a delegate has a `#[load_more]` method.
    ///
    /// This trait is implemented by the `#[gpui_table_impl]` attribute macro
    /// when it finds a method marked with `#[load_more]`. The derive macro
    /// uses this to generate the proper `TableDelegate::load_more` delegation.
    pub trait HasLoadMore: gpui_component::table::TableDelegate {
        /// Internal method that delegates to the user's load_more implementation.
        fn __load_more_impl(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>);
    }

    /// Trait providing load_more related TableDelegate method implementations.
    ///
    /// This trait is implemented by the `#[gpui_table_impl]` attribute macro
    /// and provides the `load_more`, `has_more`, and `load_more_threshold` methods
    /// that will be used by the generated `TableDelegate` implementation.
    pub trait LoadMoreDelegate: gpui_component::table::TableDelegate {
        /// Check if there is more data to load.
        fn has_more(&self, app: &App) -> bool;

        /// Threshold of rows from bottom to trigger load_more.
        fn load_more_threshold(&self) -> usize {
            10 // Default threshold
        }

        /// Load more data into the table.
        fn load_more(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>);
    }
}

/// Trait for table delegates that support loading data.
///
/// This trait provides the interface for loading initial and additional data
/// into a table. It's used by the generated code to trigger data loading
/// without needing to know the specific implementation details.
///
/// # Example
///
/// ```ignore
/// impl TableDataLoader for MyTableDelegate {
///     fn load_data(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>) {
///         // Load initial batch of data
///         self.load_more_items(window, cx);
///     }
/// }
/// ```
pub trait TableDataLoader: TableDelegate {
    /// Load data into the table.
    ///
    /// This method is called to trigger data loading (either initial load
    /// or loading more data). The implementation should handle:
    /// - Setting loading state
    /// - Fetching data (sync or async)
    /// - Appending to rows
    /// - Updating eof flag when no more data
    fn load_data(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>);
}

/// A value that can be displayed in a table cell.
pub trait TableCell {
    fn draw(&self, window: &mut Window, cx: &mut App) -> AnyElement;
}

macro_rules! impl_table_cell_display {
    ($($t:ty),* $(,)?) => {
        $(
            impl TableCell for $t {
                fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
                    self.to_string().into_any_element()
                }
            }
        )*
    };
}

macro_rules! impl_table_cell_float {
    ($($t:ty),* $(,)?) => {
        $(
            impl TableCell for $t {
                fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
                    format!("{:.2}", self).into_any_element()
                }
            }
        )*
    };
}

impl<T: TableCell> TableCell for Option<T> {
    fn draw(&self, window: &mut Window, cx: &mut App) -> AnyElement {
        match self {
            Some(value) => value.draw(window, cx),
            None => "".into_any_element(),
        }
    }
}

impl_table_cell_display!(
    String, &str, usize, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128
);
impl_table_cell_float!(f32, f64);

impl TableCell for bool {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        (if *self { "✓" } else { "✗" }).into_any_element()
    }
}

#[cfg(feature = "rust_decimal")]
impl TableCell for rust_decimal::Decimal {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        format!("{:.2}", self).into_any_element()
    }
}

#[cfg(feature = "chrono")]
impl<Tz: chrono::TimeZone> TableCell for chrono::DateTime<Tz>
where
    Tz::Offset: std::fmt::Display,
{
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        self.to_rfc3339().into_any_element()
    }
}

#[cfg(feature = "chrono")]
impl TableCell for chrono::NaiveDateTime {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        self.format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .into_any_element()
    }
}

#[cfg(feature = "chrono")]
impl TableCell for chrono::NaiveDate {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        self.format("%Y-%m-%d").to_string().into_any_element()
    }
}

#[cfg(feature = "chrono")]
impl TableCell for chrono::NaiveTime {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        self.format("%H:%M:%S").to_string().into_any_element()
    }
}

/// Metadata for a table row type.
pub trait TableRowMeta {
    /// Unique identifier for this row type.
    const TABLE_ID: &'static str;

    /// Human-readable title for the table.
    const TABLE_TITLE: &'static str;

    /// Returns the table title. This can be overridden to provide dynamic
    /// titles, for example from localization libraries.
    fn table_title() -> String {
        Self::TABLE_TITLE.to_string()
    }

    /// Returns the column definitions for this row type.
    fn table_columns() -> Vec<Column>;

    /// Returns the value for a specific column index.
    fn cell_value(&self, col_ix: usize) -> Box<dyn TableCell + '_>;

    /// Returns the filter configuration for the table.
    fn table_filters() -> Vec<crate::filter::FilterConfig> {
        Vec::new()
    }
}

/// Styling hooks for a table row.
///
/// This trait allows customizing how rows and cells are rendered.
/// The `GpuiTable` derive macro generates a default implementation
/// that uses `default_render_cell` and `default_render_row`.
pub trait TableRowStyle: TableRowMeta {
    /// The type representing the columns of the table.
    /// Usually an enum generated by the derive macro.
    type ColumnId: Into<usize> + From<usize>;

    /// Renders a single cell.
    fn render_table_cell(
        &self,
        col: Self::ColumnId,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement;

    /// Renders the row container.
    fn render_table_row(&self, row_ix: usize, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        default_render_row(row_ix, window, cx)
    }
}

/// Default implementation for rendering a cell.
pub fn default_render_cell<R: TableRowMeta + ?Sized>(
    row: &R,
    col_ix: usize,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    row.cell_value(col_ix).draw(window, cx)
}

/// Default implementation for rendering a row.
pub fn default_render_row(row_ix: usize, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
    div().id(row_ix)
}
