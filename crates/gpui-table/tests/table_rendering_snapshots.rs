use gpui_kit::component::menu::PopupMenu;
use gpui_kit::component::table::{Column, ColumnFixed, ColumnSort, TableDelegate as _, TableState};
use gpui_kit::{App, Context, TextAlign, Window};
use gpui_table::TableRowMeta;
use gpui_table::runtime::TableLoader;
use gpui_table::{GpuiTable, gpui_table_impl};
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
impl TableLoader for StyledRowTableDelegate {
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
impl TableLoader for ThresholdRowTableDelegate {
    const THRESHOLD: usize = 42;

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
impl TableLoader for AnotherThresholdRowTableDelegate {
    const THRESHOLD: usize = 15;

    fn load_more(&mut self, _window: &mut Window, _cx: &mut Context<TableState<Self>>) {
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
impl TableLoader for CallbackRowTableDelegate {
    const THRESHOLD: usize = 5;

    fn load_more(&mut self, _window: &mut Window, _cx: &mut Context<TableState<Self>>) {
        LOAD_MORE_CALLED.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

// =============================================================================
// Row with custom context menu wiring
// =============================================================================

#[derive(GpuiTable)]
#[gpui_table(custom_context_menu)]
struct ContextMenuRow {
    id: u32,
}

impl gpui_table::runtime::TableRowContextMenu for ContextMenuRow {
    fn render_table_context_menu(
        &self,
        row_ix: usize,
        menu: PopupMenu,
        _window: &mut Window,
        _cx: &mut App,
    ) -> PopupMenu {
        let _ = (self.id, row_ix);
        menu
    }
}

#[derive(GpuiTable)]
#[gpui_table(
    context_menu_row_id = "id",
    context_menu_route = "/users/{id}",
    context_menu_label = "Open user"
)]
struct ContextMenuLinkRow {
    id: u32,
    name: String,
}

fn context_menu_href_for_user_id(id: &u32) -> String {
    format!("/users/{id}")
}

fn context_menu_label_for_user_id(id: &u32) -> &'static str {
    let _ = id;
    "Open user details"
}

#[derive(GpuiTable)]
#[gpui_table(
    context_menu_route_fn = context_menu_href_for_user_id,
    context_menu_label_fn = context_menu_label_for_user_id
)]
struct ContextMenuFnRow {
    #[gpui_table(context_menu_id)]
    id: u32,
    name: String,
}

#[derive(GpuiTable)]
#[gpui_table(
    custom_context_menu,
    context_menu_row_id = "id",
    context_menu_route = "/users/{id}",
    context_menu_label = "Open user"
)]
struct ContextMenuComposedRow {
    id: u32,
}

impl gpui_table::runtime::TableRowContextMenu for ContextMenuComposedRow {
    fn render_table_context_menu(
        &self,
        row_ix: usize,
        menu: PopupMenu,
        window: &mut Window,
        cx: &mut App,
    ) -> PopupMenu {
        use gpui_table::runtime::TableRowGeneratedContextMenu as _;
        self.render_generated_table_context_menu(row_ix, menu, window, cx)
            .link("Share", format!("/share/{}", self.id))
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
    typed_table_id: String,
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
        typed_table_id: T::table_id().to_string(),
        title: T::table_title(),
        columns: T::table_columns().iter().map(to_column_snapshot).collect(),
    }
}

#[test]
fn table_row_meta_exposes_typed_table_id() {
    let table_id = BasicRow::table_id();

    assert_eq!(table_id.as_str(), BasicRow::TABLE_ID);
    assert_eq!(table_id.to_string(), "basic_row");
    assert_eq!(String::from(table_id), "basic_row");
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
    // StyledRowTableDelegate enables load_more and uses TableLoader::THRESHOLD.
    let delegate = StyledRowTableDelegate::new(vec![]);
    assert_eq!(delegate.load_more_threshold(), 10);
}

#[test]
fn test_custom_threshold() {
    // ThresholdRowTableDelegate sets TableLoader::THRESHOLD to 42.
    let delegate = ThresholdRowTableDelegate::new(vec![]);
    assert_eq!(delegate.load_more_threshold(), 42);
}

#[test]
fn test_custom_threshold_second_delegate() {
    // AnotherThresholdRowTableDelegate sets TableLoader::THRESHOLD to 15.
    let delegate = AnotherThresholdRowTableDelegate::new(vec![]);
    assert_eq!(delegate.load_more_threshold(), 15);
}

#[test]
fn test_callback_row_threshold() {
    // CallbackRowTableDelegate sets TableLoader::THRESHOLD to 5.
    let delegate = CallbackRowTableDelegate::new(vec![]);
    assert_eq!(delegate.load_more_threshold(), 5);
}

#[test]
fn test_has_more_default_eof() {
    // Test has_more with default eof field
    let delegate = BasicRowTableDelegate::new(vec![]);

    // With load_more disabled, has_more defaults to false even though eof/loading are false.
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

#[test]
fn test_custom_context_menu_delegate_compiles() {
    let delegate = ContextMenuRowTableDelegate::new(vec![ContextMenuRow { id: 1 }]);
    assert_eq!(delegate.rows.len(), 1);
}

#[test]
fn test_generated_context_menu_link_delegate_compiles() {
    let delegate = ContextMenuLinkRowTableDelegate::new(vec![ContextMenuLinkRow {
        id: 1,
        name: "A".to_string(),
    }]);
    assert_eq!(delegate.rows.len(), 1);
}

#[test]
fn test_generated_context_menu_link_fn_delegate_compiles() {
    let delegate = ContextMenuFnRowTableDelegate::new(vec![ContextMenuFnRow {
        id: 2,
        name: "B".to_string(),
    }]);
    assert_eq!(delegate.rows.len(), 1);
}

#[test]
fn test_generated_context_menu_composition_delegate_compiles() {
    let delegate = ContextMenuComposedRowTableDelegate::new(vec![ContextMenuComposedRow { id: 3 }]);
    assert_eq!(delegate.rows.len(), 1);
}
