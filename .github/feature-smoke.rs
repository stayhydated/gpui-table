fn accepts_cell<T: gpui_table::TableCell>() {}

fn main() {
    use gpui_kit::IntoElement as _;
    let _ = gpui_table::sort::ResolvedSortColumn::new("calculated", |row: &u64| Ok(Some(*row)));
    let _ = gpui_table::sort::ResolvedSortColumns::<u64>::default();
    let _ = gpui_table::runtime::ResolvedTableColumn::<u64>::new(
        gpui_kit::component::table::Column::new("calculated", "Calculated"),
        |_, _, _| gpui_kit::div().into_any_element(),
    );
    #[cfg(feature = "derive")]
    {
        let mut delegate = ConsumerRowTableDelegate::new(Vec::new());
        delegate.set_allowed_sort_columns(["value"]).unwrap();
        delegate
            .set_resolved_sort_columns(Default::default())
            .unwrap();
        delegate.set_resolved_table_columns(Vec::new()).unwrap();
    }

    let value = gpui_table::filter::TextValue::from("needle");
    let _ = gpui_table::FilterPresetValue::to_preset_json(&value);
    let _ = gpui_table::registry::TableId::new("consumer");
    accepts_cell::<String>();
    #[cfg(feature = "derive")]
    accepts_cell::<ConsumerCell>();
    #[cfg(feature = "chrono")]
    accepts_cell::<gpui_table::__deps::chrono::NaiveDate>();
    #[cfg(feature = "rust_decimal")]
    accepts_cell::<gpui_table::__deps::rust_decimal::Decimal>();
    #[cfg(feature = "spacetimedb")]
    accepts_cell::<spacetimedb_lib::TimeDuration>();
    #[cfg(feature = "mcp")]
    {
        let _ = gpui_table::mcp::server();
    }
}

#[cfg(feature = "derive")]
#[derive(gpui_table::TableCell)]
#[table_cell(display)]
struct ConsumerCell(u64);

#[cfg(feature = "derive")]
impl core::fmt::Display for ConsumerCell {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "derive")]
#[derive(Clone, gpui_table::GpuiTable)]
struct ConsumerRow {
    #[gpui_table(sortable)]
    value: u64,
}
