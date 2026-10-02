use gpui_table_core::filter::Matchable;
use gpui_table_mcp::{
    McpTable, McpTableDescriptor, McpToolCall, McpToolError, McpToolMetadata, TableQuery,
};
use gpui_table_schema::registry::RustPath;
use proptest::prelude::*;

#[derive(Debug, PartialEq)]
struct Row {
    id: usize,
    selected: bool,
}

impl Matchable<bool> for Row {
    fn matches_filters(&self, require_selected: &bool) -> bool {
        !require_selected || self.selected
    }
}

impl McpTable for Row {
    type FilterValues = bool;

    fn descriptor() -> McpTableDescriptor {
        McpTableDescriptor::new(
            "Row",
            "rows",
            "Rows",
            RustPath::from_macro_tokens_unchecked("query_properties"),
            &[],
            McpToolMetadata::new(),
        )
    }

    fn decode_query(_: McpToolCall) -> Result<TableQuery<Self>, McpToolError> {
        Ok(TableQuery {
            filters: false,
            limit: None,
            offset: 0,
        })
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, ..ProptestConfig::default() })]

    #[test]
    fn pagination_preserves_order_and_total_across_match_patterns(
        selections in prop::collection::vec(any::<bool>(), 0..256),
        require_selected in any::<bool>(),
        offset in prop_oneof![0_usize..=257, Just(usize::MAX)],
        limit in prop::option::of(0_usize..=257),
    ) {
        // Materialize all matching IDs first, independently of the streaming page loop.
        let matching: Vec<_> = if require_selected {
            selections.iter().enumerate()
                .filter_map(|(id, selected)| selected.then_some(id))
                .collect()
        } else {
            (0..selections.len()).collect()
        };
        let expected: Vec<_> = matching.iter().copied()
            .skip(offset).take(limit.unwrap_or(usize::MAX)).collect();
        let rows = selections.into_iter().enumerate()
            .map(|(id, selected)| Row { id, selected });
        let result = TableQuery { filters: require_selected, limit, offset }.filter_rows(rows);

        prop_assert_eq!(result.rows.iter().map(|row| row.id).collect::<Vec<_>>(), expected);
        prop_assert_eq!(result.total, matching.len());
        prop_assert_eq!(result.offset, offset);
        prop_assert_eq!(result.limit, limit);
    }
}

#[test]
fn zero_limit_and_out_of_range_offsets_still_count_all_matches() {
    for (offset, limit) in [(0, Some(0)), (usize::MAX, None), (3, Some(2))] {
        let rows = [false, true, false, true, true]
            .into_iter()
            .enumerate()
            .map(|(id, selected)| Row { id, selected });
        let result = TableQuery {
            filters: true,
            limit,
            offset,
        }
        .filter_rows(rows);
        assert!(result.rows.is_empty());
        assert_eq!(result.total, 3);
    }
}
