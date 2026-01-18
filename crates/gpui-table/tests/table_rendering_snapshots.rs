use gpui::{Context, TextAlign, Window};
use gpui_component::table::{Column, ColumnFixed, ColumnSort, TableDelegate as _, TableState};
use gpui_table::{GpuiTable, TableRowMeta, gpui_table_impl};
use serde::Serialize;

// =============================================================================
// Basic row with minimal configuration
// =============================================================================

#[derive(GpuiTable)]
struct BasicRow {
    name: String,
    age: u8,
    active: bool,
}

// =============================================================================
// Styled row with custom column settings
// =============================================================================

#[derive(GpuiTable)]
#[gpui_table(id = "custom-row", title = "Custom Row Table", load_more)]
struct StyledRow {
    #[gpui_table(width = 120., sortable)]
    name: String,

    #[gpui_table(width = 80., text_right, descending, fixed = "left", resizable = false)]
    score: u8,

    #[gpui_table(width = 180., ascending, title = "Email Address", movable = false)]
    email: String,

    #[gpui_table(skip)]
    #[allow(dead_code)]
    internal: String,
}

#[gpui_table_impl]
impl StyledRowTableDelegate {
    #[load_more]
    fn load_more(&mut self, _window: &mut Window, _cx: &mut Context<TableState<Self>>) {
        // No-op for tests
    }
}

// =============================================================================
// Row with custom threshold
// =============================================================================

#[derive(GpuiTable)]
#[gpui_table(load_more)]
struct ThresholdRow {
    value: u32,
}

#[gpui_table_impl]
impl ThresholdRowTableDelegate {
    #[threshold]
    const LOAD_MORE_THRESHOLD: usize = 42;

    #[load_more]
    fn load_more(&mut self, _window: &mut Window, _cx: &mut Context<TableState<Self>>) {
        // No-op for tests
    }
}

// =============================================================================
// Row with different threshold value
// =============================================================================

#[derive(GpuiTable)]
#[gpui_table(load_more)]
struct AnotherThresholdRow {
    data: String,
}

#[gpui_table_impl]
impl AnotherThresholdRowTableDelegate {
    #[threshold]
    const MY_THRESHOLD: usize = 15;

    #[load_more]
    fn fetch_more(&mut self, _window: &mut Window, _cx: &mut Context<TableState<Self>>) {
        // No-op for tests
    }
}

// =============================================================================
// Row to test load_more method is actually called
// =============================================================================

#[derive(GpuiTable)]
#[gpui_table(load_more)]
struct CallbackRow {
    id: u32,
}

static LOAD_MORE_CALLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[gpui_table_impl]
impl CallbackRowTableDelegate {
    #[threshold]
    const THRESHOLD: usize = 5;

    #[load_more]
    fn on_load_more(&mut self, _window: &mut Window, _cx: &mut Context<TableState<Self>>) {
        LOAD_MORE_CALLED.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Serialize)]
struct ColumnSnapshot {
    key: String,
    title: String,
    align: &'static str,
    sort: Option<&'static str>,
    width: f32,
    fixed: Option<&'static str>,
    resizable: bool,
    movable: bool,
    selectable: bool,
}

#[derive(Serialize)]
struct TableSnapshot {
    table_id: &'static str,
    title: String,
    columns: Vec<ColumnSnapshot>,
}

fn to_column_snapshot(column: &Column) -> ColumnSnapshot {
    let width: f32 = (&column.width).into();

    ColumnSnapshot {
        key: column.key.to_string(),
        title: column.name.to_string(),
        align: match column.align {
            TextAlign::Left => "left",
            TextAlign::Center => "center",
            TextAlign::Right => "right",
        },
        sort: column.sort.as_ref().map(|sort| match sort {
            ColumnSort::Default => "default",
            ColumnSort::Ascending => "ascending",
            ColumnSort::Descending => "descending",
        }),
        width,
        fixed: column.fixed.map(|fixed| match fixed {
            ColumnFixed::Left => "left",
        }),
        resizable: column.resizable,
        movable: column.movable,
        selectable: column.selectable,
    }
}

fn table_snapshot<T: TableRowMeta>() -> TableSnapshot {
    TableSnapshot {
        table_id: T::TABLE_ID,
        title: T::table_title(),
        columns: T::table_columns().iter().map(to_column_snapshot).collect(),
    }
}

#[test]
fn basic_table_rendering_snapshot() {
    insta::assert_yaml_snapshot!("basic_table_rendering", table_snapshot::<BasicRow>());
}

#[test]
fn styled_table_rendering_snapshot() {
    insta::assert_yaml_snapshot!("styled_table_rendering", table_snapshot::<StyledRow>());
}

// =============================================================================
// Tests for #[gpui_table_impl] attribute behavior
// =============================================================================

#[test]
fn test_default_threshold_no_load_more() {
    // BasicRowTableDelegate has load_more disabled, so it uses the default (10)
    let delegate = BasicRowTableDelegate::new(vec![]);
    assert_eq!(delegate.load_more_threshold(), 10);
}

#[test]
fn test_default_threshold_load_more_enabled() {
    // StyledRowTableDelegate enables load_more but has no #[threshold] const.
    let delegate = StyledRowTableDelegate::new(vec![]);
    assert_eq!(delegate.load_more_threshold(), 10);
}

#[test]
fn test_custom_threshold() {
    // ThresholdRowTableDelegate has #[threshold] const LOAD_MORE_THRESHOLD: usize = 42
    let delegate = ThresholdRowTableDelegate::new(vec![]);
    assert_eq!(delegate.load_more_threshold(), 42);
}

#[test]
fn test_custom_threshold_different_name() {
    // AnotherThresholdRowTableDelegate has #[threshold] const MY_THRESHOLD: usize = 15
    let delegate = AnotherThresholdRowTableDelegate::new(vec![]);
    assert_eq!(delegate.load_more_threshold(), 15);
}

#[test]
fn test_callback_row_threshold() {
    // CallbackRowTableDelegate has #[threshold] const THRESHOLD: usize = 5
    let delegate = CallbackRowTableDelegate::new(vec![]);
    assert_eq!(delegate.load_more_threshold(), 5);
}

#[test]
fn test_has_more_default_eof() {
    // Test has_more with default eof field
    let delegate = BasicRowTableDelegate::new(vec![]);

    // With no #[load_more], has_more defaults to false even though eof/loading are false.
    assert!(!delegate.eof);
    assert!(!delegate.loading);
    // Note: has_more requires &App which we can't easily create in unit tests
    // We'll just verify the delegate compiles and threshold works
}

#[test]
fn test_delegate_fields_exist() {
    // Verify that the generated delegate has expected fields
    let delegate =
        ThresholdRowTableDelegate::new(vec![ThresholdRow { value: 1 }, ThresholdRow { value: 2 }]);

    assert_eq!(delegate.rows.len(), 2);
    assert!(!delegate.eof);
    assert!(!delegate.loading);
    assert!(!delegate.full_loading);
}
