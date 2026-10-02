use gpui_table_core::filter::{RangeValue, SingleValue, TextValue};
use proptest::prelude::*;

fn ascii_text() -> impl Strategy<Value = String> {
    prop::collection::vec((b'a'..=b'z', any::<bool>()), 0..128).prop_map(|letters| {
        letters
            .into_iter()
            .map(|(letter, uppercase)| {
                char::from(if uppercase {
                    letter.to_ascii_uppercase()
                } else {
                    letter
                })
            })
            .collect()
    })
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, ..ProptestConfig::default() })]

    #[test]
    fn ranges_match_the_standard_inclusive_interval(
        min in prop::option::of(-1_000_i32..=1_000),
        max in prop::option::of(-1_000_i32..=1_000),
        value in -1_001_i32..=1_001,
    ) {
        let filter = RangeValue(min, max);
        let interval = min.unwrap_or(i32::MIN)..=max.unwrap_or(i32::MAX);
        prop_assert_eq!(filter.matches(&value), interval.contains(&value));
        prop_assert_eq!(filter.is_active(), min.is_some() || max.is_some());
        prop_assert_eq!(filter.min(), min.as_ref());
        prop_assert_eq!(filter.max(), max.as_ref());
    }

    #[test]
    fn single_selection_exposes_exactly_its_selected_value(
        selected in prop::option::of(any::<i32>()),
        probe in any::<i32>(),
        use_selection in any::<bool>(),
    ) {
        let candidate = if use_selection { selected.unwrap_or(probe) } else { probe };
        let filter = SingleValue(selected);
        let items: Vec<_> = filter.iter().copied().collect();
        prop_assert_eq!(&items, &selected.into_iter().collect::<Vec<_>>());
        prop_assert_eq!(filter.len(), items.len());
        prop_assert_eq!(filter.is_active(), !items.is_empty());
        prop_assert_eq!(filter.matches(&candidate), items.is_empty() || items.contains(&candidate));
    }

    #[test]
    fn ascii_text_matches_an_independent_byte_window_model(
        haystack in ascii_text(),
        needle in ascii_text(),
    ) {
        // ASCII byte comparisons avoid reusing the production lowercase algorithm.
        let expected = needle.is_empty() || haystack.as_bytes()
            .windows(needle.len())
            .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()));
        let filter = TextValue(needle);
        prop_assert_eq!(filter.matches(&haystack), expected);
    }

    #[test]
    fn embedded_ascii_needles_match_even_with_opposite_case(
        prefix in ascii_text(),
        letters in prop::collection::vec(b'a'..=b'z', 1..32),
        suffix in ascii_text(),
    ) {
        let needle: String = letters.into_iter().map(char::from).collect();
        let haystack = format!("{prefix}{}{suffix}", needle.to_ascii_uppercase());
        prop_assert!(TextValue(needle).matches(&haystack));
    }
}

#[test]
fn unicode_matching_preserves_lowercase_semantics_without_normalization() {
    assert!(TextValue::from("İ").matches("i\u{307}"));
    assert!(!TextValue::from("é").matches("e\u{301}"));
}

#[test]
fn reversed_ranges_and_nan_follow_the_partial_order_contract() {
    assert!(!RangeValue(Some(2), Some(1)).matches(&1));
    assert!(RangeValue::<f64>(None, None).matches(&f64::NAN));
    assert!(!RangeValue(Some(0.0), None).matches(&f64::NAN));
}
