#[test]
fn ui_compile_fail() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/table_cell_display_format.rs");
    t.pass("tests/ui/faceted_option_filter.rs");
    t.pass("tests/ui/faceted_vec_filter.rs");
    t.compile_fail("tests/ui/filter_without_struct_filters.rs");
    t.compile_fail("tests/ui/invalid_fixed_value.rs");
    t.compile_fail("tests/ui/invalid_load_more_signature.rs");
    t.compile_fail("tests/ui/invalid_context_menu_route_without_row_id.rs");
    t.compile_fail("tests/ui/invalid_context_menu_row_id_field.rs");
    t.compile_fail("tests/ui/invalid_context_menu_route_and_route_fn.rs");
    t.compile_fail("tests/ui/invalid_context_menu_multiple_id_fields.rs");
    t.compile_fail("tests/ui/legacy_filter_alias_rejected.rs");
    t.compile_fail("tests/ui/invalid_text_filter_type.rs");
    t.compile_fail("tests/ui/invalid_faceted_filter_type.rs");
    t.compile_fail("tests/ui/invalid_faceted_option_type.rs");
    #[cfg(not(feature = "chrono"))]
    t.compile_fail("tests/ui/date_range_requires_chrono.rs");

    #[cfg(feature = "chrono")]
    t.pass("tests/ui/date_range_requires_chrono.rs");

    #[cfg(feature = "chrono")]
    t.compile_fail("tests/ui/invalid_date_range_type.rs");

    #[cfg(not(feature = "rust_decimal"))]
    t.compile_fail("tests/ui/number_range_requires_rust_decimal.rs");

    #[cfg(feature = "rust_decimal")]
    t.pass("tests/ui/number_range_requires_rust_decimal.rs");

    #[cfg(feature = "rust_decimal")]
    t.compile_fail("tests/ui/invalid_number_range_type.rs");

    #[cfg(all(feature = "chrono", not(feature = "spacetimedb")))]
    t.compile_fail("tests/ui/spacetimedb_requires_feature.rs");

    #[cfg(all(feature = "chrono", feature = "spacetimedb"))]
    t.pass("tests/ui/spacetimedb_requires_feature.rs");
}
