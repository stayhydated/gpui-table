//! Filter traits for generated code.

/// Trait for structs that can be filtered against filter values.
///
/// This trait is automatically implemented by the `#[derive(GpuiTable)]` macro
/// when `#[gpui_table(filters)]` is enabled.
///
/// # Example
///
/// ```ignore
/// #[derive(GpuiTable)]
/// #[gpui_table(filters)]
/// pub struct User {
///     #[gpui_table(filter(text()))]
///     pub name: String,
/// }
///
/// // Auto-generated implementation:
/// // impl Matchable<UserFilterValues> for User {
/// //     fn matches_filters(&self, filters: &UserFilterValues) -> bool { ... }
/// // }
/// ```
pub trait Matchable<F> {
    /// Check if this struct matches the given filter values.
    /// Returns true if all active filters match their corresponding fields.
    fn matches_filters(&self, filters: &F) -> bool;
}

/// Trait for filter value structs that can report if they have active filters.
///
/// This trait is automatically implemented by the `#[derive(GpuiTable)]` macro
/// for the generated `XxxFilterValues` struct.
pub trait FilterValuesExt {
    /// Check if any filter has an active value.
    fn has_active_filters(&self) -> bool;
}
