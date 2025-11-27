use es_fluent::EsFluentKv;
use gpui::IntoElement;
use gpui_table::NamedTableRow;
use gpui_table::TableRowStyle;

#[derive(NamedTableRow, EsFluentKv, fake::Dummy)]
#[fluent_kv(display = "std")]
#[fluent_kv(this)]
#[table(fluent, custom_style)]
pub struct Fruit {
    #[table(skip)]
    id: usize,

    #[table(width = 100.)]
    name: String,

    #[table(width = 80.)]
    color: String,

    #[table(width = 60.)]
    weight_grams: u32,

    #[table(width = 50.)]
    ripe: bool,
}

impl gpui_table::TableRowStyle for Fruit {
    type ColumnId = FruitColumn;

    fn render_table_cell(
        &self,
        col: Self::ColumnId,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> gpui::AnyElement {
        use gpui::{IntoElement, ParentElement, Styled, div};

        match col {
            FruitColumn::Ripe => {
                if self.ripe {
                    return div()
                        .child("RIPE")
                        .text_color(gpui::red())
                        .bg(gpui::yellow())
                        .px_1()
                        .rounded_md()
                        .into_any_element();
                }
            },
            _ => {},
        }

        // Fallback to default
        gpui_table::default_render_cell(self, col.into(), window, cx).into_any_element()
    }
}
