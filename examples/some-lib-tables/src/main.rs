use gpui_storybook::{Assets, Gallery};
use some_lib_tables::i18n::Languages;

// bring the stories in scope for inventory
#[allow(unused_imports, clippy::single_component_path_imports)]
use some_lib_tables;

fn main() {
    env_logger::init();

    let app = gpui_platform::application().with_assets(Assets);
    let name_arg = std::env::args().nth(1);

    app.run(move |app_cx| {
        gpui_component::init(app_cx);
        gpui_table_component::i18n::init(app_cx)
            .expect("failed to initialize table component i18n");
        gpui_storybook::init(app_cx, Languages::default());
        gpui_storybook::change_locale(app_cx, Languages::default()).unwrap();

        #[cfg(feature = "router")]
        gpui_router::init(app_cx);

        gpui_tokio::init(app_cx);

        let http_client = std::sync::Arc::new(reqwest_client::ReqwestClient::new());
        app_cx.set_http_client(http_client);

        app_cx.activate(true);

        gpui_storybook::create_new_window(
            &format!("{} - Stories", env!("CARGO_PKG_NAME")),
            move |window, cx| {
                let all_stories = gpui_storybook::generate_stories(window, cx);

                Gallery::view(all_stories, name_arg.as_deref(), window, cx)
            },
            app_cx,
        );
    });
}
