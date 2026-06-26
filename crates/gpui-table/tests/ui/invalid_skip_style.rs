use gpui_table::GpuiTable;

#[derive(GpuiTable)]
struct InvalidSkipStyle {
    #[gpui_table(skip, style = render_name)]
    name: String,
}

fn render_name(
    _row: &InvalidSkipStyle,
    _value: &String,
    _window: &mut gpui::Window,
    _cx: &mut gpui::App,
) -> impl gpui::IntoElement {
    gpui::div()
}

fn main() {}
