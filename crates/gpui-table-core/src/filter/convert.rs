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
        rust_decimal::Decimal::from_f64_retain(*self).unwrap_or(rust_decimal::Decimal::ZERO)
    }
}

#[cfg(feature = "rust_decimal")]
impl ToDecimal for f32 {
    fn to_decimal(&self) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from_f32_retain(*self).unwrap_or(rust_decimal::Decimal::ZERO)
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
