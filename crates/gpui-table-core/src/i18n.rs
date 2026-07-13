use es_fluent::{FluentLabel, FluentMessage};
use es_fluent_manager_embedded::{EmbeddedI18n, EmbeddedInitError, LocalizationError};
use std::sync::{Mutex, OnceLock};

const DEFAULT_LANGUAGE: &str = "en";

es_fluent_manager_embedded::define_i18n_module!();

static I18N: OnceLock<EmbeddedI18n> = OnceLock::new();
static ACTIVE_LANGUAGE: Mutex<Option<es_fluent::unic_langid::LanguageIdentifier>> =
    Mutex::new(None);

/// Errors produced while selecting the process-global table-core locale.
#[derive(Debug, thiserror::Error)]
pub enum LocaleError {
    /// The supplied locale is not a valid Unicode language identifier.
    #[error("invalid gpui-table-core locale `{locale}`")]
    InvalidLocale {
        /// The rejected locale string.
        locale: String,
        /// The language identifier parse error.
        #[source]
        source: es_fluent::unic_langid::LanguageIdentifierError,
    },
    /// The embedded localization manager could not be initialized.
    #[error("failed to initialize gpui-table-core localization")]
    Initialization(#[from] EmbeddedInitError),
    /// The embedded localization manager rejected the requested language.
    #[error("failed to select gpui-table-core locale")]
    Selection(#[from] LocalizationError),
}

fn default_language() -> es_fluent::unic_langid::LanguageIdentifier {
    DEFAULT_LANGUAGE
        .parse()
        .expect("gpui-table-core default language must be a valid language identifier")
}

fn i18n() -> Result<&'static EmbeddedI18n, EmbeddedInitError> {
    if I18N.get().is_none() {
        let i18n = EmbeddedI18n::try_new_with_language(default_language())?;
        let _ = I18N.set(i18n);
    }

    Ok(I18N
        .get()
        .expect("gpui-table-core i18n should be initialized"))
}

fn language_is_active(language: &es_fluent::unic_langid::LanguageIdentifier) -> bool {
    ACTIVE_LANGUAGE
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .as_ref()
        == Some(language)
}

fn record_active_language(language: es_fluent::unic_langid::LanguageIdentifier) {
    *ACTIVE_LANGUAGE
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(language);
}

/// Select the locale used by built-in core filter labels.
///
/// # Errors
///
/// Returns [`LocaleError`] when the locale is invalid, localization cannot be
/// initialized, or the requested language is unsupported.
pub fn set_locale(locale: impl AsRef<str>) -> Result<(), LocaleError> {
    let locale = locale.as_ref();
    let language = locale
        .parse()
        .map_err(|source| LocaleError::InvalidLocale {
            locale: locale.to_owned(),
            source,
        })?;

    if language_is_active(&language) {
        return Ok(());
    }

    i18n()?.select_language(language.clone())?;
    record_active_language(language);
    Ok(())
}

/// Localize a typed Fluent message through the core embedded i18n context.
///
/// # Panics
///
/// Panics when localization cannot initialize or the typed resource is missing.
pub fn localize_message<T>(message: &T) -> String
where
    T: FluentMessage + ?Sized,
{
    i18n()
        .unwrap_or_else(|error| {
            panic!("failed to initialize gpui-table-core localization: {error}")
        })
        .localize_message(message)
}

/// Localize a type label through the core embedded i18n context.
///
/// # Panics
///
/// Panics when localization cannot initialize or the typed label is missing.
pub fn localize_label<T>() -> String
where
    T: FluentLabel,
{
    T::localize_label(i18n().unwrap_or_else(|error| {
        panic!("failed to initialize gpui-table-core localization: {error}")
    }))
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_LANGUAGE, LocaleError, default_language, i18n, localize_label, set_locale,
    };
    use es_fluent::{
        FluentLabel,
        registry::{StaticFluentDomain, StaticFluentEntryId},
    };

    struct PurchaseOrder;

    impl FluentLabel for PurchaseOrder {
        fn fluent_label_domain() -> StaticFluentDomain {
            StaticFluentDomain::try_new("gpui-table-core").unwrap()
        }

        fn fluent_label_id() -> StaticFluentEntryId {
            StaticFluentEntryId::try_new("purchase_order_label").unwrap()
        }
    }

    #[test]
    fn embedded_i18n_initializes_and_locale_selection_is_idempotent() {
        assert!(i18n().is_ok());
        assert!(set_locale("en").is_ok());
        assert!(set_locale("en").is_ok());
        assert!(set_locale("fr-FR").is_ok());
        assert!(set_locale("en").is_ok());
        assert_eq!(default_language().to_string(), DEFAULT_LANGUAGE);
    }

    #[test]
    fn invalid_locale_is_a_typed_error() {
        assert!(matches!(
            set_locale("not a locale"),
            Err(LocaleError::InvalidLocale { locale, .. }) if locale == "not a locale"
        ));
    }

    #[test]
    #[should_panic(expected = "missing Fluent label `purchase_order_label`")]
    fn missing_type_label_is_not_humanized() {
        localize_label::<PurchaseOrder>();
    }
}
