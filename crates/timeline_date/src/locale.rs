use crate::{TimelineDateError, TimelineDateResult};

#[cfg(not(feature = "mf2"))]
const DEFAULT_LOCALE: &str = "en";

#[cfg(feature = "mf2")]
pub(crate) fn select_locale(preferences: &[String]) -> TimelineDateResult<String> {
    use mf2_i18n::negotiate_lookup;

    let requested = parse_requested(preferences)?;
    let supported = supported_locales(crate::mf2::SUPPORTED_LOCALES)?;
    let default_locale = parse_locale(crate::mf2::DEFAULT_LOCALE)?;
    let negotiation = negotiate_lookup(&requested, &supported, &default_locale);
    Ok(negotiation.selected.normalized().to_owned())
}

#[cfg(not(feature = "mf2"))]
pub(crate) fn select_locale(_preferences: &[String]) -> TimelineDateResult<String> {
    Ok(DEFAULT_LOCALE.to_owned())
}

#[cfg(feature = "mf2")]
fn parse_requested(preferences: &[String]) -> TimelineDateResult<Vec<mf2_i18n::LanguageTag>> {
    let mut requested = Vec::new();
    let mut seen = Vec::<String>::new();

    for preference in preferences {
        let trimmed = preference.trim();
        if trimmed.is_empty() {
            continue;
        }

        let tag = mf2_i18n::LanguageTag::parse(trimmed)
            .map_err(|_| TimelineDateError::InvalidLocale(trimmed.to_owned()))?;
        let normalized = tag.normalized().to_owned();

        if seen.iter().any(|value| value == &normalized) {
            continue;
        }

        seen.push(normalized);
        requested.push(tag);
    }

    Ok(requested)
}

#[cfg(feature = "mf2")]
fn supported_locales(locales: &[&str]) -> TimelineDateResult<Vec<mf2_i18n::LanguageTag>> {
    locales.iter().map(|locale| parse_locale(locale)).collect()
}

#[cfg(feature = "mf2")]
fn parse_locale(locale: &str) -> TimelineDateResult<mf2_i18n::LanguageTag> {
    mf2_i18n::LanguageTag::parse(locale)
        .map_err(|_| TimelineDateError::Internal(format!("invalid supported locale: {locale}")))
}

#[cfg(all(test, feature = "mf2"))]
mod tests {
    use super::{parse_locale, select_locale};
    use crate::TimelineDateError;

    fn select(preferences: &[&str]) -> Result<String, TimelineDateError> {
        let preferences = preferences
            .iter()
            .map(|preference| preference.to_string())
            .collect::<Vec<_>>();
        select_locale(&preferences)
    }

    #[test]
    fn exact_match_selects_supported_locale() {
        assert_eq!(select(&["fr"]).expect("locale"), "fr");
    }

    #[test]
    fn lookup_truncates_region_to_supported_language() {
        assert_eq!(select(&["fr-CA"]).expect("locale"), "fr");
    }

    #[test]
    fn fallback_uses_default_when_no_supported_locale_matches() {
        assert_eq!(select(&["ja-JP"]).expect("locale"), "en");
    }

    #[test]
    fn empty_preferences_use_default_locale() {
        assert_eq!(select(&[]).expect("locale"), "en");
        assert_eq!(select(&["", " "]).expect("locale"), "en");
    }

    #[test]
    fn whitespace_and_case_are_normalized_before_lookup() {
        assert_eq!(select(&[" ES-mx "]).expect("locale"), "es");
    }

    #[test]
    fn malformed_locale_returns_typed_error() {
        let error = select(&["en--US"]).expect_err("invalid locale");
        assert_eq!(error, TimelineDateError::InvalidLocale("en--US".to_owned()));
    }

    #[test]
    fn duplicate_preferences_are_ignored_after_normalization() {
        assert_eq!(select(&["fr-CA", "fr-ca", "es"]).expect("locale"), "fr");
    }

    #[test]
    fn invalid_supported_locale_maps_to_internal_error() {
        let error = parse_locale("").expect_err("invalid locale");
        assert_eq!(
            error,
            TimelineDateError::Internal("invalid supported locale: ".to_owned())
        );
    }
}
