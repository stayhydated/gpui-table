use crate::text_cell;
use gpui_kit::{AnyElement, App, IntoElement as _, Window};
use std::fmt::Display;

/// A value that can be displayed in a table cell.
///
/// Built-in Jiff date and time cells use localized formatting.
pub trait TableCell {
    fn draw(&self, window: &mut Window, cx: &mut App) -> AnyElement;
}

/// Table-cell wrapper for values that should render through [`Display`].
///
/// This is useful for value objects where a blanket `TableCell for T:
/// Display` implementation would conflict with downstream implementations.
pub struct DisplayCell<T>(pub T);

impl<T> DisplayCell<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Display> TableCell for DisplayCell<T> {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        text_cell(self.0.to_string())
    }
}

/// Table-cell wrapper that renders a value with a caller-provided formatter.
pub struct FormattedCell<T, F> {
    value: T,
    formatter: F,
}

impl<T, F> FormattedCell<T, F> {
    pub fn new(value: T, formatter: F) -> Self {
        Self { value, formatter }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T, F, S> TableCell for FormattedCell<T, F>
where
    F: Fn(&T) -> S,
    S: Display,
{
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        text_cell((self.formatter)(&self.value).to_string())
    }
}

#[cfg(feature = "jiff")]
mod datetime_format {
    use icu::{
        calendar::{Date, Iso},
        datetime::{
            DateTimeFormatter, DateTimeFormatterPreferences, NoCalendarFormatter,
            fieldsets::{self, Combo},
            input::{DateTime as IcuDateTime, Time, UtcOffset, ZonedDateTime as IcuZonedDateTime},
        },
        locale::Locale,
    };
    use jiff::{Timestamp, ZonedDateTime, civil, tz::TimeZone};

    type DateFormatter = DateTimeFormatter<fieldsets::YMD>;
    type DateTimeFormatterNoZone = DateTimeFormatter<fieldsets::YMDT>;
    type DateTimeFormatterWithZone =
        DateTimeFormatter<Combo<fieldsets::YMDT, fieldsets::zone::LocalizedOffsetLong>>;
    type TimeFormatter = NoCalendarFormatter<fieldsets::T>;

    fn formatter_preferences() -> DateTimeFormatterPreferences {
        let raw = gpui_kit::component::locale();
        let normalized = raw.replace('_', "-");
        let locale = normalized.parse::<Locale>().unwrap_or_else(|error| {
            panic!("gpui-kit locale `{normalized}` is not a valid ICU locale: {error}")
        });
        locale.into()
    }

    fn date_formatter() -> DateFormatter {
        DateTimeFormatter::try_new(formatter_preferences(), fieldsets::YMD::medium())
            .unwrap_or_else(|error| panic!("failed to create ICU date formatter: {error:?}"))
    }

    fn datetime_formatter() -> DateTimeFormatterNoZone {
        let fieldset = fieldsets::YMD::medium().with_time_hms();
        DateTimeFormatter::try_new(formatter_preferences(), fieldset)
            .unwrap_or_else(|error| panic!("failed to create ICU datetime formatter: {error:?}"))
    }

    fn zoned_datetime_formatter() -> DateTimeFormatterWithZone {
        let fieldset = fieldsets::YMD::medium()
            .with_time_hms()
            .with_zone(fieldsets::zone::LocalizedOffsetLong);
        DateTimeFormatter::try_new(formatter_preferences(), fieldset).unwrap_or_else(|error| {
            panic!("failed to create ICU zoned datetime formatter: {error:?}")
        })
    }

    fn time_formatter() -> TimeFormatter {
        NoCalendarFormatter::try_new(formatter_preferences(), fieldsets::T::medium())
            .unwrap_or_else(|error| panic!("failed to create ICU time formatter: {error:?}"))
    }

    fn to_icu_date(value: civil::Date) -> Date<Iso> {
        let month = u8::try_from(value.month()).expect("jiff months fit ICU month values");
        let day = u8::try_from(value.day()).expect("jiff days fit ICU day values");
        Date::try_new_iso(i32::from(value.year()), month, day).unwrap_or_else(|error| {
            panic!("jiff date `{value}` is not a valid ICU date: {error:?}")
        })
    }

    fn to_icu_time(value: civil::Time) -> Time {
        let hour = u8::try_from(value.hour()).expect("jiff hours fit ICU hour values");
        let minute = u8::try_from(value.minute()).expect("jiff minutes fit ICU minute values");
        let second = u8::try_from(value.second()).expect("jiff seconds fit ICU second values");
        let nanosecond =
            u32::try_from(value.subsec_nanosecond()).expect("jiff nanoseconds fit ICU values");
        Time::try_new(hour, minute, second, nanosecond).unwrap_or_else(|error| {
            panic!("jiff time `{value}` is not a valid ICU time: {error:?}")
        })
    }

    pub(super) fn format_zoned(value: &ZonedDateTime) -> String {
        let Ok(utc_offset) = UtcOffset::try_from_seconds(value.offset().seconds()) else {
            // A valid Jiff fixed offset can exceed ICU's 18-hour range.
            return value.to_string();
        };
        let zoned = IcuZonedDateTime::from_epoch_milliseconds_and_utc_offset(
            i64::try_from(value.timestamp().as_nanosecond().div_euclid(1_000_000))
                .expect("Jiff epoch milliseconds fit i64"),
            utc_offset,
        );
        let formatter = zoned_datetime_formatter();
        formatter.format(&zoned).to_string()
    }

    pub(super) fn format_timestamp_local(value: Timestamp) -> String {
        let zoned = value.to_zoned_datetime(TimeZone::system());
        format_zoned(&zoned)
    }

    pub(super) fn format_civil_datetime(value: civil::DateTime) -> String {
        let date = to_icu_date(value.date());
        let time = to_icu_time(value.time());
        let datetime = IcuDateTime { date, time };
        datetime_formatter().format(&datetime).to_string()
    }

    pub(super) fn format_civil_date(value: civil::Date) -> String {
        let date = to_icu_date(value);
        date_formatter().format(&date).to_string()
    }

    pub(super) fn format_civil_time(value: civil::Time) -> String {
        let time = to_icu_time(value);
        time_formatter().format(&time).to_string()
    }
}

macro_rules! impl_table_cell_display {
    ($($t:ty),* $(,)?) => {
        $(
            impl TableCell for $t {
                fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
                    text_cell(self.to_string())
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
                    text_cell(format!("{:.2}", self))
                }
            }
        )*
    };
}

impl<T: TableCell> TableCell for Option<T> {
    fn draw(&self, window: &mut Window, cx: &mut App) -> AnyElement {
        match self {
            Some(value) => value.draw(window, cx),
            None => text_cell(""),
        }
    }
}

impl<T: TableCell> TableCell for Vec<T> {
    fn draw(&self, window: &mut Window, cx: &mut App) -> AnyElement {
        use gpui_kit::{InteractiveElement as _, ParentElement as _, Styled as _};

        let mut children = Vec::with_capacity(self.len().saturating_mul(2));
        for (index, value) in self.iter().enumerate() {
            if index > 0 {
                children.push(", ".into_any_element());
            }
            children.push(
                gpui_kit::div()
                    .id(("cell-value", index))
                    .child(value.draw(window, cx))
                    .into_any_element(),
            );
        }

        gpui_kit::div()
            .flex()
            .items_center()
            .gap_1()
            .children(children)
            .into_any_element()
    }
}

impl_table_cell_display!(
    String, &str, usize, isize, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128
);
impl_table_cell_float!(f32, f64);

impl TableCell for bool {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        text_cell(if *self { "✓" } else { "✗" })
    }
}

#[cfg(feature = "spacetimedb")]
impl TableCell for spacetimedb_lib::Timestamp {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        text_cell(self.to_string())
    }
}

#[cfg(feature = "rust_decimal")]
impl TableCell for rust_decimal::Decimal {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        text_cell(format!("{:.2}", self))
    }
}

#[cfg(feature = "jiff")]
impl TableCell for jiff::ZonedDateTime {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        text_cell(datetime_format::format_zoned(self))
    }
}

#[cfg(feature = "jiff")]
impl TableCell for jiff::Timestamp {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        text_cell(datetime_format::format_timestamp_local(*self))
    }
}

#[cfg(feature = "jiff")]
impl TableCell for jiff::civil::DateTime {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        text_cell(datetime_format::format_civil_datetime(*self))
    }
}

#[cfg(feature = "jiff")]
impl TableCell for jiff::civil::Date {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        text_cell(datetime_format::format_civil_date(*self))
    }
}

#[cfg(feature = "jiff")]
impl TableCell for jiff::civil::Time {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        text_cell(datetime_format::format_civil_time(*self))
    }
}

#[cfg(feature = "spacetimedb")]
impl TableCell for spacetimedb_lib::TimeDuration {
    fn draw(&self, _: &mut Window, _: &mut App) -> AnyElement {
        text_cell(self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{DisplayCell, FormattedCell};

    #[test]
    fn cell_wrappers_preserve_values_and_formatters() {
        let display = DisplayCell::new(42);
        assert_eq!(display.into_inner(), 42);

        let formatted = FormattedCell::new(42, |value: &i32| format!("{value} kg"));
        assert_eq!(formatted.value(), &42);
        assert_eq!((formatted.formatter)(formatted.value()), "42 kg");
        assert_eq!(formatted.into_inner(), 42);
    }

    #[cfg(feature = "jiff")]
    #[test]
    fn jiff_values_format_through_icu() {
        use super::datetime_format::{
            format_civil_date, format_civil_datetime, format_civil_time, format_timestamp_local,
            format_zoned,
        };
        use jiff::{Timestamp, ZonedDateTime, civil};

        let date = civil::Date::new(2026, 7, 11).unwrap();
        let time = civil::Time::new(12, 34, 56, 123_000_000).unwrap();
        let datetime = civil::DateTime::from_parts(date, time);
        let zoned: ZonedDateTime = "2026-07-11T12:34:56-04:00[America/New_York]"
            .parse()
            .unwrap();
        let timestamp: Timestamp = "2026-07-11T16:34:56Z".parse().unwrap();

        assert!(!format_civil_date(date).is_empty());
        assert!(!format_civil_time(time).is_empty());
        assert!(!format_civil_datetime(datetime).is_empty());
        assert!(!format_zoned(&zoned).is_empty());
        assert!(!format_timestamp_local(timestamp).is_empty());
    }

    #[cfg(feature = "jiff")]
    #[test]
    fn pre_epoch_cell_does_not_round_across_midnight() {
        use super::datetime_format::format_zoned;
        use jiff::{Timestamp, tz::TimeZone};

        let formatted = |nanos| {
            format_zoned(
                &Timestamp::from_nanosecond(nanos)
                    .unwrap()
                    .to_zoned_datetime(TimeZone::UTC),
            )
        };
        assert_eq!(formatted(-1), formatted(-1_000_000));
        assert_ne!(formatted(-1), formatted(0));
    }

    #[cfg(feature = "jiff")]
    #[test]
    fn valid_offsets_outside_icu_range_keep_the_original_value() {
        use super::datetime_format::format_zoned;
        use jiff::{
            Timestamp,
            tz::{Offset, TimeZone},
        };

        let timestamp: Timestamp = "2026-01-01T00:00:00Z".parse().unwrap();
        let offset = Offset::from_seconds(19 * 3600).unwrap();
        let zoned = timestamp.to_zoned_datetime(TimeZone::fixed(offset));
        assert_eq!(format_zoned(&zoned), zoned.to_string());
    }
}
