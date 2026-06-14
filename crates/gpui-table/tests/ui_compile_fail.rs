#[test]
fn ui_compile_fail() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/table_cell_display_format.rs");
    #[cfg(feature = "chrono")]
    t.pass("tests/ui/explicit_filter_shapes.rs");
    t.pass("tests/ui/faceted_option_filter.rs");
    t.pass("tests/ui/faceted_vec_filter.rs");
    t.pass("tests/ui/gpui_table_filter_shape_adapter_runtime.rs");
    #[cfg(feature = "mcp")]
    t.pass("tests/ui/mcp_filter_shape_derive.rs");
    #[cfg(feature = "mcp")]
    t.pass("tests/ui/gpui_table_filter_shape_adapter.rs");
    t.compile_fail("tests/ui/duplicate_filter_shape_field.rs");
    t.compile_fail("tests/ui/bare_filter_requires_shape.rs");
    t.compile_fail("tests/ui/filter_without_struct_filters.rs");
    t.compile_fail("tests/ui/koruma_filter_requires_mcp.rs");
    #[cfg(not(feature = "mcp"))]
    t.compile_fail("tests/ui/mcp_requires_feature.rs");
    #[cfg(all(feature = "mcp", feature = "chrono", feature = "rust_decimal"))]
    t.compile_fail("tests/ui/mcp_filter_shape_requires_decode.rs");
    #[cfg(all(
        feature = "mcp",
        not(feature = "chrono"),
        not(feature = "rust_decimal")
    ))]
    t.compile_fail("tests/ui/mcp_filter_shape_requires_decode_minimal.rs");
    t.compile_fail("tests/ui/invalid_fixed_value.rs");
    t.compile_fail("tests/ui/invalid_gpui_table_impl_arguments.rs");
    t.compile_fail("tests/ui/invalid_gpui_table_impl_target.rs");
    t.compile_fail("tests/ui/invalid_context_menu_route_without_row_id.rs");
    t.compile_fail("tests/ui/invalid_context_menu_row_id_field.rs");
    t.compile_fail("tests/ui/invalid_context_menu_route_and_route_fn.rs");
    t.compile_fail("tests/ui/invalid_context_menu_multiple_id_fields.rs");
    t.compile_fail("tests/ui/invalid_filter_shape_path.rs");
    t.compile_fail("tests/ui/invalid_text_filter_type.rs");
    t.compile_fail("tests/ui/invalid_faceted_filter_type.rs");
    t.compile_fail("tests/ui/invalid_faceted_option_type.rs");
    #[cfg(not(feature = "chrono"))]
    t.compile_fail("tests/ui/date_range_requires_chrono.rs");

    #[cfg(feature = "chrono")]
    t.pass("tests/ui/date_range_requires_chrono.rs");

    #[cfg(all(feature = "chrono", not(feature = "spacetimedb")))]
    t.compile_fail("tests/ui/invalid_date_range_type.rs");

    #[cfg(all(feature = "chrono", feature = "spacetimedb"))]
    t.compile_fail("tests/ui/invalid_date_range_type_spacetimedb.rs");

    #[cfg(not(feature = "rust_decimal"))]
    t.compile_fail("tests/ui/number_range_requires_rust_decimal.rs");

    #[cfg(feature = "rust_decimal")]
    t.pass("tests/ui/number_range_requires_rust_decimal.rs");

    #[cfg(all(feature = "rust_decimal", not(feature = "spacetimedb")))]
    t.compile_fail("tests/ui/invalid_number_range_type.rs");

    #[cfg(all(feature = "rust_decimal", feature = "spacetimedb"))]
    t.compile_fail("tests/ui/invalid_number_range_type_spacetimedb.rs");

    #[cfg(all(feature = "chrono", not(feature = "spacetimedb")))]
    t.compile_fail("tests/ui/spacetimedb_requires_feature.rs");

    #[cfg(all(feature = "chrono", feature = "spacetimedb"))]
    t.pass("tests/ui/spacetimedb_requires_feature.rs");
}
