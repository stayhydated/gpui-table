use es_fluent::{EsFluentLabel, EsFluentVariants};
use fake::faker::{chrono::en::DateTime, color::en::HexColor, lorem::en::Word};
use fake::uuid::UUIDv4;
use fake::{Fake as _, Faker};
use gpui_kit::component::table::TableState;
use gpui_kit::{Context, Window};
use gpui_table::GpuiTable;
use gpui_table::runtime::TableLoader;
use std::time::Duration;

#[derive(fake::Dummy, EsFluentLabel, EsFluentVariants, GpuiTable)]
#[gpui_table(fluent, load_more)]
pub struct Item {
    #[gpui_table(skip)]
    #[dummy(faker = "UUIDv4")]
    #[allow(dead_code)]
    id: uuid::Uuid,

    #[gpui_table(width = 100.)]
    #[dummy(faker = "Word()")]
    name: String,

    #[gpui_table(width = 80., resizable = false, style = render_color_cell)]
    #[dummy(faker = "HexColor()")]
    color: String,

    #[gpui_table(width = 120., movable = false, ascending, style = render_weight_cell)]
    #[dummy(faker = "1..67")]
    weight: u8,

    #[gpui_table(width = 250.)]
    #[dummy(faker = "DateTime()")]
    acquired_on: chrono::DateTime<chrono::Utc>,
}

fn render_color_cell(
    row: &Item,
    value: &str,
    window: &mut gpui_kit::Window,
    cx: &mut gpui_kit::App,
) -> impl gpui_kit::IntoElement {
    use gpui_kit::Styled as _;

    let _ = (row, window, cx);
    let color_hex = value.trim_start_matches('#');
    let color_u32 = u32::from_str_radix(color_hex, 16).unwrap_or(0xFFFFFF);

    gpui_kit::div().bg(gpui_kit::rgb(color_u32)).px_2().py_0p5()
}

fn render_weight_cell(
    row: &Item,
    value: &u8,
    window: &mut gpui_kit::Window,
    cx: &mut gpui_kit::App,
) -> impl gpui_kit::IntoElement {
    use gpui_kit::{ParentElement as _, Styled as _};

    let _ = (row, window, cx);
    let weight_ratio = (*value as f32) / 67.0;
    let weight_ratio = weight_ratio.min(1.0);

    let green = (255.0 * (1.0 - weight_ratio)) as u32;
    let blue = (255.0 * (1.0 - weight_ratio)) as u32;
    let hex_color = 0xFF0000 | (green << 8) | blue;
    let bg_color = gpui_kit::rgb(hex_color);

    let (tag_label, tag_bg_color, tag_text_color) = if *value < 30 {
        ("light", gpui_kit::rgb(0x22c55e), gpui_kit::white())
    } else if *value < 50 {
        ("medium", gpui_kit::rgb(0xeab308), gpui_kit::white())
    } else {
        ("heavy", gpui_kit::rgb(0xef4444), gpui_kit::white())
    };

    gpui_kit::div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            gpui_kit::div()
                .child(format!("{value} kg"))
                .text_color(gpui_kit::black())
                .bg(bg_color)
                .px_1()
                .rounded_md(),
        )
        .child(
            gpui_kit::div()
                .child(tag_label)
                .text_xs()
                .px_2()
                .py_0p5()
                .rounded_md()
                .bg(tag_bg_color)
                .text_color(tag_text_color),
        )
}

/// Implement the TableLoader trait to define loading behavior.
/// The `#[gpui_table_impl]` attribute on a trait impl block automatically
/// wires up the trait to the generated TableDelegate implementation.
#[gpui_table::gpui_table_impl]
impl TableLoader for ItemTableDelegate {
    const THRESHOLD: usize = 20;

    fn load_more(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        if self.loading || self.eof {
            return;
        }

        self.loading = true;
        cx.notify();

        cx.spawn(async move |view, cx| {
            // Simulate network delay
            cx.background_executor()
                .timer(Duration::from_millis(100))
                .await;

            // Generate fake data - in a real app, this would be an API call
            let new_rows: Vec<Item> = (0..50).map(|_| Faker.fake()).collect();

            cx.update(|cx| {
                view.update(cx, |table, cx| {
                    let delegate = table.delegate_mut();
                    delegate.rows.extend(new_rows);
                    delegate.loading = false;

                    // Stop after 500 rows for demo purposes
                    if delegate.rows.len() >= 500 {
                        delegate.eof = true;
                    }

                    cx.notify();
                })
                .unwrap();
            });
        })
        .detach();
    }
}
