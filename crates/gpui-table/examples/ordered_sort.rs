//! Typed temporal ordering over a complete source, with the same key used for display.
use gpui_kit::{AnyElement, App, Window};
use gpui_table::{
    GpuiTable, TableCell as _,
    mcp::{McpTable as _, McpToolCall, serde_json::json},
    runtime::FormattedCell,
    sort::SortOrder,
};
use spacetimedb_lib::{TimeDuration, Timestamp};

#[derive(Clone, GpuiTable)]
#[gpui_table(row_id = "id", mcp)]
struct Record {
    #[gpui_table(skip)]
    id: u64,
    #[gpui_table(sortable)]
    created: Timestamp,
    #[gpui_table(sortable, filter(gpui_table_component::NumberRangeFilter))]
    elapsed: TimeDuration,
    #[gpui_table(sortable, sort_key = remaining, style = remaining_cell)]
    score: i64,
}
fn remaining(row: &Record) -> Option<i64> {
    row.score.checked_sub(row.elapsed.to_micros())
}
fn remaining_cell(row: &Record, _: &i64, window: &mut Window, cx: &mut App) -> AnyElement {
    FormattedCell::new(remaining(row), |value: &Option<i64>| {
        value.map_or_else(|| "—".into(), |value| value.to_string())
    })
    .draw(window, cx)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rows = [(3, 10), (2, 20), (1, 10)]
        .into_iter()
        .map(|(id, elapsed)| Record {
            id,
            created: Timestamp::from_micros_since_unix_epoch(0),
            elapsed: TimeDuration::from_micros(elapsed),
            score: 100,
        })
        .collect::<Vec<_>>();
    let arguments = json!({ "sort":[{"column":"score","direction":"descending"},{"column":"created","direction":"ascending"}], "offset":1,"limit":1 });
    let query = Record::decode_query(McpToolCall::from_value(Some(arguments))?)?;
    let order: SortOrder = query.ordering.clone();
    let mut delegate = RecordTableDelegate::new(rows.clone());
    delegate.set_ordering(order)?;
    assert_eq!(
        delegate
            .visible_row_indices()
            .iter()
            .map(|ix| delegate.rows[*ix].id)
            .collect::<Vec<_>>(),
        [1, 3, 2]
    );
    let page = query.filter_rows(rows)?;
    assert_eq!(page.total, 3);
    assert_eq!(page.rows[0].id, 3);
    println!(
        "Temporal ordering verified: total={}, page id={}",
        page.total, page.rows[0].id
    );
    Ok(())
}
