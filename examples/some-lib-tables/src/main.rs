use gpui_storybook::Assets;

fn main() {
    env_logger::init();

    let app = gpui_kit::application().with_assets(Assets);
    app.run(some_lib_tables::launch_storybook);
}
