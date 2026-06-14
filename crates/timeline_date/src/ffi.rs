use crate::{
    HourCycle, TimelineDateError, TimelineDateFormatter, TimelineDateOptions, TimelineDateStyle,
};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum TimelineDateFfiError {
    #[error("invalid timestamp")]
    InvalidTimestamp,
    #[error("invalid timezone")]
    InvalidTimezone,
    #[error("invalid locale")]
    InvalidLocale,
    #[error("invalid hour cycle")]
    InvalidHourCycle,
    #[error("i18n initialization failed")]
    I18nInit,
    #[error("i18n formatting failed")]
    I18nFormat,
    #[error("formatting unsupported")]
    FormattingUnsupported,
    #[error("locale data unavailable")]
    LocaleDataUnavailable,
    #[error("internal error")]
    Internal,
}

impl From<TimelineDateError> for TimelineDateFfiError {
    fn from(error: TimelineDateError) -> Self {
        match error {
            TimelineDateError::InvalidTimestamp(_) => Self::InvalidTimestamp,
            TimelineDateError::InvalidTimezone(_) => Self::InvalidTimezone,
            TimelineDateError::InvalidLocale(_) => Self::InvalidLocale,
            TimelineDateError::I18nInit(_) => Self::I18nInit,
            TimelineDateError::I18nFormat(_) => Self::I18nFormat,
            TimelineDateError::FormattingUnsupported(_) => Self::FormattingUnsupported,
            TimelineDateError::LocaleDataUnavailable(_) => Self::LocaleDataUnavailable,
            TimelineDateError::Internal(_) => Self::Internal,
        }
    }
}

#[uniffi::export]
pub fn format_timeline_date_for_feed(
    event_unix_ms: i64,
    now_unix_ms: i64,
    timezone: String,
    locale_preferences_csv: String,
    hour_cycle: String,
) -> Result<String, TimelineDateFfiError> {
    format_feed_label_inner(
        event_unix_ms,
        now_unix_ms,
        timezone,
        locale_preferences_csv,
        hour_cycle,
    )
}

#[uniffi::export]
pub fn format_feed_label(
    now_unix_ms: i64,
    event_unix_ms: i64,
    timezone: String,
    locale_preferences_csv: String,
    hour_cycle: String,
) -> Result<String, TimelineDateFfiError> {
    format_timeline_date_for_feed(
        event_unix_ms,
        now_unix_ms,
        timezone,
        locale_preferences_csv,
        hour_cycle,
    )
}

fn format_feed_label_inner(
    event_unix_ms: i64,
    now_unix_ms: i64,
    timezone: String,
    locale_preferences_csv: String,
    hour_cycle: String,
) -> Result<String, TimelineDateFfiError> {
    let options = TimelineDateOptions::new(now_unix_ms, timezone)
        .with_locale_preferences(parse_locale_preferences_csv(&locale_preferences_csv))
        .with_hour_cycle(parse_hour_cycle(&hour_cycle)?);
    let formatter = TimelineDateFormatter::new(options).map_err(TimelineDateFfiError::from)?;
    formatter
        .format_millis(event_unix_ms, TimelineDateStyle::Feed)
        .map_err(TimelineDateFfiError::from)
}

fn parse_locale_preferences_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

fn parse_hour_cycle(value: &str) -> Result<HourCycle, TimelineDateFfiError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "" | "locale" | "default" | "locale-default" | "locale_default" => {
            Ok(HourCycle::LocaleDefault)
        }
        "h12" | "12" => Ok(HourCycle::H12),
        "h24" | "24" => Ok(HourCycle::H24),
        _ => Err(TimelineDateFfiError::InvalidHourCycle),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TimelineDateFfiError, format_feed_label, format_timeline_date_for_feed, parse_hour_cycle,
        parse_locale_preferences_csv,
    };
    use crate::{HourCycle, TimelineDateError};

    #[test]
    fn parses_csv_locale_preferences() {
        assert_eq!(
            parse_locale_preferences_csv(" fr-CA, es-MX ,, en "),
            vec!["fr-CA", "es-MX", "en"]
        );
        assert!(parse_locale_preferences_csv(" , , ").is_empty());
    }

    #[test]
    fn parses_hour_cycle_values() {
        assert_eq!(
            parse_hour_cycle("").expect("empty"),
            HourCycle::LocaleDefault
        );
        assert_eq!(
            parse_hour_cycle("locale-default").expect("default"),
            HourCycle::LocaleDefault
        );
        assert_eq!(parse_hour_cycle("H12").expect("h12"), HourCycle::H12);
        assert_eq!(parse_hour_cycle("24").expect("h24"), HourCycle::H24);
        assert_eq!(
            parse_hour_cycle("midday"),
            Err(TimelineDateFfiError::InvalidHourCycle)
        );
    }

    #[test]
    fn formats_timeline_date_for_feed_with_canonical_order() {
        assert_eq!(
            format_timeline_date_for_feed(
                120_000,
                600_000,
                "UTC".to_owned(),
                "es-MX, fr".to_owned(),
                "h24".to_owned(),
            )
            .expect("label"),
            "hace 8 min"
        );
    }

    #[test]
    fn compatibility_alias_delegates_old_argument_order() {
        assert_eq!(
            format_feed_label(
                600_000,
                120_000,
                "UTC".to_owned(),
                "es-MX, fr".to_owned(),
                "h24".to_owned(),
            ),
            format_timeline_date_for_feed(
                120_000,
                600_000,
                "UTC".to_owned(),
                "es-MX, fr".to_owned(),
                "h24".to_owned(),
            )
        );
    }

    #[test]
    fn canonical_order_does_not_silently_match_swapped_arguments() {
        assert_ne!(
            format_timeline_date_for_feed(
                120_000,
                600_000,
                "UTC".to_owned(),
                "en".to_owned(),
                "h24".to_owned(),
            )
            .expect("canonical"),
            format_timeline_date_for_feed(
                600_000,
                120_000,
                "UTC".to_owned(),
                "en".to_owned(),
                "h24".to_owned(),
            )
            .expect("swapped")
        );
    }

    #[test]
    fn propagates_invalid_timezone_and_locale() {
        assert_eq!(
            format_timeline_date_for_feed(
                0,
                0,
                "Mars/Base".to_owned(),
                "en".to_owned(),
                "default".to_owned(),
            ),
            Err(TimelineDateFfiError::InvalidTimezone)
        );
        assert_eq!(
            format_timeline_date_for_feed(
                0,
                0,
                "UTC".to_owned(),
                "en--US".to_owned(),
                "default".to_owned(),
            ),
            Err(TimelineDateFfiError::InvalidLocale)
        );
    }

    #[test]
    fn facade_rejects_invalid_hour_cycle() {
        assert_eq!(
            format_timeline_date_for_feed(
                0,
                0,
                "UTC".to_owned(),
                "en".to_owned(),
                "midday".to_owned(),
            ),
            Err(TimelineDateFfiError::InvalidHourCycle)
        );
    }

    #[test]
    fn maps_rust_errors_to_stable_ffi_cases() {
        let cases = [
            (
                TimelineDateError::InvalidTimestamp(1),
                TimelineDateFfiError::InvalidTimestamp,
            ),
            (
                TimelineDateError::InvalidTimezone("Mars/Base".to_owned()),
                TimelineDateFfiError::InvalidTimezone,
            ),
            (
                TimelineDateError::InvalidLocale("bad".to_owned()),
                TimelineDateFfiError::InvalidLocale,
            ),
            (
                TimelineDateError::I18nInit("catalog".to_owned()),
                TimelineDateFfiError::I18nInit,
            ),
            (
                TimelineDateError::I18nFormat("message".to_owned()),
                TimelineDateFfiError::I18nFormat,
            ),
            (
                TimelineDateError::FormattingUnsupported("feature".to_owned()),
                TimelineDateFfiError::FormattingUnsupported,
            ),
            (
                TimelineDateError::LocaleDataUnavailable("fr".to_owned()),
                TimelineDateFfiError::LocaleDataUnavailable,
            ),
            (
                TimelineDateError::Internal("state".to_owned()),
                TimelineDateFfiError::Internal,
            ),
        ];

        for (source, expected) in cases {
            assert_eq!(TimelineDateFfiError::from(source), expected);
        }
    }
}
