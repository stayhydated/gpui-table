#![cfg(all(feature = "spacetimedb", feature = "rust_decimal", feature = "mcp"))]

use gpui_table::{
    GpuiTable,
    mcp::{McpServer, McpTable as _, McpToolCall, table},
    sort::{SortClause, SortDirection, SortOrder},
};
use serde::Serialize;
use serde_json::json;
use spacetimedb_lib::{TimeDuration, Timestamp};

#[derive(Clone, GpuiTable, Serialize)]
#[gpui_table(row_id = "id", filters, mcp(name = "query_temporal_records"))]
struct TemporalRecord {
    #[gpui_table(skip)]
    id: u64,
    #[gpui_table(sortable, filter(gpui_table_component::TextFilter))]
    category: String,
    #[gpui_table(sortable, filter(gpui_table_component::DateRangeFilter))]
    #[serde(serialize_with = "serialize_timestamp")]
    created: Timestamp,
    #[gpui_table(sortable, filter(gpui_table_component::NumberRangeFilter))]
    #[serde(serialize_with = "serialize_duration")]
    elapsed: TimeDuration,
    #[gpui_table(sortable, sort_key = remaining_score)]
    score: i64,
}
fn remaining_score(row: &TemporalRecord) -> Option<i64> {
    row.score.checked_sub(row.elapsed.to_micros())
}
fn records() -> Vec<TemporalRecord> {
    [
        (3, "a", 1, 7),
        (2, "b", 0, 20),
        (1, "a", 1, 7),
        (4, "a", 0, 10),
    ]
    .into_iter()
    .map(|(id, category, day, elapsed)| TemporalRecord {
        id,
        category: category.into(),
        created: Timestamp::from_micros_since_unix_epoch(day * 86_400_000_000),
        elapsed: TimeDuration::from_micros(elapsed),
        score: 100,
    })
    .collect()
}
fn order_json() -> serde_json::Value {
    json!([{"column":"category","direction":"ascending"}, {"column":"created","direction":"ascending"}, {"column":"elapsed","direction":"descending"}])
}

#[test]
fn real_temporal_types_agree_between_generated_delegate_and_mcp_pagination() {
    let order: SortOrder = serde_json::from_value(order_json()).unwrap();
    let mut delegate = TemporalRecordTableDelegate::new(records());
    delegate.set_ordering(order.clone()).unwrap();
    assert_eq!(
        delegate
            .visible_row_indices()
            .iter()
            .map(|ix| delegate.rows[*ix].id)
            .collect::<Vec<_>>(),
        [4, 1, 3, 2]
    );
    let mut server = McpServer::new("temporal-test", "1");
    table::<TemporalRecord>(&mut server)
        .rows(records())
        .unwrap();
    let result = server.call_tool("query_temporal_records", Some(json!({
        "category":"a", "created":{"min":"1970-01-01", "max":"1970-01-02"}, "elapsed":{"min":"7","max":"10"}, "sort":order_json(), "offset":1,"limit":2
    })));
    assert_eq!(result.is_error, Some(false), "{result:?}");
    let content = result.structured_content.unwrap();
    assert_eq!(content["total"], 3);
    assert_eq!(
        content["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["id"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [1, 3]
    );
    assert_eq!(content["offset"], 1);
    assert_eq!(content["limit"], 2);

    let query = TemporalRecord::decode_query(
        McpToolCall::from_value(Some(json!({"sort":order_json()}))).unwrap(),
    )
    .unwrap();
    assert_eq!(query.ordering, order);
    let schema = TemporalRecord::descriptor().input_schema();
    assert_eq!(
        schema["properties"]["sort"]["items"]["properties"]["column"]["enum"],
        json!(["category", "created", "elapsed", "score"])
    );
}

#[test]
fn calculated_duration_keys_recompute_and_signed_temporal_order_is_exact() {
    let mut rows = records();
    rows[0].elapsed = TimeDuration::from_micros(-5);
    rows[0].created = Timestamp::from_micros_since_unix_epoch(-1);
    let mut delegate = TemporalRecordTableDelegate::new(rows);
    let order = SortOrder::new(vec![SortClause::new("score", SortDirection::Descending)]).unwrap();
    delegate.set_ordering(order).unwrap();
    assert_eq!(
        delegate
            .visible_row_indices()
            .iter()
            .map(|ix| delegate.rows[*ix].id)
            .collect::<Vec<_>>(),
        [3, 1, 4, 2]
    );
    delegate.rows[0].elapsed = TimeDuration::from_micros(50);
    delegate.refresh_filtered_rows();
    assert_eq!(
        delegate
            .visible_row_indices()
            .iter()
            .map(|ix| delegate.rows[*ix].id)
            .collect::<Vec<_>>(),
        [1, 4, 2, 3]
    );
    delegate
        .set_ordering(
            SortOrder::new(vec![SortClause::new("created", SortDirection::Ascending)]).unwrap(),
        )
        .unwrap();
    assert_eq!(
        delegate
            .visible_row_indices()
            .iter()
            .map(|ix| delegate.rows[*ix].id)
            .collect::<Vec<_>>(),
        [3, 2, 4, 1]
    );
}

#[test]
fn invalid_ordering_is_rejected_before_a_custom_backend_runs() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let backend_calls = calls.clone();
    let mut server = McpServer::new("temporal-test", "1");
    table::<TemporalRecord>(&mut server)
        .query(move |query| {
            backend_calls.fetch_add(1, Ordering::SeqCst);
            query.filter_rows(records())
        })
        .unwrap();
    for sort in [
        json!([{"column":"id","direction":"ascending"}]),
        json!([{"column":"created","direction":"sideways"}]),
        json!([{"column":"created","direction":"ascending","extra":true}]),
        json!([{"column":"created","direction":"ascending"},{"column":"created","direction":"descending"}]),
        json!(null),
    ] {
        let result = server.call_tool("query_temporal_records", Some(json!({"sort":sort})));
        assert_eq!(result.is_error, Some(true));
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let result = server.call_tool(
        "query_temporal_records",
        Some(json!({"sort":order_json(),"limit":1})),
    );
    assert_eq!(result.is_error, Some(false));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(result.structured_content.unwrap()["rows"][0]["id"], 4);
}

fn serialize_timestamp<S: serde::Serializer>(
    value: &Timestamp,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_i64(value.to_micros_since_unix_epoch())
}
fn serialize_duration<S: serde::Serializer>(
    value: &TimeDuration,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_i64(value.to_micros())
}
