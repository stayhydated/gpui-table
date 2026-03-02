use gpui::{
    AnyElement, App, Context, Div, InteractiveElement as _, IntoElement, Stateful, Window, div,
};
use gpui_component::table::{Column, TableDelegate, TableState};

pub mod filter;
#[cfg(feature = "fluent")]
pub mod i18n;
pub mod registry;

/// Private module for macro internals. Not part of public API.
#[doc(hidden)]
pub mod __private {
    use gpui::{App, Context, Window};
    use gpui_component::table::TableState;

    /// Internal trait implemented by `#[gpui_table_impl]` to provide loading behavior.
    ///
    /// This trait bridges user-defined loading logic (via `TableLoader` trait or
    /// freestanding `#[load_more]` methods) to the generated `TableDelegate` implementation.
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

/// Trait for defining table loading behavior.
///
/// Implement this trait on your table delegate and apply `#[gpui_table_impl]`
/// to wire it up to the generated `TableDelegate` implementation.
/// The table struct must also set `#[gpui_table(load_more)]` to enable
/// load-more wiring in the generated delegate.
///
/// # Example
///
/// ```ignore
/// use gpui_table::TableLoader;
///
/// #[gpui_table_impl]
/// impl TableLoader for MyTableDelegate {
///     const THRESHOLD: usize = 20;
///
///     fn load_more(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>) {
///         // Load more data...
///     }
/// }
/// ```
pub trait TableLoader: TableDelegate {
    /// Number of rows from the bottom at which to trigger loading more data.
    const THRESHOLD: usize = 10;

    /// Load more data into the table.
    ///
    /// This method is called when the user scrolls near the bottom of the table
    /// (within `THRESHOLD` rows). The implementation should:
    /// - Check if already loading (`self.loading`) and return early if so
    /// - Check if at end of data (`self.eof`) and return early if so
    /// - Set `self.loading = true` and notify
    /// - Fetch data (typically async via `cx.spawn`)
    /// - Append new rows to `self.rows`
    /// - Set `self.eof = true` if no more data
    /// - Set `self.loading = false` and notify
    fn load_more(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>);
}

/// A value that can be displayed in a table cell.
pub trait TableCell {
    fn draw(&self, window: &mut Window, cx: &mut App) -> AnyElement;
}

#[cfg(feature = "jiff")]
mod datetime_format {
    use icu::{
        calendar::{Date, Iso},
        datetime::{
            DateTimeFormatter, NoCalendarFormatter,
            fieldsets::{self, Combo},
            input::{DateTime as IcuDateTime, Time, UtcOffset, ZonedDateTime},
        },
        locale::locale,
    };
    use jiff::{Timestamp, Zoned, civil, tz::TimeZone};

    type DateFormatter = DateTimeFormatter<fieldsets::YMD>;
    type DateTimeFormatterNoZone = DateTimeFormatter<fieldsets::YMDT>;
    type DateTimeFormatterWithZone =
        DateTimeFormatter<Combo<fieldsets::YMDT, fieldsets::zone::LocalizedOffsetLong>>;
    type TimeFormatter = NoCalendarFormatter<fieldsets::T>;

    fn date_formatter() -> Option<DateFormatter> {
        DateTimeFormatter::try_new(locale!("en-US").into(), fieldsets::YMD::medium()).ok()
    }

    fn datetime_formatter() -> Option<DateTimeFormatterNoZone> {
        let fieldset = fieldsets::YMD::medium().with_time_hms();
        DateTimeFormatter::try_new(locale!("en-US").into(), fieldset).ok()
    }

    fn zoned_datetime_formatter() -> Option<DateTimeFormatterWithZone> {
        let fieldset = fieldsets::YMD::medium()
            .with_time_hms()
            .with_zone(fieldsets::zone::LocalizedOffsetLong);
        DateTimeFormatter::try_new(locale!("en-US").into(), fieldset).ok()
    }

    fn time_formatter() -> Option<TimeFormatter> {
        NoCalendarFormatter::try_new(locale!("en-US").into(), fieldsets::T::medium()).ok()
    }

    fn to_icu_date(value: civil::Date) -> Option<Date<Iso>> {
        let month = u8::try_from(value.month()).ok()?;
        let day = u8::try_from(value.day()).ok()?;
        Date::try_new_iso(i32::from(value.year()), month, day).ok()
    }

    fn to_icu_time(value: civil::Time) -> Option<Time> {
        let hour = u8::try_from(value.hour()).ok()?;
        let minute = u8::try_from(value.minute()).ok()?;
        let second = u8::try_from(value.second()).ok()?;
        let nanosecond = u32::try_from(value.subsec_nanosecond()).ok()?;
        Time::try_new(hour, minute, second, nanosecond).ok()
    }

    pub(super) fn format_zoned(value: &Zoned) -> Option<String> {
        let utc_offset = UtcOffset::try_from_seconds(value.offset().seconds()).ok()?;
        let zoned = ZonedDateTime::from_epoch_milliseconds_and_utc_offset(
            value.timestamp().as_millisecond(),
            utc_offset,
        );
        let formatter = zoned_datetime_formatter()?;
        Some(formatter.format(&zoned).to_string())
    }

    pub(super) fn format_timestamp_local(value: Timestamp) -> Option<String> {
        let zoned = value.to_zoned(TimeZone::system());
        format_zoned(&zoned)
    }

    pub(super) fn format_civil_datetime(value: civil::DateTime) -> Option<String> {
        let date = to_icu_date(value.date())?;
        let time = to_icu_time(value.time())?;
        let datetime = IcuDateTime { date, time };
        let formatter = datetime_formatter()?;
        Some(formatter.format(&datetime).to_string())
    }

    pub(super) fn format_civil_date(value: civil::Date) -> Option<String> {
        let date = to_icu_date(value)?;
        let formatter = date_formatter()?;
        Some(formatter.format(&date).to_string())
    }

    pub(super) fn format_civil_time(value: civil::Time) -> Option<String> {
        let time = to_icu_time(value)?;
        let formatter = time_formatter()?;
        Some(formatter.format(&time).to_string())
    }

    #[cfg(feature = "chrono")]
    pub(super) fn chrono_datetime_to_system_zoned<Tz: chrono::TimeZone>(
        value: &chrono::DateTime<Tz>,
    ) -> Option<Zoned> {
        let nanosecond = i32::try_from(value.timestamp_subsec_nanos()).ok()?;
        let timestamp = Timestamp::new(value.timestamp(), nanosecond).ok()?;
        Some(timestamp.to_zoned(TimeZone::system()))
    }

    #[cfg(feature = "chrono")]
    pub(super) fn chrono_naive_datetime_to_jiff(
        value: &chrono::NaiveDateTime,
    ) -> Option<civil::DateTime> {
        use chrono::{Datelike as _, Timelike as _};

        let month = i8::try_from(value.month()).ok()?;
        let day = i8::try_from(value.day()).ok()?;
        let hour = i8::try_from(value.hour()).ok()?;
        let minute = i8::try_from(value.minute()).ok()?;
        let second = i8::try_from(value.second()).ok()?;
        let nanosecond = i32::try_from(value.nanosecond()).ok()?;
        let year = i16::try_from(value.year()).ok()?;
        civil::DateTime::new(year, month, day, hour, minute, second, nanosecond).ok()
    }

    #[cfg(feature = "chrono")]
    pub(super) fn chrono_naive_date_to_jiff(value: &chrono::NaiveDate) -> Option<civil::Date> {
        use chrono::Datelike as _;

        let month = i8::try_from(value.month()).ok()?;
        let day = i8::try_from(value.day()).ok()?;
        let year = i16::try_from(value.year()).ok()?;
        civil::Date::new(year, month, day).ok()
    }

    #[cfg(feature = "chrono")]
    pub(super) fn chrono_naive_time_to_jiff(value: &chrono::NaiveTime) -> Option<civil::Time> {
        use chrono::Timelike as _;

        let hour = i8::try_from(value.hour()).ok()?;
        let minute = i8::try_from(value.minute()).ok()?;
        let second = i8::try_from(value.second()).ok()?;
        let nanosecond = i32::try_from(value.nanosecond()).ok()?;
        civil::Time::new(hour, minute, second, nanosecond).ok()
    }
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
    String, &str, usize, isize, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128
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
        datetime_format::chrono_datetime_to_system_zoned(self)
            .as_ref()
            .and_then(datetime_format::format_zoned)
            .unwrap_or_else(|| self.to_rfc3339())
            .into_any_element()
    }
}

#[cfg(feature = "chrono")]
impl TableCell for chrono::NaiveDateTime {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        datetime_format::chrono_naive_datetime_to_jiff(self)
            .and_then(datetime_format::format_civil_datetime)
            .unwrap_or_else(|| self.to_string())
            .into_any_element()
    }
}

#[cfg(feature = "chrono")]
impl TableCell for chrono::NaiveDate {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        datetime_format::chrono_naive_date_to_jiff(self)
            .and_then(datetime_format::format_civil_date)
            .unwrap_or_else(|| self.to_string())
            .into_any_element()
    }
}

#[cfg(feature = "chrono")]
impl TableCell for chrono::NaiveTime {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        datetime_format::chrono_naive_time_to_jiff(self)
            .and_then(datetime_format::format_civil_time)
            .unwrap_or_else(|| self.to_string())
            .into_any_element()
    }
}

#[cfg(feature = "jiff")]
impl TableCell for jiff::Zoned {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        datetime_format::format_zoned(self)
            .unwrap_or_else(|| self.to_string())
            .into_any_element()
    }
}

#[cfg(feature = "jiff")]
impl TableCell for jiff::Timestamp {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        datetime_format::format_timestamp_local(*self)
            .unwrap_or_else(|| self.to_string())
            .into_any_element()
    }
}

#[cfg(feature = "jiff")]
impl TableCell for jiff::civil::DateTime {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        datetime_format::format_civil_datetime(*self)
            .unwrap_or_else(|| self.to_string())
            .into_any_element()
    }
}

#[cfg(feature = "jiff")]
impl TableCell for jiff::civil::Date {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        datetime_format::format_civil_date(*self)
            .unwrap_or_else(|| self.to_string())
            .into_any_element()
    }
}

#[cfg(feature = "jiff")]
impl TableCell for jiff::civil::Time {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        datetime_format::format_civil_time(*self)
            .unwrap_or_else(|| self.to_string())
            .into_any_element()
    }
}

#[cfg(feature = "spacetimedb")]
impl TableCell for spacetimedb::Timestamp {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        self.to_chrono_date_time()
            .ok()
            .and_then(|dt| datetime_format::chrono_datetime_to_system_zoned(&dt))
            .as_ref()
            .and_then(datetime_format::format_zoned)
            .unwrap_or_default()
            .into_any_element()
    }
}

#[cfg(feature = "spacetimedb")]
impl TableCell for spacetimedb::Identity {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        self.to_string().into_any_element()
    }
}

#[cfg(feature = "spacetimedb")]
impl TableCell for spacetimedb::ConnectionId {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        self.to_string().into_any_element()
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
