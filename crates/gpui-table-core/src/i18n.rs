use es_fluent::{
    FluentArgs, FluentLabel, FluentLocalizer, FluentLocalizerExt as _, FluentMessage,
    registry::{StaticFluentDomain, StaticFluentEntryId},
};
use es_fluent_manager_embedded::{EmbeddedI18n, EmbeddedInitError};
use std::sync::{Mutex, OnceLock};

const FALLBACK_LANGUAGE: &str = "en";

es_fluent_manager_embedded::define_i18n_module!();

static I18N: OnceLock<EmbeddedI18n> = OnceLock::new();
static ACTIVE_LANGUAGE: Mutex<Option<es_fluent::unic_langid::LanguageIdentifier>> =
    Mutex::new(None);

struct FallbackLocalizer;

impl FluentLocalizer for FallbackLocalizer {
    fn localize<'a>(
        &self,
        id: StaticFluentEntryId,
        _args: Option<&FluentArgs<'a>>,
    ) -> Option<String> {
        Some(id.as_ref().to_string())
    }

    fn localize_in_domain<'a>(
        &self,
        _domain: StaticFluentDomain,
        id: StaticFluentEntryId,
        _args: Option<&FluentArgs<'a>>,
    ) -> Option<String> {
        Some(id.as_ref().to_string())
    }
}

fn fallback_language() -> es_fluent::unic_langid::LanguageIdentifier {
    FALLBACK_LANGUAGE
        .parse()
        .expect("gpui-table-core fallback language must be a valid language identifier")
}

fn i18n() -> Result<&'static EmbeddedI18n, EmbeddedInitError> {
    if I18N.get().is_none() {
        let i18n = EmbeddedI18n::try_new_with_language(fallback_language())?;
        let _ = I18N.set(i18n);
    }

    Ok(I18N
        .get()
        .expect("gpui-table-core i18n should be initialized"))
}

fn mark_language_active(language: &es_fluent::unic_langid::LanguageIdentifier) -> bool {
    let mut active_language = ACTIVE_LANGUAGE
        .lock()
        .unwrap_or_else(|error| error.into_inner());

    if active_language.as_ref() == Some(language) {
        return true;
    }

    *active_language = Some(language.clone());
    false
}

/// Select the locale used by built-in core filter labels.
pub fn set_locale(locale: impl AsRef<str>) {
    let Ok(language) = locale
        .as_ref()
        .parse::<es_fluent::unic_langid::LanguageIdentifier>()
    else {
        return;
    };

    if mark_language_active(&language) {
        return;
    }

    if let Ok(i18n) = i18n() {
        let _ = i18n.select_language(language);
    }
}

/// Localize a typed Fluent message through the core embedded i18n context.
pub fn localize_message<T>(message: &T) -> String
where
    T: FluentMessage + ?Sized,
{
    match i18n() {
        Ok(i18n) => i18n.localize_message(message),
        Err(_) => FallbackLocalizer.localize_message(message),
    }
}

/// Render a type label without consulting GPUI component state.
pub fn fallback_label<T>() -> String
where
    T: FluentLabel,
{
    es_fluent::fallback_label::<T>()
}

#[cfg(test)]
mod tests {
    use super::{
        FallbackLocalizer, fallback_label, fallback_language, i18n, mark_language_active,
        set_locale,
    };
    use es_fluent::{
        FluentLabel, FluentLocalizer as _,
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
        let english = fallback_language();
        let french = "fr".parse().unwrap();

        assert!(i18n().is_ok());
        assert!(!mark_language_active(&english));
        assert!(mark_language_active(&english));
        assert!(!mark_language_active(&french));
        assert!(mark_language_active(&french));

        set_locale("not a locale");
        set_locale("en");
        set_locale("en");
        set_locale("fr");
    }

    #[test]
    fn fallback_localizer_preserves_ids_and_humanizes_type_labels() {
        let domain = StaticFluentDomain::try_new("gpui-table-core").unwrap();
        let id = StaticFluentEntryId::try_new("purchase_order_label").unwrap();

        assert_eq!(
            FallbackLocalizer.localize(id, None).as_deref(),
            Some("purchase_order_label")
        );
        assert_eq!(
            FallbackLocalizer
                .localize_in_domain(domain, id, None)
                .as_deref(),
            Some("purchase_order_label")
        );
        assert_eq!(fallback_label::<PurchaseOrder>(), "Purchase Order");
    }
}
