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

/// Trait for converting date/time types to NaiveDate for range filter matching.
#[cfg(feature = "chrono")]
pub trait ToNaiveDate {
    fn to_naive_date(&self) -> chrono::NaiveDate;
}

#[cfg(feature = "chrono")]
impl ToNaiveDate for chrono::NaiveDate {
    fn to_naive_date(&self) -> chrono::NaiveDate {
        *self
    }
}

#[cfg(feature = "chrono")]
impl<Tz: chrono::TimeZone> ToNaiveDate for chrono::DateTime<Tz> {
    fn to_naive_date(&self) -> chrono::NaiveDate {
        self.date_naive()
    }
}

#[cfg(feature = "chrono")]
impl ToNaiveDate for chrono::NaiveDateTime {
    fn to_naive_date(&self) -> chrono::NaiveDate {
        self.date()
    }
}

#[cfg(all(feature = "chrono", feature = "spacetimedb"))]
impl ToNaiveDate for spacetimedb_lib::Timestamp {
    fn to_naive_date(&self) -> chrono::NaiveDate {
        chrono::DateTime::from_timestamp_micros(self.to_micros_since_unix_epoch())
            .map(|value| value.date_naive())
            .unwrap_or_else(|| {
                panic!(
                    "SpacetimeDB timestamp `{}` is outside chrono's supported date range",
                    self.to_micros_since_unix_epoch()
                )
            })
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "rust_decimal")]
    use super::ToDecimal as _;
    #[cfg(feature = "chrono")]
    use super::ToNaiveDate as _;

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

    #[cfg(feature = "chrono")]
    #[test]
    fn chrono_values_convert_to_their_calendar_date() {
        use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

        let date = NaiveDate::from_ymd_opt(2026, 7, 11).unwrap();
        let datetime = date.and_hms_opt(23, 59, 58).unwrap();
        let zoned = DateTime::<Utc>::from_naive_utc_and_offset(datetime, Utc);

        assert_eq!(date.to_naive_date(), date);
        assert_eq!(NaiveDateTime::to_naive_date(&datetime), date);
        assert_eq!(zoned.to_naive_date(), date);
    }

    #[cfg(all(feature = "chrono", feature = "spacetimedb"))]
    #[test]
    fn spacetimedb_timestamps_convert_to_dates() {
        use chrono::NaiveDate;
        use spacetimedb_lib::Timestamp;

        assert_eq!(
            Timestamp::from_micros_since_unix_epoch(0).to_naive_date(),
            NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
        );
    }

    #[cfg(all(feature = "chrono", feature = "spacetimedb"))]
    #[test]
    #[should_panic(expected = "outside chrono's supported date range")]
    fn out_of_range_spacetimedb_timestamps_are_rejected() {
        use spacetimedb_lib::Timestamp;

        _ = Timestamp::from_micros_since_unix_epoch(i64::MAX).to_naive_date();
    }
}
