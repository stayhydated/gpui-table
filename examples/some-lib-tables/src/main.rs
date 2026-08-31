use gpui_storybook::Assets;

fn main() {
    env_logger::init();

    let app = gpui_platform::application().with_assets(Assets);
    some_lib_tables::run_storybook(app);
}
