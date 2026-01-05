//! Conversion traits for filter matching.

/// Trait for converting numeric types to Decimal for range filter matching.
#[cfg(feature = "rust_decimal")]
pub trait IntoDecimal {
    fn into_decimal(&self) -> rust_decimal::Decimal;
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for rust_decimal::Decimal {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        *self
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for f64 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from_f64_retain(*self).unwrap_or(rust_decimal::Decimal::ZERO)
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for f32 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from_f32_retain(*self).unwrap_or(rust_decimal::Decimal::ZERO)
    }
}

// rust_decimal::Decimal implements From for all standard integer types
#[cfg(feature = "rust_decimal")]
impl IntoDecimal for i8 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        (*self).into()
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for i16 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        (*self).into()
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for i32 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        (*self).into()
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for i64 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        (*self).into()
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for u8 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        (*self).into()
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for u16 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        (*self).into()
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for u32 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        (*self).into()
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for u64 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        (*self).into()
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for usize {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        (*self as u64).into()
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for isize {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        (*self as i64).into()
    }
}

/// Trait for converting date/time types to NaiveDate for range filter matching.
#[cfg(feature = "chrono")]
pub trait IntoNaiveDate {
    fn into_naive_date(&self) -> chrono::NaiveDate;
}

#[cfg(feature = "chrono")]
impl IntoNaiveDate for chrono::NaiveDate {
    fn into_naive_date(&self) -> chrono::NaiveDate {
        *self
    }
}

#[cfg(feature = "chrono")]
impl<Tz: chrono::TimeZone> IntoNaiveDate for chrono::DateTime<Tz> {
    fn into_naive_date(&self) -> chrono::NaiveDate {
        self.date_naive()
    }
}

#[cfg(feature = "chrono")]
impl IntoNaiveDate for chrono::NaiveDateTime {
    fn into_naive_date(&self) -> chrono::NaiveDate {
        self.date()
    }
}
