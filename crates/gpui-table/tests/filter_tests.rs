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

// ============================================================================
// Text Filter Edge Cases
// ============================================================================

#[test]
fn test_text_filter_case_insensitive() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Uppercase should match lowercase
    filters.name = "ALICE".to_string();
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        1
    );

    // Mixed case
    filters.name = "AlIcE".to_string();
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        1
    );
}

#[test]
fn test_text_filter_partial_match() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Single character
    filters.name = "b".to_string();
    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].name, "Bob");

    // Middle of string
    filters.name = "harl".to_string();
    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].name, "Charlie");
}

#[test]
fn test_text_filter_whitespace() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Whitespace-only filter should match nothing (or everything depending on impl)
    filters.name = "   ".to_string();
    // Current impl: whitespace doesn't match any names
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        0
    );
}

#[test]
fn test_text_filter_special_characters() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // @ symbol in email
    filters.email = "@".to_string();
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        5
    );

    // . in email
    filters.email = ".com".to_string();
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        3
    );
}

// ============================================================================
// Number Range Edge Cases
// ============================================================================

#[test]
fn test_number_range_exact_value() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Exact match (min == max)
    filters.age = (Some(25.0), Some(25.0));
    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].name, "Alice");
}

#[test]
fn test_number_range_zero() {
    // Test with zero values
    assert!(number_in_range(&0u8, &(None, None)));
    assert!(number_in_range(&0u8, &(Some(0.0), None)));
    assert!(number_in_range(&0u8, &(None, Some(0.0))));
    assert!(number_in_range(&0u8, &(Some(0.0), Some(0.0))));
    assert!(!number_in_range(&0u8, &(Some(1.0), None)));
}

#[test]
fn test_number_range_negative() {
    // Test with negative numbers (using i32)
    assert!(number_in_range(&-5i32, &(Some(-10.0), Some(0.0))));
    assert!(!number_in_range(&-15i32, &(Some(-10.0), Some(0.0))));
    assert!(number_in_range(&-10i32, &(Some(-10.0), None)));
}

#[test]
fn test_number_range_large_values() {
    // Test with large numbers
    assert!(number_in_range(
        &1_000_000u64,
        &(Some(999_999.0), Some(1_000_001.0))
    ));
    assert!(number_in_range(&u32::MAX, &(None, None)));
}

#[test]
fn test_number_range_float_precision() {
    // Test floating point values
    assert!(number_in_range(&3.14f64, &(Some(3.0), Some(4.0))));
    assert!(number_in_range(&0.001f64, &(Some(0.0), Some(0.01))));
}

// ============================================================================
// Faceted Filter Edge Cases
// ============================================================================

#[test]
fn test_faceted_filter_bool_false() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Specifically filter for inactive users
    filters.active = HashSet::from([false.to_string()]);
    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    assert_eq!(matches.len(), 2); // Charlie, Eve
    assert!(matches.iter().all(|r| !r.active));
}

#[test]
fn test_faceted_filter_all_options_selected() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Select all categories - should match all
    filters.category = HashSet::from([
        TestCategory::Engineering.to_string(),
        TestCategory::Marketing.to_string(),
        TestCategory::Sales.to_string(),
    ]);
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        5
    );
}

#[test]
fn test_faceted_filter_both_bool_values() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Select both true and false - should match all
    filters.active = HashSet::from([true.to_string(), false.to_string()]);
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        5
    );
}

// ============================================================================
// Empty/Minimal Dataset Tests
// ============================================================================

#[test]
fn test_filter_empty_dataset() {
    let data: Vec<TestRow> = vec![];
    let mut filters = TestRowFilters::default();

    // No filters
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        0
    );

    // With filters
    filters.name = "test".to_string();
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        0
    );
}

#[test]
fn test_filter_single_row_dataset() {
    let data = vec![TestRow::new(
        "Solo",
        "solo@test.com",
        30,
        100,
        true,
        TestCategory::Engineering,
    )];
    let mut filters = TestRowFilters::default();

    // Matching filter
    filters.name = "solo".to_string();
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        1
    );

    // Non-matching filter
    filters.name = "other".to_string();
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        0
    );
}

// ============================================================================
// Filter Independence Tests
// ============================================================================

#[test]
fn test_filter_clone_independence() {
    let mut filters1 = TestRowFilters::default();
    filters1.name = "alice".to_string();

    let mut filters2 = filters1.clone();
    filters2.name = "bob".to_string();

    // Changing filters2 should not affect filters1
    assert_eq!(filters1.name, "alice");
    assert_eq!(filters2.name, "bob");
}

#[test]
fn test_filter_default_values() {
    let filters = TestRowFilters::default();

    // All filters should be in "match all" state
    assert!(filters.name.is_empty());
    assert!(filters.email.is_empty());
    assert_eq!(filters.age, (None, None));
    assert_eq!(filters.score, (None, None));
    assert!(filters.active.is_empty());
    assert!(filters.category.is_empty());
}

// ============================================================================
// Complex Multi-Filter Scenarios
// ============================================================================

#[test]
fn test_mutually_exclusive_filters() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Filter for Engineering AND Sales (impossible for single row)
    // This tests that faceted filters use OR within, AND between
    filters.category = HashSet::from([TestCategory::Engineering.to_string()]);
    filters.active = HashSet::from([false.to_string()]);

    let matches: Vec<_> = data
        .iter()
        .filter(|r| r.matches_filters(&filters))
        .collect();
    // Only Charlie (Engineering, inactive)
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].name, "Charlie");
}

#[test]
fn test_filter_narrows_progressively() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    let count_all = data.iter().filter(|r| r.matches_filters(&filters)).count();
    assert_eq!(count_all, 5);

    // Each filter should narrow or maintain the count
    filters.active = HashSet::from([true.to_string()]);
    let count1 = data.iter().filter(|r| r.matches_filters(&filters)).count();
    assert!(count1 <= count_all);

    filters.category = HashSet::from([TestCategory::Engineering.to_string()]);
    let count2 = data.iter().filter(|r| r.matches_filters(&filters)).count();
    assert!(count2 <= count1);

    filters.age = (Some(20.0), Some(30.0));
    let count3 = data.iter().filter(|r| r.matches_filters(&filters)).count();
    assert!(count3 <= count2);
}

#[test]
fn test_contradictory_range_filters() {
    let data = create_test_data();
    let mut filters = TestRowFilters::default();

    // Age must be > 100 (impossible with our data)
    filters.age = (Some(100.0), None);
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        0
    );

    // Score must be exactly 0 (impossible with our data)
    filters.age = (None, None); // reset
    filters.score = (Some(0.0), Some(0.0));
    assert_eq!(
        data.iter().filter(|r| r.matches_filters(&filters)).count(),
        0
    );
}
