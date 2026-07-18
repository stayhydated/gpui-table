use gpui_storybook::{Assets, ConsumerId, Gallery, StorybookOptions};
use some_lib::i18n::{self, Languages};

// bring the stories in scope for inventory
#[allow(unused_imports, clippy::single_component_path_imports)]
use some_lib_tables;

const CONSUMER_ID: &str = "gpui-table-some-lib-tables";

fn storybook_options() -> Result<StorybookOptions<Languages>, gpui_storybook::ConsumerIdError> {
    Ok(StorybookOptions::new(
        ConsumerId::new(CONSUMER_ID)?,
        Languages::default(),
        i18n::apply_locale,
    ))
}

fn main() {
    env_logger::init();

    let app = gpui_platform::application().with_assets(Assets);
    let name_arg = std::env::args().nth(1);

    app.run(move |app_cx| {
        let options = match storybook_options() {
            Ok(options) => options,
            Err(error) => {
                eprintln!("invalid table example Storybook consumer id: {error}");
                app_cx.quit();
                return;
            },
        };
        let readiness = match gpui_storybook::init(app_cx, options) {
            Ok(readiness) => readiness,
            Err(error) => {
                eprintln!("failed to initialize table example Storybook: {error}");
                app_cx.quit();
                return;
            },
        };

        #[cfg(feature = "router")]
        some_lib_tables::route::init(app_cx);

        let http_client = std::sync::Arc::new(reqwest_client::ReqwestClient::new());
        app_cx.set_http_client(http_client);

        app_cx
            .spawn(async move |cx| {
                let ready = readiness.await;
                if !ready.diagnostics.is_empty() {
                    eprintln!(
                        "table example Storybook preferences initialized with diagnostics: {:?}",
                        ready.diagnostics
                    );
                }

                cx.update(|app_cx| {
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
            })
            .detach();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, es_fluent::EsFluent)]
    #[fluent(domain = "gpui-table-component")]
    enum ResetFiltersFtl {
        Reset,
    }

    #[test]
    fn startup_contract_uses_a_stable_example_consumer() {
        let options = storybook_options().expect("example consumer id should be valid");

        assert_eq!(options.consumer_id.as_ref(), CONSUMER_ID);
        assert_eq!(options.fallback_language, Languages::default());
    }

    #[gpui::test]
    async fn disabled_french_startup_applies_domain_component_and_core_locales(
        cx: &mut gpui::TestAppContext,
    ) {
        cx.executor().allow_parking();
        let readiness = cx.update(|cx| {
            gpui_storybook::init(
                cx,
                storybook_options()
                    .expect("example consumer id should be valid")
                    .with_persistence(gpui_storybook::PersistenceMode::Disabled)
                    .with_overrides(gpui_storybook::PreferenceOverrides {
                        language: Some(Languages::FrFr),
                        ..Default::default()
                    }),
            )
            .expect("table example Storybook startup should initialize")
        });

        let ready = readiness.await;
        assert_eq!(
            ready.persistence_status,
            gpui_storybook::PersistenceStatus::Ready
        );
        assert!(ready.diagnostics.is_empty());

        let (source, language, domain_user, component_reset, core_true) = cx.update(|cx| {
            let state = gpui_storybook::try_preference_state(cx)
                .expect("preference state should be installed after initialization");
            let domain_user =
                gpui_table_component::i18n::localize_label::<some_lib::structs::user::User>(cx);
            let component_reset =
                gpui_table_component::i18n::localize_message(cx, &ResetFiltersFtl::Reset);
            let core_true = <bool as gpui_table::filter::Filterable>::options()
                .into_iter()
                .find(|option| option.value == "true")
                .expect("the boolean filter should include true")
                .label;
            (
                state.resolved.language.source,
                state.resolved.language.language.to_string(),
                domain_user,
                component_reset,
                core_true,
            )
        });

        assert_eq!(source, gpui_storybook::LanguageSource::Override);
        assert_eq!(language, "fr-FR");
        assert_eq!(domain_user, "Utilisateur");
        assert_eq!(component_reset, "Réinitialiser");
        assert_eq!(core_true, "Vrai");
    }
}
