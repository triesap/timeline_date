pub type TimelineDateResult<T> = Result<T, TimelineDateError>;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TimelineDateError {
    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(i64),
    #[error("invalid timezone: {0}")]
    InvalidTimezone(String),
    #[error("invalid locale: {0}")]
    InvalidLocale(String),
    #[error("i18n runtime initialization failed: {0}")]
    I18nInit(String),
    #[error("i18n formatting failed: {0}")]
    I18nFormat(String),
    #[error("formatting unsupported: {0}")]
    FormattingUnsupported(String),
    #[error("locale data unavailable: {0}")]
    LocaleDataUnavailable(String),
    #[error("internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::{TimelineDateError, TimelineDateResult};

    #[test]
    fn result_alias_accepts_timeline_error() {
        let result: TimelineDateResult<()> = Err(TimelineDateError::InvalidTimestamp(7));
        assert_eq!(result, Err(TimelineDateError::InvalidTimestamp(7)));
    }

    #[test]
    fn errors_render_stable_messages() {
        let cases = [
            (
                TimelineDateError::InvalidTimestamp(1),
                "invalid timestamp: 1",
            ),
            (
                TimelineDateError::InvalidTimezone("Mars/Base".to_owned()),
                "invalid timezone: Mars/Base",
            ),
            (
                TimelineDateError::InvalidLocale("not locale".to_owned()),
                "invalid locale: not locale",
            ),
            (
                TimelineDateError::I18nInit("missing catalog".to_owned()),
                "i18n runtime initialization failed: missing catalog",
            ),
            (
                TimelineDateError::I18nFormat("missing key".to_owned()),
                "i18n formatting failed: missing key",
            ),
            (
                TimelineDateError::FormattingUnsupported("date option".to_owned()),
                "formatting unsupported: date option",
            ),
            (
                TimelineDateError::LocaleDataUnavailable("fr-CA".to_owned()),
                "locale data unavailable: fr-CA",
            ),
            (
                TimelineDateError::Internal("catalog invariant".to_owned()),
                "internal error: catalog invariant",
            ),
        ];

        for (error, message) in cases {
            assert_eq!(error.to_string(), message);
        }
    }
}
