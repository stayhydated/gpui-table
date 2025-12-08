use es_fluent::EsFluentKv;
use fake::faker::{chrono::en::DateTime, color::en::Color, lorem::en::Word};
use fake::uuid::UUIDv4;
use gpui_table::NamedTableRow;
#[derive(fake::Dummy, EsFluentKv, NamedTableRow)]
#[fluent_kv(this)]
#[table(fluent, custom_style)]
pub struct Item {
    #[table(skip)]
    #[dummy(faker = "UUIDv4")]
    id: uuid::Uuid,

    #[table(width = 100.)]
    #[dummy(faker = "Word()")]
    name: String,

    #[table(width = 80.)]
    #[dummy(faker = "Color()")]
    color: String,

    #[table(width = 60.)]
    #[dummy(faker = "18..67")]
    weight: u8,

    #[table(width = 50.)]
    #[dummy(faker = "DateTime()")]
    acquired_on: chrono::DateTime<chrono::Utc>,
}

impl gpui_table::TableRowStyle for Item {
    type ColumnId = ItemTableColumn;

    fn render_table_cell(
        &self,
        col: Self::ColumnId,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> gpui::AnyElement {
        use gpui::{IntoElement, ParentElement, Styled, div};

        match col {
            ItemTableColumn::Weight => {
                return div()
                    .child(format!("{} g", self.weight))
                    .text_color(gpui::black())
                    .bg(gpui::white())
                    .px_1()
                    .rounded_md()
                    .into_any_element();
            },
            _ => {},
        }

        gpui_table::default_render_cell(self, col.into(), window, cx).into_any_element()
    }
}
