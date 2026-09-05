use gpui_table::GpuiTable;

#[derive(GpuiTable)]
struct InvalidDuplicateStyle {
    #[gpui_table(style = render_name, style = render_name)]
    name: String,
}

fn render_name(
    _row: &InvalidDuplicateStyle,
    _value: &String,
    _window: &mut gpui_kit::Window,
    _cx: &mut gpui_kit::App,
) -> impl gpui_kit::IntoElement {
    gpui_kit::div()
}

fn main() {}
