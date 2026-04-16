use gpui::{AnyElement, App, IntoElement, Window};

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

#[cfg(feature = "spacetimedb")]
impl TableCell for spacetimedb_lib::Timestamp {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        self.to_string().into_any_element()
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
