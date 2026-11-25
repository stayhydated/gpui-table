use es_fluent::unic_langid::LanguageIdentifier;
use iso_types::iso_locale;

es_fluent_manager_embedded::define_i18n_module!();

#[iso_locale(custom)]
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
