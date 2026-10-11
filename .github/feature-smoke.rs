fn accepts_cell<T: gpui_table::TableCell>() {}

fn main() {
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
