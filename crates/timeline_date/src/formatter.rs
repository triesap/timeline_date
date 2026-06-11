use std::sync::Arc;

use crate::{
    TimelineDateBucket, TimelineDateOptions, TimelineDateResult, TimelineDateStyle, classify,
    locale, time,
};

#[derive(Clone, Debug)]
pub struct TimelineDateFormatter {
    inner: Arc<TimelineDateFormatterInner>,
}

#[derive(Debug)]
struct TimelineDateFormatterInner {
    options: TimelineDateOptions,
    selected_locale: String,
    clock: time::ValidatedClock,
}

impl TimelineDateFormatter {
    pub fn new(options: TimelineDateOptions) -> TimelineDateResult<Self> {
        let selected_locale = locale::select_locale(&options.locale_preferences)?;
        let clock = time::ValidatedClock::new(options.now_unix_ms, &options.timezone)?;
        #[cfg(feature = "mf2")]
        crate::mf2::embedded_runtime()?;
        Ok(Self {
            inner: Arc::new(TimelineDateFormatterInner {
                options,
                selected_locale,
                clock,
            }),
        })
    }

    pub fn options(&self) -> &TimelineDateOptions {
        &self.inner.options
    }

    pub fn selected_locale(&self) -> &str {
        &self.inner.selected_locale
    }

    pub fn classify_millis(
        &self,
        event_unix_ms: i64,
        style: TimelineDateStyle,
    ) -> TimelineDateResult<TimelineDateBucket> {
        match style {
            TimelineDateStyle::Feed => classify::classify_feed_millis(
                event_unix_ms,
                &self.inner.clock,
                self.inner.options.future_policy,
            ),
            TimelineDateStyle::Detail => {
                classify::classify_fixed_millis(event_unix_ms, TimelineDateBucket::Detail)
            }
            TimelineDateStyle::Audit => {
                classify::classify_fixed_millis(event_unix_ms, TimelineDateBucket::Audit)
            }
        }
    }

    pub fn format_millis(
        &self,
        event_unix_ms: i64,
        style: TimelineDateStyle,
    ) -> TimelineDateResult<String> {
        let bucket = self.classify_millis(event_unix_ms, style)?;
        #[cfg(feature = "mf2")]
        {
            crate::mf2::format_millis(self, event_unix_ms, style, bucket)
        }
        #[cfg(not(feature = "mf2"))]
        {
            let _ = (event_unix_ms, style, bucket);
            Err(crate::TimelineDateError::FormattingUnsupported(
                "formatting requires the mf2 feature".to_owned(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TimelineDateFormatter;
    use crate::{HourCycle, TimelineDateError, TimelineDateOptions};

    #[test]
    #[cfg(all(feature = "jiff", feature = "mf2"))]
    fn new_stores_options() {
        let options = TimelineDateOptions::new(1_780_958_400_000, "America/Vancouver")
            .with_locale_preferences(["fr-CA", "fr"])
            .with_hour_cycle(HourCycle::H24);
        let formatter = TimelineDateFormatter::new(options.clone()).expect("formatter");
        assert_eq!(formatter.options(), &options);
        assert_eq!(formatter.selected_locale(), "fr");
    }

    #[test]
    #[cfg(feature = "jiff")]
    fn cloned_formatter_keeps_options() {
        let options = TimelineDateOptions::new(1, "UTC");
        let formatter = TimelineDateFormatter::new(options.clone()).expect("formatter");
        let cloned = formatter.clone();
        assert_eq!(cloned.options(), &options);
        assert_eq!(cloned.selected_locale(), "en");
    }

    #[test]
    #[cfg(all(feature = "jiff", feature = "mf2"))]
    fn independent_formatters_keep_selected_locale() {
        let french = TimelineDateFormatter::new(
            TimelineDateOptions::new(1, "UTC").with_locale_preferences(["fr-CA"]),
        )
        .expect("french formatter");
        let spanish = TimelineDateFormatter::new(
            TimelineDateOptions::new(1, "UTC").with_locale_preferences(["es-MX"]),
        )
        .expect("spanish formatter");

        assert_eq!(french.selected_locale(), "fr");
        assert_eq!(spanish.selected_locale(), "es");
    }

    #[test]
    fn formatter_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TimelineDateFormatter>();
    }

    #[test]
    #[cfg(all(feature = "jiff", feature = "mf2"))]
    fn new_rejects_malformed_locale() {
        let options = TimelineDateOptions::new(1, "UTC").with_locale_preferences(["en--US"]);
        let error = TimelineDateFormatter::new(options).expect_err("invalid locale");
        assert_eq!(error, TimelineDateError::InvalidLocale("en--US".to_owned()));
    }

    #[test]
    #[cfg(feature = "jiff")]
    fn new_rejects_invalid_timezone() {
        let options = TimelineDateOptions::new(1, "Mars/Base");
        let error = TimelineDateFormatter::new(options).expect_err("invalid timezone");
        assert_eq!(
            error,
            TimelineDateError::InvalidTimezone("Mars/Base".to_owned())
        );
    }

    #[test]
    #[cfg(feature = "jiff")]
    fn new_rejects_invalid_now_timestamp() {
        let options = TimelineDateOptions::new(i64::MAX, "UTC");
        let error = TimelineDateFormatter::new(options).expect_err("invalid timestamp");
        assert_eq!(error, TimelineDateError::InvalidTimestamp(i64::MAX));
    }

    #[test]
    #[cfg(all(feature = "jiff", feature = "mf2"))]
    fn format_millis_uses_selected_locale_catalog() {
        let formatter = TimelineDateFormatter::new(
            TimelineDateOptions::new(600_000, "UTC").with_locale_preferences(["es-MX"]),
        )
        .expect("formatter");
        let label = formatter
            .format_millis(120_000, crate::TimelineDateStyle::Feed)
            .expect("label");
        assert_eq!(label, "hace 8 min");
    }

    #[test]
    #[cfg(all(feature = "jiff", feature = "mf2", not(feature = "icu")))]
    fn format_millis_maps_detail_error_and_audit_stable_output() {
        let formatter =
            TimelineDateFormatter::new(TimelineDateOptions::new(0, "UTC")).expect("formatter");
        let detail = formatter
            .format_millis(0, crate::TimelineDateStyle::Detail)
            .expect_err("detail");
        let audit = formatter
            .format_millis(0, crate::TimelineDateStyle::Audit)
            .expect("audit");
        assert_eq!(detail, expected_datetime_format_error());
        assert_eq!(audit, "1970-01-01T00:00:00.000 UTC");
    }

    #[test]
    #[cfg(all(feature = "jiff", feature = "mf2", feature = "icu"))]
    fn format_millis_formats_detail_and_audit_styles_with_icu() {
        let event = ms("2026-06-08T12:00:00Z");
        let formatter =
            TimelineDateFormatter::new(TimelineDateOptions::new(event, "UTC")).expect("formatter");
        let detail = formatter
            .format_millis(event, crate::TimelineDateStyle::Detail)
            .expect("detail");
        let audit = formatter
            .format_millis(event, crate::TimelineDateStyle::Audit)
            .expect("audit");

        for part in ["Monday", "June", "8", "2026"] {
            assert!(detail.contains(part), "{detail:?} should contain {part:?}");
        }
        assert_eq!(audit, "2026-06-08T12:00:00.000 UTC");
    }

    #[test]
    #[cfg(all(feature = "jiff", feature = "mf2", feature = "icu"))]
    fn localized_h24_goldens_cover_feed_detail_and_audit_surfaces() {
        let cases = [
            (
                "en",
                "2026-06-08T18:59:50Z",
                crate::TimelineDateStyle::Feed,
                "Just now",
            ),
            (
                "en",
                "2026-06-08T18:52:00Z",
                crate::TimelineDateStyle::Feed,
                "8 min ago",
            ),
            (
                "en",
                "2026-06-08T17:00:00Z",
                crate::TimelineDateStyle::Feed,
                "Today at 10:00",
            ),
            (
                "en",
                "2026-06-08T06:30:00Z",
                crate::TimelineDateStyle::Feed,
                "Yesterday at 23:30",
            ),
            (
                "en",
                "2026-06-02T19:00:00Z",
                crate::TimelineDateStyle::Feed,
                "Tuesday at 12:00",
            ),
            (
                "en",
                "2026-06-01T19:00:00Z",
                crate::TimelineDateStyle::Feed,
                "Jun 1 at 12:00",
            ),
            (
                "en",
                "2025-12-31T20:00:00Z",
                crate::TimelineDateStyle::Feed,
                "Dec 31, 2025",
            ),
            (
                "en",
                "2026-06-08T21:00:00Z",
                crate::TimelineDateStyle::Feed,
                "Jun 8, 2026, 14:00",
            ),
            (
                "en",
                "2026-06-08T19:00:00Z",
                crate::TimelineDateStyle::Detail,
                "Monday, June 8, 2026 at 12:00",
            ),
            (
                "en",
                "2026-06-08T19:00:00.250Z",
                crate::TimelineDateStyle::Audit,
                "2026-06-08T12:00:00.250 America/Vancouver",
            ),
            (
                "es",
                "2026-06-08T18:59:50Z",
                crate::TimelineDateStyle::Feed,
                "Ahora mismo",
            ),
            (
                "es",
                "2026-06-08T18:52:00Z",
                crate::TimelineDateStyle::Feed,
                "hace 8 min",
            ),
            (
                "es",
                "2026-06-08T17:00:00Z",
                crate::TimelineDateStyle::Feed,
                "Hoy a las 10:00",
            ),
            (
                "es",
                "2026-06-08T06:30:00Z",
                crate::TimelineDateStyle::Feed,
                "Ayer a las 23:30",
            ),
            (
                "es",
                "2026-06-02T19:00:00Z",
                crate::TimelineDateStyle::Feed,
                "martes a las 12:00",
            ),
            (
                "es",
                "2026-06-01T19:00:00Z",
                crate::TimelineDateStyle::Feed,
                "1 jun a las 12:00",
            ),
            (
                "es",
                "2025-12-31T20:00:00Z",
                crate::TimelineDateStyle::Feed,
                "31 dic 2025",
            ),
            (
                "es",
                "2026-06-08T21:00:00Z",
                crate::TimelineDateStyle::Feed,
                "8 jun 2026, 14:00",
            ),
            (
                "es",
                "2026-06-08T19:00:00Z",
                crate::TimelineDateStyle::Detail,
                "lunes, 8 de junio de 2026 a las 12:00",
            ),
            (
                "es",
                "2026-06-08T19:00:00.250Z",
                crate::TimelineDateStyle::Audit,
                "2026-06-08T12:00:00.250 America/Vancouver",
            ),
            (
                "fr",
                "2026-06-08T18:59:50Z",
                crate::TimelineDateStyle::Feed,
                "A l'instant",
            ),
            (
                "fr",
                "2026-06-08T18:52:00Z",
                crate::TimelineDateStyle::Feed,
                "il y a 8 min",
            ),
            (
                "fr",
                "2026-06-08T17:00:00Z",
                crate::TimelineDateStyle::Feed,
                "Aujourd'hui a 10:00",
            ),
            (
                "fr",
                "2026-06-08T06:30:00Z",
                crate::TimelineDateStyle::Feed,
                "Hier a 23:30",
            ),
            (
                "fr",
                "2026-06-02T19:00:00Z",
                crate::TimelineDateStyle::Feed,
                "mardi a 12:00",
            ),
            (
                "fr",
                "2026-06-01T19:00:00Z",
                crate::TimelineDateStyle::Feed,
                "1 juin a 12:00",
            ),
            (
                "fr",
                "2025-12-31T20:00:00Z",
                crate::TimelineDateStyle::Feed,
                "31 d\u{e9}c. 2025",
            ),
            (
                "fr",
                "2026-06-08T21:00:00Z",
                crate::TimelineDateStyle::Feed,
                "8 juin 2026, 14:00",
            ),
            (
                "fr",
                "2026-06-08T19:00:00Z",
                crate::TimelineDateStyle::Detail,
                "lundi 8 juin 2026 a 12:00",
            ),
            (
                "fr",
                "2026-06-08T19:00:00.250Z",
                crate::TimelineDateStyle::Audit,
                "2026-06-08T12:00:00.250 America/Vancouver",
            ),
        ];

        for (locale, event, style, expected) in cases {
            let formatter = TimelineDateFormatter::new(
                TimelineDateOptions::new(ms("2026-06-08T19:00:00Z"), "America/Vancouver")
                    .with_locale_preferences([locale])
                    .with_hour_cycle(HourCycle::H24),
            )
            .expect("formatter");
            assert_eq!(
                formatter.format_millis(ms(event), style).expect("label"),
                expected
            );
        }
    }

    #[test]
    #[cfg(all(feature = "jiff", feature = "mf2"))]
    fn format_millis_returns_classification_errors() {
        let formatter =
            TimelineDateFormatter::new(TimelineDateOptions::new(0, "UTC")).expect("formatter");
        let error = formatter
            .format_millis(i64::MAX, crate::TimelineDateStyle::Feed)
            .expect_err("invalid event");
        assert_eq!(error, TimelineDateError::InvalidTimestamp(i64::MAX));
    }

    #[test]
    #[cfg(all(feature = "jiff", feature = "mf2", not(feature = "icu")))]
    fn reduced_mode_formats_text_only_feed_messages() {
        let formatter = TimelineDateFormatter::new(TimelineDateOptions::new(600_000, "UTC"))
            .expect("formatter");
        assert_eq!(
            formatter
                .format_millis(590_000, crate::TimelineDateStyle::Feed)
                .expect("just now"),
            "Just now"
        );
        assert_eq!(
            formatter
                .format_millis(120_000, crate::TimelineDateStyle::Feed)
                .expect("minutes"),
            "8 min ago"
        );
    }

    #[test]
    #[cfg(all(feature = "jiff", feature = "mf2", not(feature = "icu")))]
    fn reduced_mode_date_time_formatting_is_typed() {
        let formatter =
            TimelineDateFormatter::new(TimelineDateOptions::new(0, "UTC")).expect("formatter");
        let error = formatter
            .format_millis(0, crate::TimelineDateStyle::Detail)
            .expect_err("detail");
        assert_eq!(
            error,
            TimelineDateError::I18nFormat(
                "unsupported: datetime formatting requires the icu feature for localized output"
                    .to_owned()
            )
        );
    }

    #[cfg(all(feature = "jiff", feature = "mf2", not(feature = "icu")))]
    fn expected_datetime_format_error() -> TimelineDateError {
        TimelineDateError::I18nFormat(format!(
            "unsupported: {}",
            crate::backend::datetime_unsupported_message()
        ))
    }

    #[cfg(all(feature = "jiff", feature = "mf2", feature = "icu"))]
    fn ms(value: &str) -> i64 {
        value
            .parse::<jiff::Timestamp>()
            .expect("timestamp")
            .as_millisecond()
    }
}
