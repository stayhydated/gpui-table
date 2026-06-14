use chrono::NaiveDate;
use gpui_table::{Filterable, GpuiTable, TableCell};
use serde::Serialize;

#[derive(Clone, Debug, Eq, Filterable, Hash, PartialEq, Serialize, TableCell)]
enum IssueState {
    Open,
    InReview,
    Closed,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(
    id = "issues",
    title = "Issues",
    filters,
    mcp(
        name = "mcp_query_issues",
        title = "Query issues",
        description = "Query in-memory issues with generated table filters."
    )
)]
struct IssueRow {
    #[gpui_table(width = 80, filter(gpui_table::runtime::shape::NumberRangeFilter))]
    id: u32,

    #[gpui_table(width = 220, filter(gpui_table::runtime::shape::TextFilter))]
    title: String,

    #[gpui_table(width = 120, filter(gpui_table::runtime::shape::FacetedFilter::<IssueState>))]
    state: IssueState,

    #[gpui_table(width = 120, filter(gpui_table::runtime::shape::DateRangeFilter))]
    updated_on: NaiveDate,
}

fn main() -> gpui_table::mcp::ServeStdioResult {
    gpui_table::mcp::serve_stdio_blocking()
}

#[gpui_table::mcp_query]
fn rows() -> Result<Vec<IssueRow>, String> {
    Ok(vec![
        IssueRow {
            id: 101,
            title: "Add table MCP query bridge".to_string(),
            state: IssueState::InReview,
            updated_on: date(2026, 6, 9),
        },
        IssueRow {
            id: 102,
            title: "Document filter facets".to_string(),
            state: IssueState::Open,
            updated_on: date(2026, 6, 7),
        },
        IssueRow {
            id: 103,
            title: "Retire old query parameter glue".to_string(),
            state: IssueState::Closed,
            updated_on: date(2026, 5, 30),
        },
    ])
}

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("example date should be valid")
}
