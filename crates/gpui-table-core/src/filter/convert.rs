//! Conversion traits for filter matching.

/// Trait for converting numeric types to Decimal for range filter matching.
#[cfg(feature = "rust_decimal")]
pub trait ToDecimal {
    fn to_decimal(&self) -> rust_decimal::Decimal;
}

#[cfg(feature = "rust_decimal")]
impl ToDecimal for rust_decimal::Decimal {
    fn to_decimal(&self) -> rust_decimal::Decimal {
        *self
    }
}

#[cfg(feature = "rust_decimal")]
impl ToDecimal for f64 {
    fn to_decimal(&self) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from_f64_retain(*self).unwrap_or_else(|| {
            panic!("floating-point value `{self}` cannot be represented as a Decimal")
        })
    }
}

#[cfg(feature = "rust_decimal")]
impl ToDecimal for f32 {
    fn to_decimal(&self) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from_f32_retain(*self).unwrap_or_else(|| {
            panic!("floating-point value `{self}` cannot be represented as a Decimal")
        })
    }
}

// rust_decimal::Decimal implements From for all standard integer types
#[cfg(feature = "rust_decimal")]
impl ToDecimal for i8 {
    fn to_decimal(&self) -> rust_decimal::Decimal {
        (*self).into()
    }
}

#[cfg(feature = "rust_decimal")]
impl ToDecimal for i16 {
    fn to_decimal(&self) -> rust_decimal::Decimal {
        (*self).into()
    }
}

#[cfg(feature = "rust_decimal")]
impl ToDecimal for i32 {
    fn to_decimal(&self) -> rust_decimal::Decimal {
        (*self).into()
    }
}

#[cfg(feature = "rust_decimal")]
impl ToDecimal for i64 {
    fn to_decimal(&self) -> rust_decimal::Decimal {
        (*self).into()
    }
}

#[cfg(feature = "rust_decimal")]
impl ToDecimal for u8 {
    fn to_decimal(&self) -> rust_decimal::Decimal {
        (*self).into()
    }
}

#[cfg(feature = "rust_decimal")]
impl ToDecimal for u16 {
    fn to_decimal(&self) -> rust_decimal::Decimal {
        (*self).into()
    }
}

#[cfg(feature = "rust_decimal")]
impl ToDecimal for u32 {
    fn to_decimal(&self) -> rust_decimal::Decimal {
        (*self).into()
    }
}

#[cfg(feature = "rust_decimal")]
impl ToDecimal for u64 {
    fn to_decimal(&self) -> rust_decimal::Decimal {
        (*self).into()
    }
}

#[cfg(feature = "rust_decimal")]
impl ToDecimal for usize {
    fn to_decimal(&self) -> rust_decimal::Decimal {
        (*self as u64).into()
    }
}

#[cfg(feature = "rust_decimal")]
impl ToDecimal for isize {
    fn to_decimal(&self) -> rust_decimal::Decimal {
        (*self as i64).into()
    }
}

#[cfg(all(feature = "rust_decimal", feature = "spacetimedb"))]
impl ToDecimal for spacetimedb_lib::Timestamp {
    fn to_decimal(&self) -> rust_decimal::Decimal {
        self.to_micros_since_unix_epoch().into()
    }
}

#[cfg(all(feature = "rust_decimal", feature = "spacetimedb"))]
impl ToDecimal for spacetimedb_lib::TimeDuration {
    fn to_decimal(&self) -> rust_decimal::Decimal {
        self.to_micros().into()
    }
}

/// A civil date used by date-range filters.
///
/// Civil values and zoned datetimes keep their own calendar date. Bare
/// instants use UTC, independently of the time zone used to display them.
#[cfg(feature = "jiff")]
pub trait ToCivilDate {
    fn to_civil_date(&self) -> jiff::civil::Date;
}

#[cfg(feature = "jiff")]
impl ToCivilDate for jiff::civil::Date {
    fn to_civil_date(&self) -> jiff::civil::Date {
        *self
    }
}

#[cfg(feature = "jiff")]
impl ToCivilDate for jiff::civil::DateTime {
    fn to_civil_date(&self) -> jiff::civil::Date {
        self.date()
    }
}

#[cfg(feature = "jiff")]
impl ToCivilDate for jiff::ZonedDateTime {
    fn to_civil_date(&self) -> jiff::civil::Date {
        self.date()
    }
}

#[cfg(feature = "jiff")]
impl ToCivilDate for jiff::Timestamp {
    fn to_civil_date(&self) -> jiff::civil::Date {
        self.to_zoned_datetime(jiff::tz::TimeZone::UTC).date()
    }
}

#[cfg(feature = "spacetimedb")]
impl ToCivilDate for spacetimedb_lib::Timestamp {
    fn to_civil_date(&self) -> jiff::civil::Date {
        let micros = self.to_micros_since_unix_epoch();
        jiff::Timestamp::from_microsecond(micros)
            .unwrap_or_else(|error| {
                panic!("SpacetimeDB timestamp `{micros}` is outside Jiff's supported date range: {error}")
            })
            .to_civil_date()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "jiff")]
    use super::ToCivilDate as _;
    #[cfg(feature = "rust_decimal")]
    use super::ToDecimal as _;

    #[cfg(feature = "rust_decimal")]
    #[test]
    fn standard_numbers_convert_to_decimal_without_losing_their_value() {
        use rust_decimal::Decimal;

        assert_eq!(Decimal::new(125, 2).to_decimal(), Decimal::new(125, 2));
        assert_eq!(1.5_f64.to_decimal(), Decimal::from_f64_retain(1.5).unwrap());
        assert_eq!(2.5_f32.to_decimal(), Decimal::from_f32_retain(2.5).unwrap());
        assert_eq!((-8_i8).to_decimal(), Decimal::from(-8));
        assert_eq!((-16_i16).to_decimal(), Decimal::from(-16));
        assert_eq!((-32_i32).to_decimal(), Decimal::from(-32));
        assert_eq!((-64_i64).to_decimal(), Decimal::from(-64));
        assert_eq!(8_u8.to_decimal(), Decimal::from(8));
        assert_eq!(16_u16.to_decimal(), Decimal::from(16));
        assert_eq!(32_u32.to_decimal(), Decimal::from(32));
        assert_eq!(64_u64.to_decimal(), Decimal::from(64));
        assert_eq!(8_usize.to_decimal(), Decimal::from(8));
        assert_eq!((-8_isize).to_decimal(), Decimal::from(-8));
    }

    #[cfg(feature = "rust_decimal")]
    #[test]
    #[should_panic(expected = "cannot be represented as a Decimal")]
    fn non_finite_numbers_are_rejected_instead_of_becoming_zero() {
        _ = f64::NAN.to_decimal();
    }

    #[cfg(all(feature = "rust_decimal", feature = "spacetimedb"))]
    #[test]
    fn spacetimedb_time_values_convert_to_decimal_microseconds() {
        use rust_decimal::Decimal;
        use spacetimedb_lib::{TimeDuration, Timestamp};

        assert_eq!(
            Timestamp::from_micros_since_unix_epoch(42).to_decimal(),
            Decimal::from(42)
        );
        assert_eq!(
            TimeDuration::from_micros(-42).to_decimal(),
            Decimal::from(-42)
        );
    }

    #[cfg(feature = "jiff")]
    #[test]
    fn civil_and_zoned_values_keep_their_date_and_instants_use_utc() {
        use jiff::{Timestamp, ZonedDateTime, civil::Date};

        let date = Date::new(2026, 1, 1).unwrap();
        let datetime = date.at(0, 30, 0, 0);
        let zoned: ZonedDateTime = "2026-01-01T00:30:00+01:00[Europe/Paris]".parse().unwrap();
        let instant: Timestamp = "2025-12-31T23:30:00Z".parse().unwrap();

        assert_eq!(date.to_civil_date(), date);
        assert_eq!(datetime.to_civil_date(), date);
        assert_eq!(zoned.to_civil_date(), date);
        assert_eq!(zoned.timestamp(), instant);
        assert_eq!(instant.to_civil_date(), Date::new(2025, 12, 31).unwrap());
    }

    #[cfg(feature = "spacetimedb")]
    #[test]
    fn spacetimedb_timestamps_keep_signed_microseconds_at_the_utc_date_boundary() {
        use jiff::civil::Date;
        use spacetimedb_lib::Timestamp;

        for (micros, date) in [
            (-1, Date::new(1969, 12, 31).unwrap()),
            (0, Date::new(1970, 1, 1).unwrap()),
            (1, Date::new(1970, 1, 1).unwrap()),
        ] {
            assert_eq!(
                Timestamp::from_micros_since_unix_epoch(micros).to_civil_date(),
                date
            );
        }
    }

    #[cfg(feature = "spacetimedb")]
    #[test]
    #[should_panic(expected = "outside Jiff's supported date range")]
    fn out_of_range_spacetimedb_timestamps_are_rejected() {
        _ = spacetimedb_lib::Timestamp::from_micros_since_unix_epoch(i64::MAX).to_civil_date();
    }
}
