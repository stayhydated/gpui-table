use es_fluent::EsFluentKv;
use gpui_table::NamedTableRow;

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
    weight: u32,

    #[table(width = 50.)]
    ripe: bool,
}

impl gpui_table::TableRowStyle for Fruit {
    type ColumnId = FruitTableColumn;

    fn render_table_cell(
        &self,
        col: Self::ColumnId,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> gpui::AnyElement {
        use gpui::{IntoElement, ParentElement, Styled, div};

        match col {
            FruitTableColumn::Ripe => {
                if self.ripe {
                    return div()
                        .child("RIPE")
                        .text_color(gpui::green())
                        .bg(gpui::yellow())
                        .px_1()
                        .rounded_md()
                        .into_any_element();
                } else {
                    return div()
                        .child("UNRIPE")
                        .text_color(gpui::white())
                        .bg(gpui::black())
                        .px_1()
                        .rounded_md()
                        .into_any_element();
                }
            },
            _ => {},
        }

        gpui_table::default_render_cell(self, col.into(), window, cx).into_any_element()
    }
}
