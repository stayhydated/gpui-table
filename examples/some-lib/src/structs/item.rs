use es_fluent::EsFluentKv;
use fake::faker::{chrono::en::DateTime, color::en::HexColor, lorem::en::Word};
use fake::uuid::UUIDv4;

use gpui_table::GpuiTable;
#[derive(fake::Dummy, EsFluentKv, GpuiTable)]
#[fluent_kv(this)]
#[gpui_table(fluent, custom_style)]
pub struct Item {
    #[gpui_table(skip)]
    #[dummy(faker = "UUIDv4")]
    #[allow(dead_code)]
    id: uuid::Uuid,

    #[gpui_table(width = 100., filter = "Text")]
    #[dummy(faker = "Word()")]
    name: String,

    #[gpui_table(width = 80., resizable = false, filter = "Text")]
    #[dummy(faker = "HexColor()")]
    color: String,

    #[gpui_table(width = 120., movable = false, ascending, filter = "Number")]
    #[dummy(faker = "1..67")]
    weight: u8,

    #[gpui_table(width = 250., filter = "Date")]
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
            ItemTableColumn::Color => {
                let color_hex = self.color.trim_start_matches('#');
                let color_u32 = u32::from_str_radix(color_hex, 16).unwrap_or(0xFFFFFF);

                return div()
                    .bg(gpui::rgb(color_u32))
                    .px_2()
                    .py_0p5()
                    .into_any_element();
            },
            ItemTableColumn::Weight => {
                let weight_ratio = (self.weight as f32) / 67.0;
                let weight_ratio = weight_ratio.min(1.0);

                let green = (255.0 * (1.0 - weight_ratio)) as u32;
                let blue = (255.0 * (1.0 - weight_ratio)) as u32;
                let hex_color = 0xFF0000 | (green << 8) | blue;
                let bg_color = gpui::rgb(hex_color);

                let (tag_label, tag_bg_color, tag_text_color) = if self.weight < 30 {
                    ("light", gpui::rgb(0x22c55e), gpui::white())
                } else if self.weight < 50 {
                    ("medium", gpui::rgb(0xeab308), gpui::white())
                } else {
                    ("heavy", gpui::rgb(0xef4444), gpui::white())
                };

                return div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .child(format!("{} kg", self.weight))
                            .text_color(gpui::black())
                            .bg(bg_color)
                            .px_1()
                            .rounded_md(),
                    )
                    .child(
                        div()
                            .child(tag_label)
                            .text_xs()
                            .px_2()
                            .py_0p5()
                            .rounded_md()
                            .bg(tag_bg_color)
                            .text_color(tag_text_color),
                    )
                    .into_any_element();
            },
            _ => {},
        }

        gpui_table::default_render_cell(self, col.into(), window, cx).into_any_element()
    }
}
