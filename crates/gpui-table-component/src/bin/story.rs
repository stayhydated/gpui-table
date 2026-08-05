use gpui_storybook::{Assets, ConsumerId, Gallery, StorybookOptions};
use gpui_table_component::i18n::{self, Languages};

// bring the stories in scope for inventory
#[allow(unused_imports, clippy::single_component_path_imports)]
use gpui_table_component;

const CONSUMER_ID: &str = "gpui-table-component-story";

fn storybook_options() -> Result<StorybookOptions<Languages>, gpui_storybook::ConsumerIdError> {
    Ok(StorybookOptions::new(
        ConsumerId::new(CONSUMER_ID)?,
        Languages::default(),
        i18n::apply_locale,
    ))
}

fn main() {
    let app = gpui_platform::application().with_assets(Assets);
    let name_arg = std::env::args().nth(1);

    app.run(move |app_cx| {
        let options = match storybook_options() {
            Ok(options) => options,
            Err(error) => {
                eprintln!("invalid table component Storybook consumer id: {error}");
                app_cx.quit();
                return;
            },
        };
        let readiness = match gpui_storybook::init(app_cx, options) {
            Ok(readiness) => readiness,
            Err(error) => {
                eprintln!("failed to initialize table component Storybook: {error}");
                app_cx.quit();
                return;
            },
        };

        app_cx
            .spawn(async move |cx| {
                let ready = readiness.await;
                if !ready.diagnostics.is_empty() {
                    eprintln!(
                        "table component Storybook preferences initialized with diagnostics: {:?}",
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
    use gpui::AppContext as _;

    #[derive(Clone, Copy, es_fluent::EsFluent)]
    enum ResetFiltersFtl {
        Reset,
    }

    #[test]
    fn startup_contract_uses_a_stable_component_consumer() {
        let options = storybook_options().expect("component consumer id should be valid");

        assert_eq!(options.consumer_id.as_ref(), CONSUMER_ID);
        assert_eq!(options.fallback_language, Languages::default());
    }

    #[gpui::test]
    async fn disabled_french_startup_applies_locales_and_generates_registered_stories(
        cx: &mut gpui::TestAppContext,
    ) {
        cx.executor().allow_parking();
        let readiness = cx.update(|cx| {
            gpui_storybook::init(
                cx,
                storybook_options()
                    .expect("component consumer id should be valid")
                    .with_persistence(gpui_storybook::PersistenceMode::Disabled)
                    .with_overrides(gpui_storybook::PreferenceOverrides {
                        language: Some(Languages::FrFr),
                        ..Default::default()
                    }),
            )
            .expect("component Storybook startup should initialize")
        });

        let ready = readiness.await;
        assert_eq!(
            ready.persistence_status,
            gpui_storybook::PersistenceStatus::Ready
        );
        assert!(ready.diagnostics.is_empty());

        let (source, language, component_reset, core_true) = cx.update(|cx| {
            let state = gpui_storybook::try_preference_state(cx)
                .expect("preference state should be installed after initialization");
            let component_reset =
                gpui_table_component::i18n::localize_message(cx, &ResetFiltersFtl::Reset);
            let core_true = <bool as gpui_table_core::filter::Filterable>::options()
                .into_iter()
                .find(|option| option.value == "true")
                .expect("the boolean filter should include true")
                .label;
            (
                state.resolved.language.source,
                state.resolved.language.language.to_string(),
                component_reset,
                core_true,
            )
        });

        assert_eq!(source, gpui_storybook::LanguageSource::Override);
        assert_eq!(language, "fr-FR");
        assert_eq!(component_reset, "Réinitialiser");
        assert_eq!(core_true, "Vrai");

        cx.update(|cx| {
            cx.open_window(Default::default(), |window, cx| {
                let story_keys = gpui_storybook::generate_stories(window, cx)
                    .into_iter()
                    .map(|story| {
                        story
                            .read(cx)
                            .story_key()
                            .expect("registered component story should have a stable key")
                            .as_str()
                    })
                    .collect::<Vec<_>>();

                assert_eq!(
                    story_keys,
                    [
                        "gpui-table-component-DateRangeFilterStory",
                        "gpui-table-component-FacetedFilterStory",
                        "gpui-table-component-NumberRangeFilterStory",
                        "gpui-table-component-ResetFiltersStory",
                        "gpui-table-component-TableStatusBarStory",
                        "gpui-table-component-TextFilterStory",
                    ]
                );

                cx.new(|_| gpui::EmptyView)
            })
            .expect("component Storybook should generate stories in a test window");
        });
    }
}
