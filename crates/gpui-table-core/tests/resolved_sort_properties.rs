use gpui_table_core::sort::{
    NullPlacement, ResolvedSortColumn, ResolvedSortColumns, SortClause, SortDirection, SortError,
    SortOrder, SortableRow, compare_values,
};
use proptest::prelude::*;
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    id: usize,
    value: Option<i64>,
}

impl SortableRow for Row {
    fn sortable_columns() -> &'static [&'static str] {
        &["native"]
    }

    fn compare_sort_clause(
        &self,
        other: &Self,
        clause: &SortClause,
    ) -> Result<Ordering, SortError> {
        compare_values(self.value.as_ref(), other.value.as_ref(), clause)
    }

    fn compare_sort_identity(&self, other: &Self) -> Result<Ordering, SortError> {
        Ok(self.id.cmp(&other.id))
    }
}

proptest! {
    #[test]
    fn resolved_and_native_keys_agree_for_subsets_and_complete_sources(
        values in prop::collection::vec(prop::option::of(-100i64..=100), 0..64),
        descending in any::<bool>(),
        nulls_first in any::<bool>(),
    ) {
        let rows = values.into_iter().enumerate().map(|(id, value)| Row { id, value }).collect::<Vec<_>>();
        let direction = if descending { SortDirection::Descending } else { SortDirection::Ascending };
        let nulls = if nulls_first { NullPlacement::First } else { NullPlacement::Last };
        let native_order = SortOrder::new(vec![SortClause::new("native", direction).with_nulls(nulls)]).unwrap();
        let resolved_order = SortOrder::new(vec![SortClause::new("resolved", direction).with_nulls(nulls)]).unwrap();
        let context = ResolvedSortColumns::new([
            ResolvedSortColumn::new("resolved", |row: &Row| Ok(row.value)).unwrap(),
        ]).unwrap();
        let mut native = (0..rows.len()).filter(|index| index % 2 == 0).collect::<Vec<_>>();
        let mut resolved = native.clone();
        native_order.sort_indices(&rows, &mut native).unwrap();
        context.sort_indices(&resolved_order, &rows, &mut resolved).unwrap();
        prop_assert_eq!(resolved, native);
        let mut native_rows = rows.clone();
        let mut resolved_rows = rows;
        native_order.sort_rows(&mut native_rows).unwrap();
        context.sort_rows(&resolved_order, &mut resolved_rows).unwrap();
        prop_assert_eq!(resolved_rows, native_rows);
    }
}
