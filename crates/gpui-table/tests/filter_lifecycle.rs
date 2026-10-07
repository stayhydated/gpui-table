use gpui_kit::component::table::TableState;
use gpui_kit::{AppContext as _, Empty, TestAppContext};

#[derive(Clone, gpui_table::GpuiTable)]
#[gpui_table(filters, row_id = "id")]
struct Record {
    #[gpui_table(skip)]
    id: u64,
    #[gpui_table(filter(gpui_table_component::TextFilterAdapter))]
    name: String,
}

#[gpui_kit::test]
fn generated_filter_builders_release_entities_and_ignore_closed_tables(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    cx.update(|cx| gpui_table_component::i18n::init(cx).unwrap());
    let window = cx.add_window(|_, _| Empty);
    for loader in [false, true] {
        let (table, filters) = window
            .update(cx, |_, window, cx| {
                let table = cx.new(|cx| {
                    TableState::new(
                        RecordTableDelegate::new(vec![
                            Record {
                                id: 1,
                                name: "alpha".into(),
                            },
                            Record {
                                id: 2,
                                name: "beta".into(),
                            },
                        ]),
                        window,
                        cx,
                    )
                });
                let filters = if loader {
                    RecordFilterEntities::build_for_table_loader(table.clone(), window, cx)
                } else {
                    RecordFilterEntities::build_for_table(table.clone(), cx)
                };
                filters.apply_values(
                    RecordFilterValues {
                        name: "alpha".into(),
                    },
                    window,
                    cx,
                );
                (table, filters)
            })
            .unwrap();
        cx.update(|cx| {
            assert_eq!(
                table.read(cx).delegate().visible_row_indices().len(),
                if loader { 0 } else { 1 }
            )
        });
        let table_weak = table.downgrade();
        let filter_weak = filters.name.downgrade();
        drop(table);
        cx.update(|_| {});
        assert!(table_weak.upgrade().is_none());
        // A queued control callback may run after its target table closes.
        window
            .update(cx, |_, window, cx| filters.reset_filters(window, cx))
            .unwrap();
        drop(filters);
        cx.update(|_| {});
        assert!(filter_weak.upgrade().is_none());
    }
}
