use gpui::Application;
use gpui_storybook::{ConsumerId, StorybookOptions, StorybookWindow};
use some_lib::i18n::{self, Languages};

#[cfg(feature = "router")]
pub mod route;
pub mod tables;

const CONSUMER_ID: &str = "gpui-table-some-lib-tables";

fn storybook_options() -> Result<StorybookOptions<Languages>, gpui_storybook::ConsumerIdError> {
    let options = StorybookOptions::new(
        ConsumerId::new(CONSUMER_ID)?,
        Languages::default(),
        i18n::apply_locale,
    );

    #[cfg(target_family = "wasm")]
    let options = options.with_persistence(gpui_storybook::PersistenceMode::Disabled);

    Ok(options)
}

pub fn run_storybook(app: Application) {
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
        route::init(app_cx);

        #[cfg(not(target_family = "wasm"))]
        {
            let http_client = std::sync::Arc::new(reqwest_client::ReqwestClient::new());
            app_cx.set_http_client(http_client);
        }

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
                    gpui_storybook::create_storybook_window(
                        &format!("{} - Stories", env!("CARGO_PKG_NAME")),
                        move |window, cx| {
                            let stories = gpui_storybook::generate_stories(window, cx);
                            assert!(
                                !stories.is_empty(),
                                "table example Storybook requires linked stories"
                            );
                            StorybookWindow::new(stories)
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

    #[derive(Clone, Copy)]
    enum ResetFiltersFtl {
        Reset,
    }

    impl es_fluent::FluentMessage for ResetFiltersFtl {
        fn to_fluent_string_with(
            &self,
            localize: &mut es_fluent::FluentMessageLookup<'_>,
        ) -> String {
            localize(
                es_fluent::registry::__macro::static_message_key(
                    "gpui-table-component",
                    es_fluent::registry::__macro::static_domain("gpui-table-component"),
                    es_fluent::registry::__macro::static_entry_id("reset_filters_ftl-Reset"),
                ),
                None,
            )
        }
    }

    #[test]
    fn startup_contract_uses_a_stable_example_consumer() {
        let options = storybook_options().expect("example consumer id should be valid");

        assert_eq!(options.consumer_id.as_ref(), CONSUMER_ID);
        assert_eq!(options.fallback_language, Languages::default());
    }

    #[test]
    fn binary_links_expected_story_registrations() {
        let mut story_keys =
            gpui_storybook::__inventory::iter::<gpui_storybook::__registry::StoryEntry>()
                .filter(|entry| entry.crate_name == env!("CARGO_PKG_NAME"))
                .map(|entry| entry.key.as_str())
                .collect::<Vec<_>>();
        story_keys.sort_unstable();

        assert_eq!(
            story_keys,
            [
                "some-lib-tables-ItemTableStory",
                "some-lib-tables-UserTableStory",
            ]
        );
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
