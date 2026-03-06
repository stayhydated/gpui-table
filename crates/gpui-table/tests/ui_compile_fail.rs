#[test]
fn ui_compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/filter_without_struct_filters.rs");
    t.compile_fail("tests/ui/invalid_fixed_value.rs");
    t.compile_fail("tests/ui/invalid_load_more_signature.rs");
    t.compile_fail("tests/ui/invalid_number_range_min_max.rs");
    t.compile_fail("tests/ui/invalid_number_range_step.rs");

    #[cfg(not(feature = "rust_decimal"))]
    t.compile_fail("tests/ui/number_range_requires_rust_decimal.rs");

    #[cfg(feature = "rust_decimal")]
    t.pass("tests/ui/number_range_requires_rust_decimal.rs");
}
