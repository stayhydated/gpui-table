use es_fluent::EsFluent;
use es_fluent::unic_langid::LanguageIdentifier;
use es_fluent_lang::es_fluent_language;
use strum::EnumIter;

es_fluent_manager_embedded::define_i18n_module!();

#[es_fluent_language]
#[derive(Clone, Copy, Debug, EnumIter, EsFluent, PartialEq)]
pub enum Languages {}

/// Applies Storybook's resolved locale to the domain, component, and table-core contexts.
///
/// # Errors
///
/// Returns an error when the embedded application resources cannot initialize
/// or a table localization context rejects the locale.
pub fn apply_locale(
    language: Languages,
    cx: &mut gpui_kit::App,
) -> Result<(), gpui_table_component::i18n::LocaleError> {
    let _linked_module = &SOME_LIB_I18N_MODULE;
    let language: LanguageIdentifier = language.into();
    gpui_es_fluent::replace_with_language(cx, language.clone())
        .map_err(gpui_es_fluent::ComponentLocaleError::Initialization)?;
    gpui_table_component::i18n::set_locale(cx, language.to_string())
}
