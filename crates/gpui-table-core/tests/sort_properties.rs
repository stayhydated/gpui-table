use gpui_table_core::sort::{
    NullPlacement, SortClause, SortDirection, SortError, SortOrder, SortableRow, compare_values,
};
use proptest::prelude::*;
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq)]
struct Row {
    id: usize,
    category: u8,
    score: Option<i32>,
}
impl SortableRow for Row {
    fn sortable_columns() -> &'static [&'static str] {
        &["category", "score"]
    }
    fn compare_sort_clause(
        &self,
        other: &Self,
        clause: &SortClause,
    ) -> Result<Ordering, SortError> {
        match clause.column() {
            "category" => compare_values(Some(&self.category), Some(&other.category), clause),
            "score" => compare_values(self.score.as_ref(), other.score.as_ref(), clause),
            key => Err(SortError::UnsupportedColumn(key.into())),
        }
    }
    fn compare_sort_identity(&self, other: &Self) -> Result<Ordering, SortError> {
        Ok(self.id.cmp(&other.id))
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, ..ProptestConfig::default() })]
    #[test]
    fn multiple_keys_match_tuple_order_and_preserve_every_record(
        values in prop::collection::vec((0_u8..4, prop::option::of(-20_i32..20)), 0..128),
    ) {
        let rows: Vec<_> = values.into_iter().enumerate().map(|(id, (category, score))| Row { id, category, score }).collect();
        let mut expected = rows.clone();
        expected.sort_by_key(|row| (row.category, row.score.is_none(), std::cmp::Reverse(row.score), row.id));
        let order = SortOrder::new(vec![SortClause::new("category", SortDirection::Ascending), SortClause::new("score", SortDirection::Descending)]).unwrap();
        let mut actual = rows;
        order.sort_rows(&mut actual).unwrap();
        prop_assert_eq!(actual, expected);
    }
}

#[test]
fn null_placement_is_independent_of_direction() {
    for direction in [SortDirection::Ascending, SortDirection::Descending] {
        for nulls in [NullPlacement::First, NullPlacement::Last] {
            let clause = SortClause::new("score", direction).with_nulls(nulls);
            let expected = if nulls == NullPlacement::First {
                Ordering::Less
            } else {
                Ordering::Greater
            };
            assert_eq!(
                compare_values(None::<&i32>, Some(&1), &clause).unwrap(),
                expected
            );
            assert_eq!(
                compare_values(Some(&1), None, &clause).unwrap(),
                expected.reverse()
            );
        }
    }
}

#[test]
fn invalid_keys_are_rejected_on_an_empty_source() {
    let order = SortOrder::new(vec![SortClause::new("unknown", SortDirection::Ascending)]).unwrap();
    assert_eq!(
        order.sorted_indices::<Row>(&[]),
        Err(SortError::UnsupportedColumn("unknown".into()))
    );
    assert_eq!(
        SortOrder::new(vec![
            SortClause::new("score", SortDirection::Ascending),
            SortClause::new("score", SortDirection::Descending)
        ]),
        Err(SortError::DuplicateColumn("score".into()))
    );
}

#[test]
fn comparison_error_does_not_partially_reorder_the_source() {
    #[derive(Debug, PartialEq)]
    struct FloatRow(f64);
    impl SortableRow for FloatRow {
        fn sortable_columns() -> &'static [&'static str] {
            &["value"]
        }
        fn compare_sort_clause(
            &self,
            other: &Self,
            clause: &SortClause,
        ) -> Result<Ordering, SortError> {
            compare_values(Some(&self.0), Some(&other.0), clause)
        }
    }
    let mut rows = vec![FloatRow(3.), FloatRow(f64::NAN), FloatRow(1.)];
    let order = SortOrder::new(vec![SortClause::new("value", SortDirection::Ascending)]).unwrap();
    assert_eq!(
        order.sort_rows(&mut rows),
        Err(SortError::UnorderedColumn("value".into()))
    );
    assert_eq!(rows[0].0, 3.);
    assert!(rows[1].0.is_nan());
    assert_eq!(rows[2].0, 1.);
}
