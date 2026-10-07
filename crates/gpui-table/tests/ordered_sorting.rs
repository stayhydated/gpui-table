use gpui_table::{
    GpuiTable,
    sort::{NullPlacement, SortClause, SortDirection, SortError, SortOrder},
};

#[derive(Clone, GpuiTable)]
#[gpui_table(row_id = "id")]
struct Record {
    #[gpui_table(skip)]
    id: u64,
    #[gpui_table(sortable, col = "category_key")]
    category: String,
    #[gpui_table(sortable, sort_key = calculated_score, style = score_cell)]
    score: i64,
    #[gpui_table(skip)]
    adjustment: i64,
    #[gpui_table(sortable)]
    optional: Option<i64>,
    label: String,
}

fn calculated_score(row: &Record) -> Result<Option<i64>, SortError> {
    row.score
        .checked_add(row.adjustment)
        .map(Some)
        .ok_or_else(|| SortError::Calculation {
            column: "score".into(),
            message: "overflow".into(),
        })
}
fn records() -> Vec<Record> {
    vec![
        Record {
            id: 3,
            category: "a".into(),
            score: 4,
            adjustment: 0,
            optional: None,
            label: "third".into(),
        },
        Record {
            id: 2,
            category: "b".into(),
            score: 1,
            adjustment: 10,
            optional: Some(2),
            label: "second".into(),
        },
        Record {
            id: 1,
            category: "a".into(),
            score: 3,
            adjustment: 1,
            optional: Some(1),
            label: "first".into(),
        },
    ]
}
fn order() -> SortOrder {
    SortOrder::new(vec![
        SortClause::new("category_key", SortDirection::Ascending),
        SortClause::new("score", SortDirection::Descending),
    ])
    .unwrap()
}
fn visible_ids(delegate: &RecordTableDelegate) -> Vec<u64> {
    delegate
        .visible_row_indices()
        .iter()
        .map(|ix| delegate.rows[*ix].id)
        .collect()
}

#[test]
fn generated_ordering_uses_column_keys_calculated_values_and_identity_ties() {
    assert_eq!(RecordTableColumn::Category.key(), "category_key");
    let mut delegate = RecordTableDelegate::new(records());
    delegate.set_ordering(order()).unwrap();
    assert_eq!(visible_ids(&delegate), [1, 3, 2]);
    assert_eq!(
        delegate.rows.iter().map(|row| row.id).collect::<Vec<_>>(),
        [3, 2, 1]
    );
    delegate.rows[0].adjustment = 10;
    delegate.refresh_filtered_rows();
    assert_eq!(visible_ids(&delegate), [3, 1, 2]);
    delegate.set_row_scope(|row| row.id != 3);
    assert_eq!(visible_ids(&delegate), [1, 2]);
    delegate.clear_row_scope();
    delegate.set_ordering(SortOrder::default()).unwrap();
    assert_eq!(visible_ids(&delegate), [3, 2, 1]);
}

#[test]
fn calculated_errors_and_unsupported_columns_are_explicit_and_atomic() {
    let mut delegate = RecordTableDelegate::new(records());
    delegate.set_ordering(order()).unwrap();
    let unsupported =
        SortOrder::new(vec![SortClause::new("label", SortDirection::Ascending)]).unwrap();
    assert_eq!(
        delegate.set_ordering(unsupported),
        Err(SortError::UnsupportedColumn("label".into()))
    );
    assert_eq!(visible_ids(&delegate), [1, 3, 2]);
    delegate.rows[0].score = i64::MAX;
    delegate.rows[0].adjustment = 1;
    delegate.refresh_filtered_rows();
    assert!(matches!(
        delegate.sort_error(),
        Some(SortError::Calculation { .. })
    ));
    delegate.set_row_scope(|row| row.id != 3);
    assert_eq!(delegate.sort_error(), None);
    assert_eq!(visible_ids(&delegate), [1, 2]);
}

#[test]
fn null_policy_and_persistence_round_trip() {
    let order = SortOrder::new(vec![
        SortClause::new("optional", SortDirection::Descending).with_nulls(NullPlacement::First),
    ])
    .unwrap();
    let json = serde_json::to_value(&order).unwrap();
    let restored: SortOrder = serde_json::from_value(json).unwrap();
    assert_eq!(restored, order);
    let mut delegate = RecordTableDelegate::new(records());
    delegate.set_ordering(restored).unwrap();
    assert_eq!(visible_ids(&delegate), [3, 2, 1]);
    for invalid in [
        serde_json::json!([{"column":"optional", "direction":"ascending", "extra":true}]),
        serde_json::json!([{"column":"optional", "direction":"sideways"}]),
        serde_json::json!([{"column":"optional", "direction":"ascending"}, {"column":"optional", "direction":"descending"}]),
    ] {
        assert!(serde_json::from_value::<SortOrder>(invalid).is_err());
    }
}

#[gpui_kit::test]
fn controlled_sort_and_refresh_keep_row_and_cell_selection_on_stable_ids(
    cx: &mut gpui_kit::TestAppContext,
) {
    use gpui_kit::component::table::{TableSelection, TableState};
    use gpui_kit::{AppContext as _, Empty};
    use gpui_table::runtime::{TableRowSelection, set_table_ordering};
    cx.update(gpui_kit::init);
    let window = cx.add_window(|_, _| Empty);
    let table = window
        .update(cx, |_, window, cx| {
            cx.new(|cx| TableState::new(RecordTableDelegate::new(records()), window, cx))
        })
        .unwrap();
    table.update(cx, |table, cx| {
        table.set_selected_row(0, cx);
        set_table_ordering(table, order(), cx).unwrap();
        assert_eq!(table.selection(), TableSelection::Row(1));
        table.set_selected_cell(2, 1, cx);
        let selection = TableRowSelection::capture(table);
        table.delegate_mut().rows.insert(
            0,
            Record {
                id: 0,
                category: "a".into(),
                score: 20,
                adjustment: 0,
                optional: None,
                label: "inserted".into(),
            },
        );
        table.delegate().refresh_filtered_rows();
        selection.restore(table, cx);
        assert_eq!(table.selection(), TableSelection::Cell(3, 1));
        let selection = TableRowSelection::capture(table);
        assert_eq!(selection.row_id(), Some(&2));
        table.delegate_mut().rows.retain(|row| row.id != 2);
        table.delegate().refresh_filtered_rows();
        selection.clone().restore(table, cx);
        assert_eq!(table.selection(), TableSelection::None);
        table
            .delegate_mut()
            .rows
            .extend(records().into_iter().filter(|row| row.id == 2));
        table.delegate().refresh_filtered_rows();
        selection.restore(table, cx);
        assert_eq!(table.selection(), TableSelection::Cell(3, 1));
    });
}

#[cfg(feature = "inventory")]
#[test]
fn registered_metadata_preserves_source_names_and_exposes_calculated_column_keys() {
    let shape = gpui_table::schema::registry::inventory::iter::<
        gpui_table::schema::registry::GpuiTableShape,
    >
        .into_iter()
        .find(|shape| shape.struct_name == "Record")
        .unwrap();
    assert_eq!(shape.columns[0].field_name, "category");
    assert_eq!(shape.columns[0].key, "category_key");
    assert!(!shape.columns[0].calculated);
    assert!(shape.columns[1].calculated);
}

fn score_cell(
    row: &Record,
    _: &i64,
    _: &mut gpui_kit::Window,
    _: &mut gpui_kit::App,
) -> gpui_kit::AnyElement {
    use gpui_kit::test::TestSupportExt;
    use gpui_kit::{
        InteractiveElement as _, IntoElement as _, ParentElement as _,
        StatefulInteractiveElement as _,
    };
    let label = calculated_score(row)
        .ok()
        .flatten()
        .map_or_else(|| "Unavailable".into(), |value| value.to_string());
    gpui_kit::div()
        .id(format!("calculated-score-{}", row.id))
        .test_support()
        .aria_label(label.clone())
        .child(label)
        .into_any_element()
}

#[gpui_kit::test]
fn calculated_cell_presentation_uses_the_same_key_and_refreshes(cx: &mut gpui_kit::TestAppContext) {
    use gpui_kit::component::{Root, v_flex};
    use gpui_kit::test::TestWindowExt;
    use gpui_kit::{
        AppContext as _, Context, IntoElement, ParentElement as _, Render, Styled as _, Window, px,
        size,
    };
    use gpui_table::runtime::TableRowStyle as _;
    struct Preview(Vec<Record>);
    impl Render for Preview {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            v_flex().size_full().children(
                self.0
                    .iter()
                    .map(|row| row.render_table_cell(RecordTableColumn::Score, window, cx)),
            )
        }
    }
    cx.update(gpui_kit::init);
    let mut preview = None;
    let window = cx.open_window(size(px(640.), px(480.)), |window, cx| {
        let view = cx.new(|_| Preview(records()));
        preview = Some(view.clone());
        Root::new(view, window, cx)
    });
    let preview = preview.unwrap();
    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(window.find("calculated-score-1").label(), Some("4"));
        preview.update(cx, |preview, cx| {
            preview.0[2].adjustment = 40;
            cx.notify();
        });
        window.render_frame(cx);
        assert_eq!(window.find("calculated-score-1").label(), Some("43"));
    })
    .unwrap();
}

#[gpui_kit::test]
fn declared_ordering_capabilities_constrain_headers_and_direct_changes_atomically(
    cx: &mut gpui_kit::TestAppContext,
) {
    use gpui_kit::component::table::TableDelegate;
    cx.update(|cx| {
        let mut delegate = RecordTableDelegate::new(records());
        assert!(delegate.column(1, cx).sort.is_some());
        delegate
            .set_allowed_sort_columns(["category_key", "optional"])
            .unwrap();
        assert!(delegate.column(0, cx).sort.is_some());
        assert!(delegate.column(1, cx).sort.is_none());
        assert_eq!(
            delegate.set_ordering(order()),
            Err(SortError::UnsupportedColumn("score".into()))
        );
        assert!(delegate.ordering().is_empty());
        assert_eq!(
            delegate.set_allowed_sort_columns(["label"]),
            Err(SortError::UnsupportedColumn("label".into()))
        );
        assert!(delegate.column(1, cx).sort.is_none());
        assert_eq!(
            delegate.set_allowed_sort_columns(["optional", "optional"]),
            Err(SortError::DuplicateColumn("optional".into()))
        );
        delegate
            .set_ordering(
                SortOrder::new(vec![SortClause::new("optional", SortDirection::Descending)])
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            delegate.set_allowed_sort_columns(["category_key"]),
            Err(SortError::UnsupportedColumn("optional".into()))
        );
        assert!(delegate.column(2, cx).sort.is_some());
        delegate.set_ordering(SortOrder::default()).unwrap();
        delegate
            .set_allowed_sort_columns(std::iter::empty::<String>())
            .unwrap();
        assert!(delegate.column(0, cx).sort.is_none());
        assert!(delegate.column(2, cx).sort.is_none());
    });
}
