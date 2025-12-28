//! Tests for multi-filter functionality.
//!
//! These tests verify that filtering works correctly when multiple filters
//! are applied simultaneously.

use std::collections::HashSet;

use gpui_table::filter_helpers::{facet_matches, number_in_range};
use gpui_table::{Filterable, GpuiTable};
use strum::Display;

/// Test category enum for faceted filtering
#[derive(Clone, Debug, PartialEq, Filterable, Display)]
pub enum TestCategory {
    Engineering,
    Marketing,
    Sales,
}

impl gpui_table::TableCell for TestCategory {
    fn draw(&self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> gpui::AnyElement {
        use gpui::IntoElement;
        self.to_string().into_any_element()
    }
}

/// A test struct with multiple filterable fields
#[derive(Clone, Debug, GpuiTable)]
pub struct TestRow {
    #[gpui_table(filter(text()))]
    pub name: String,

    #[gpui_table(filter(text()))]
    pub email: String,

    #[gpui_table(filter(number_range()))]
    pub age: u8,

    #[gpui_table(filter(number_range()))]
    pub score: u32,

    #[gpui_table(filter(faceted()))]
    pub active: bool,

    #[gpui_table(filter(faceted()))]
    pub category: TestCategory,
}

impl TestRow {
    fn new(
        name: &str,
        email: &str,
        age: u8,
        score: u32,
        active: bool,
        category: TestCategory,
    ) -> Self {
        Self {
            name: name.to_string(),
            email: email.to_string(),
            age,
            score,
            active,
            category,
        }
    }
}

fn create_test_data() -> Vec<TestRow> {
    vec![
        TestRow::new(
            "Alice",
            "alice@example.com",
            25,
            85,
            true,
            TestCategory::Engineering,
        ),
        TestRow::new("Bob", "bob@test.org", 30, 92, true, TestCategory::Marketing),
        TestRow::new(
            "Charlie",
            "charlie@example.com",
            35,
            78,
            false,
            TestCategory::Engineering,
        ),
        TestRow::new("Diana", "diana@test.org", 28, 95, true, TestCategory::Sales),
        TestRow::new(
            "Eve",
            "eve@example.com",
            22,
            88,
            false,
            TestCategory::Marketing,
        ),
    ]
}

// ============================================================================
// Single Filter Tests
// ============================================================================

#[test]
fn test_single_text_filter_name() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // No filter - all rows match
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        5
    );

    // Filter by name containing "a" (case-insensitive)
    filters.name = "a".to_string();
    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    assert_eq!(matches.len(), 3); // Alice, Charlie, Diana
    assert!(matches.iter().any(|r| r.name == "Alice"));
    assert!(matches.iter().any(|r| r.name == "Charlie"));
    assert!(matches.iter().any(|r| r.name == "Diana"));
}

#[test]
fn test_single_text_filter_email() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Filter by email domain
    filters.email = "example.com".to_string();
    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    assert_eq!(matches.len(), 3); // Alice, Charlie, Eve
}

#[test]
fn test_single_number_range_filter() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Filter by age range (25-30)
    filters.age = (Some(25.0), Some(30.0));
    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    assert_eq!(matches.len(), 3); // Alice (25), Bob (30), Diana (28)
}

#[test]
fn test_single_faceted_filter_bool() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Filter by active = true
    filters.active = HashSet::from([true.to_string()]);
    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    assert_eq!(matches.len(), 3); // Alice, Bob, Diana
}

#[test]
fn test_single_faceted_filter_enum() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Filter by category = Engineering
    filters.category = HashSet::from([TestCategory::Engineering.to_string()]);
    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    assert_eq!(matches.len(), 2); // Alice, Charlie
}

// ============================================================================
// Multi-Filter Tests (the main focus)
// ============================================================================

#[test]
fn test_two_text_filters() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Filter by name containing "a" AND email containing "example"
    filters.name = "a".to_string();
    filters.email = "example".to_string();

    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    // Alice (name has 'a', email has 'example')
    // Charlie (name has 'a', email has 'example')
    // Diana has 'a' but email is test.org, so excluded
    assert_eq!(matches.len(), 2);
    assert!(matches.iter().any(|r| r.name == "Alice"));
    assert!(matches.iter().any(|r| r.name == "Charlie"));
}

#[test]
fn test_text_and_number_range_filter() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Filter by name containing "a" AND age >= 25
    filters.name = "a".to_string();
    filters.age = (Some(25.0), None);

    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    // Alice (25), Charlie (35), Diana (28) - all have 'a' in name and age >= 25
    assert_eq!(matches.len(), 3);
}

#[test]
fn test_text_and_faceted_filter() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Filter by email containing "example" AND active = true
    filters.email = "example".to_string();
    filters.active = HashSet::from([true.to_string()]);

    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    // Alice (example.com, active=true)
    // Charlie has example.com but active=false
    // Eve has example.com but active=false
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].name, "Alice");
}

#[test]
fn test_two_number_range_filters() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Filter by age 25-30 AND score >= 85
    filters.age = (Some(25.0), Some(30.0));
    filters.score = (Some(85.0), None);

    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    // Alice (25, 85) - matches both
    // Bob (30, 92) - matches both
    // Diana (28, 95) - matches both
    assert_eq!(matches.len(), 3);
}

#[test]
fn test_two_faceted_filters() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Filter by active = true AND category = Engineering
    filters.active = HashSet::from([true.to_string()]);
    filters.category = HashSet::from([TestCategory::Engineering.to_string()]);

    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    // Only Alice (active=true, Engineering)
    // Charlie is Engineering but not active
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].name, "Alice");
}

#[test]
fn test_three_filters_combined() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Filter by name containing "a" AND age >= 25 AND active = true
    filters.name = "a".to_string();
    filters.age = (Some(25.0), None);
    filters.active = HashSet::from([true.to_string()]);

    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    // Alice (name has 'a', 25, active)
    // Diana (name has 'a', 28, active)
    // Charlie has 'a' and is 35 but NOT active
    assert_eq!(matches.len(), 2);
    assert!(matches.iter().any(|r| r.name == "Alice"));
    assert!(matches.iter().any(|r| r.name == "Diana"));
}

#[test]
fn test_all_filters_combined() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Apply all filters at once
    filters.name = "alice".to_string();
    filters.email = "example".to_string();
    filters.age = (Some(20.0), Some(30.0));
    filters.score = (Some(80.0), Some(90.0));
    filters.active = HashSet::from([true.to_string()]);
    filters.category = HashSet::from([TestCategory::Engineering.to_string()]);

    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    // Only Alice matches all criteria
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].name, "Alice");
}

#[test]
fn test_no_matches_with_multi_filter() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Filter that matches nobody
    filters.name = "xyz".to_string();
    filters.active = HashSet::from([true.to_string()]);

    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    assert_eq!(matches.len(), 0);
}

// ============================================================================
// Filter Reset Tests
// ============================================================================

#[test]
fn test_filter_reset_to_empty() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Apply a filter
    filters.name = "alice".to_string();
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        1
    );

    // Reset to empty
    filters.name = String::new();
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        5
    );
}

#[test]
fn test_filter_sequential_changes() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Start with all rows
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        5
    );

    // Add first filter
    filters.name = "a".to_string();
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        3
    );

    // Add second filter (should further narrow)
    filters.active = HashSet::from([true.to_string()]);
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        2
    );

    // Add third filter
    filters.category = HashSet::from([TestCategory::Engineering.to_string()]);
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        1
    );

    // Remove first filter (should widen)
    filters.name = String::new();
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        1
    ); // Still just Alice

    // Remove all filters
    filters.active = HashSet::new();
    filters.category = HashSet::new();
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        5
    );
}

// ============================================================================
// Filter Helper Function Tests
// ============================================================================

#[test]
fn test_number_in_range_helper() {
    // No range - always matches
    assert!(number_in_range(&25u8, &(None, None)));

    // Min only
    assert!(number_in_range(&25u8, &(Some(20.0), None)));
    assert!(!number_in_range(&15u8, &(Some(20.0), None)));

    // Max only
    assert!(number_in_range(&25u8, &(None, Some(30.0))));
    assert!(!number_in_range(&35u8, &(None, Some(30.0))));

    // Both min and max
    assert!(number_in_range(&25u8, &(Some(20.0), Some(30.0))));
    assert!(!number_in_range(&15u8, &(Some(20.0), Some(30.0))));
    assert!(!number_in_range(&35u8, &(Some(20.0), Some(30.0))));

    // Edge cases - inclusive bounds
    assert!(number_in_range(&20u8, &(Some(20.0), Some(30.0))));
    assert!(number_in_range(&30u8, &(Some(20.0), Some(30.0))));
}

#[test]
fn test_facet_matches_helper() {
    // Empty filter - always matches
    assert!(facet_matches(&TestCategory::Engineering, &HashSet::new()));

    // Single value in filter
    let filter = HashSet::from([TestCategory::Engineering.to_string()]);
    assert!(facet_matches(&TestCategory::Engineering, &filter));
    assert!(!facet_matches(&TestCategory::Marketing, &filter));

    // Multiple values in filter (OR logic within facet)
    let filter = HashSet::from([
        TestCategory::Engineering.to_string(),
        TestCategory::Marketing.to_string(),
    ]);
    assert!(facet_matches(&TestCategory::Engineering, &filter));
    assert!(facet_matches(&TestCategory::Marketing, &filter));
    assert!(!facet_matches(&TestCategory::Sales, &filter));
}

#[test]
fn test_facet_matches_with_multiple_selected() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Filter by multiple categories (should OR within facet)
    filters.category = HashSet::from([
        TestCategory::Engineering.to_string(),
        TestCategory::Marketing.to_string(),
    ]);

    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    // Alice (Engineering), Bob (Marketing), Charlie (Engineering), Eve (Marketing)
    assert_eq!(matches.len(), 4);
    assert!(!matches.iter().any(|r| r.name == "Diana")); // Sales
}
