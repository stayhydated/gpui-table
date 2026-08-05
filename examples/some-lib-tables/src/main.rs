use gpui_storybook::Assets;

fn main() {
    env_logger::init();

    let app = gpui_platform::application().with_assets(Assets);
    let selected_story = std::env::args().nth(1);
    some_lib_tables::run_storybook(app, selected_story);
}
