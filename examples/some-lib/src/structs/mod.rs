#[cfg(feature = "client")]
pub mod context_menu_common;
#[cfg(feature = "client")]
pub mod item;
#[cfg(feature = "client")]
pub mod user;

#[cfg(feature = "client")]
struct TimestampFaker;

#[cfg(feature = "client")]
impl fake::Dummy<TimestampFaker> for jiff::Timestamp {
    fn dummy_with_rng<R: fake::RngExt + ?Sized>(_: &TimestampFaker, rng: &mut R) -> Self {
        // Match the old faker's signed Unix nanosecond domain and keep the caller's RNG.
        let nanoseconds: i64 = rng.random();
        Self::from_nanosecond(i128::from(nanoseconds)).expect("i64 epoch nanoseconds fit Jiff")
    }
}

#[cfg(all(feature = "client", test))]
mod tests {
    use super::TimestampFaker;
    use fake::rand::{SeedableRng as _, rngs::StdRng};
    use fake::{Fake as _, RngExt as _};

    #[test]
    fn timestamp_faker_uses_the_supplied_signed_nanosecond_sequence() {
        let mut raw = StdRng::seed_from_u64(73);
        let mut timestamps = StdRng::seed_from_u64(73);
        for _ in 0..16 {
            let expected: i64 = raw.random();
            let actual: jiff::Timestamp = TimestampFaker.fake_with_rng(&mut timestamps);
            assert_eq!(actual.as_nanosecond(), i128::from(expected));
        }
    }
}
