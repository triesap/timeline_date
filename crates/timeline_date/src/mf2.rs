include!(concat!(env!("OUT_DIR"), "/timeline_date_i18n_runtime.rs"));

use crate::{
    HourCycle, TimelineDateBucket, TimelineDateError, TimelineDateFormatter, TimelineDateResult,
    TimelineDateStyle,
};

pub(crate) fn format_millis(
    formatter: &TimelineDateFormatter,
    event_unix_ms: i64,
    style: TimelineDateStyle,
    bucket: TimelineDateBucket,
) -> TimelineDateResult<String> {
    format_selected(
        formatter.selected_locale(),
        &formatter.options().timezone,
        formatter.options().hour_cycle,
        event_unix_ms,
        style,
        bucket,
    )
}

fn format_selected(
    selected_locale: &str,
    timezone: &str,
    hour_cycle: HourCycle,
    event_unix_ms: i64,
    style: TimelineDateStyle,
    bucket: TimelineDateBucket,
) -> TimelineDateResult<String> {
    let key = message_key(style, bucket)?;
    let args = message_args(event_unix_ms, style, bucket, timezone);
    let backend = crate::backend::TimelineDateBackend::new(selected_locale, timezone, hour_cycle)?;
    format_key_with_backend(embedded_runtime()?, selected_locale, key, &args, &backend)
}

pub(crate) fn embedded_runtime() -> TimelineDateResult<&'static mf2_i18n::EmbeddedRuntime> {
    runtime().map_err(TimelineDateError::I18nInit)
}

fn message_key(
    style: TimelineDateStyle,
    bucket: TimelineDateBucket,
) -> TimelineDateResult<&'static str> {
    match style {
        TimelineDateStyle::Detail => Ok("timeline_date.detail_datetime"),
        TimelineDateStyle::Audit => Ok("timeline_date.audit_datetime"),
        TimelineDateStyle::Feed => match bucket {
            TimelineDateBucket::JustNow => Ok("timeline_date.just_now"),
            TimelineDateBucket::MinutesAgo { .. } => Ok("timeline_date.minutes_ago"),
            TimelineDateBucket::Today => Ok("timeline_date.today_at_time"),
            TimelineDateBucket::Yesterday => Ok("timeline_date.yesterday_at_time"),
            TimelineDateBucket::Weekday => Ok("timeline_date.weekday_at_time"),
            TimelineDateBucket::SameYear => Ok("timeline_date.same_year_at_time"),
            TimelineDateBucket::Older => Ok("timeline_date.older_date"),
            TimelineDateBucket::Future => Ok("timeline_date.future_at_datetime"),
            TimelineDateBucket::Detail | TimelineDateBucket::Audit => Err(
                TimelineDateError::Internal(format!("invalid feed bucket: {bucket:?}")),
            ),
        },
    }
}

fn message_args(
    event_unix_ms: i64,
    style: TimelineDateStyle,
    bucket: TimelineDateBucket,
    timezone: &str,
) -> mf2_i18n::Args {
    let mut args = mf2_i18n::Args::new();

    if message_uses_event(style, bucket) {
        args.insert(
            "event",
            mf2_i18n::Value::DateTime(mf2_i18n::DateTimeValue::unix_milliseconds(event_unix_ms)),
        );
    }

    if matches!(style, TimelineDateStyle::Feed)
        && let TimelineDateBucket::MinutesAgo { minutes } = bucket
    {
        args.insert("minutes", mf2_i18n::Value::Num(f64::from(minutes)));
    }

    if matches!(style, TimelineDateStyle::Audit) {
        args.insert("timezone", mf2_i18n::Value::Str(timezone.to_owned()));
    }

    args
}

fn message_uses_event(style: TimelineDateStyle, bucket: TimelineDateBucket) -> bool {
    match style {
        TimelineDateStyle::Detail | TimelineDateStyle::Audit => true,
        TimelineDateStyle::Feed => matches!(
            bucket,
            TimelineDateBucket::Today
                | TimelineDateBucket::Yesterday
                | TimelineDateBucket::Weekday
                | TimelineDateBucket::SameYear
                | TimelineDateBucket::Older
                | TimelineDateBucket::Future
        ),
    }
}

fn format_key_with_backend(
    runtime: &mf2_i18n::EmbeddedRuntime,
    locale: &str,
    key: &str,
    args: &mf2_i18n::Args,
    backend: &dyn mf2_i18n::FormatBackend,
) -> TimelineDateResult<String> {
    runtime
        .format_with_backend(locale, key, args, backend)
        .map_err(|error| TimelineDateError::I18nFormat(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_LOCALE, SUPPORTED_LOCALES, embedded_runtime, format_key_with_backend,
        format_selected, message_args, message_key,
    };
    use crate::{
        HourCycle, TimelineDateBucket, TimelineDateError, TimelineDateFormatter,
        TimelineDateOptions, TimelineDateStyle,
    };

    const MESSAGE_KEYS: [&str; 10] = [
        "timeline_date.just_now",
        "timeline_date.minutes_ago",
        "timeline_date.today_at_time",
        "timeline_date.yesterday_at_time",
        "timeline_date.weekday_at_time",
        "timeline_date.same_year_at_time",
        "timeline_date.older_date",
        "timeline_date.future_at_datetime",
        "timeline_date.detail_datetime",
        "timeline_date.audit_datetime",
    ];

    const CATALOG_SOURCES: [&str; 3] = [
        include_str!("../i18n/locales/en/timeline_date.mf2"),
        include_str!("../i18n/locales/es/timeline_date.mf2"),
        include_str!("../i18n/locales/fr/timeline_date.mf2"),
    ];

    #[test]
    fn generated_runtime_exposes_expected_locales() {
        assert_eq!(DEFAULT_LOCALE, "en");
        assert_eq!(SUPPORTED_LOCALES, ["en", "es", "fr"]);
    }

    #[test]
    fn every_key_formats_in_every_locale() {
        let runtime = embedded_runtime().expect("runtime");
        let backend = mf2_i18n::embedded::BasicFormatBackend;
        let mut args = mf2_i18n::Args::new();
        args.insert(
            "event",
            mf2_i18n::Value::DateTime(mf2_i18n::DateTimeValue::unix_milliseconds(0)),
        );
        args.insert("minutes", mf2_i18n::Value::Num(2.0));
        args.insert("timezone", mf2_i18n::Value::Str("UTC".to_owned()));

        for locale in SUPPORTED_LOCALES {
            for key in MESSAGE_KEYS {
                let value = runtime
                    .format_with_backend(locale, key, &args, &backend)
                    .expect("message should format");
                assert!(!value.is_empty());
            }
        }
    }

    #[test]
    fn message_key_maps_every_feed_bucket() {
        let cases = [
            (TimelineDateBucket::JustNow, "timeline_date.just_now"),
            (
                TimelineDateBucket::MinutesAgo { minutes: 1 },
                "timeline_date.minutes_ago",
            ),
            (TimelineDateBucket::Today, "timeline_date.today_at_time"),
            (
                TimelineDateBucket::Yesterday,
                "timeline_date.yesterday_at_time",
            ),
            (TimelineDateBucket::Weekday, "timeline_date.weekday_at_time"),
            (
                TimelineDateBucket::SameYear,
                "timeline_date.same_year_at_time",
            ),
            (TimelineDateBucket::Older, "timeline_date.older_date"),
            (
                TimelineDateBucket::Future,
                "timeline_date.future_at_datetime",
            ),
        ];

        for (bucket, key) in cases {
            assert_eq!(message_key(TimelineDateStyle::Feed, bucket), Ok(key));
        }
    }

    #[test]
    fn message_key_maps_fixed_styles_without_feed_bucket_dependence() {
        assert_eq!(
            message_key(TimelineDateStyle::Detail, TimelineDateBucket::JustNow),
            Ok("timeline_date.detail_datetime")
        );
        assert_eq!(
            message_key(
                TimelineDateStyle::Detail,
                TimelineDateBucket::MinutesAgo { minutes: 3 }
            ),
            Ok("timeline_date.detail_datetime")
        );
        assert_eq!(
            message_key(TimelineDateStyle::Audit, TimelineDateBucket::Older),
            Ok("timeline_date.audit_datetime")
        );
    }

    #[test]
    fn message_key_rejects_invalid_feed_buckets() {
        assert_eq!(
            message_key(TimelineDateStyle::Feed, TimelineDateBucket::Detail),
            Err(TimelineDateError::Internal(
                "invalid feed bucket: Detail".to_owned()
            ))
        );
        assert_eq!(
            message_key(TimelineDateStyle::Feed, TimelineDateBucket::Audit),
            Err(TimelineDateError::Internal(
                "invalid feed bucket: Audit".to_owned()
            ))
        );
    }

    #[test]
    fn message_args_include_only_needed_values() {
        let just_now = message_args(
            42,
            TimelineDateStyle::Feed,
            TimelineDateBucket::JustNow,
            "UTC",
        );
        assert!(just_now.get("event").is_none());
        assert!(just_now.get("minutes").is_none());
        assert!(just_now.get("timezone").is_none());

        let minutes = message_args(
            42,
            TimelineDateStyle::Feed,
            TimelineDateBucket::MinutesAgo { minutes: 8 },
            "UTC",
        );
        assert!(minutes.get("event").is_none());
        assert!(has_number_arg(&minutes, "minutes", 8.0));
        assert!(minutes.get("timezone").is_none());

        for bucket in [
            TimelineDateBucket::Today,
            TimelineDateBucket::Yesterday,
            TimelineDateBucket::Weekday,
            TimelineDateBucket::SameYear,
            TimelineDateBucket::Older,
            TimelineDateBucket::Future,
        ] {
            let args = message_args(42, TimelineDateStyle::Feed, bucket, "UTC");
            assert!(has_datetime_arg(&args, "event", 42));
            assert!(args.get("minutes").is_none());
            assert!(args.get("timezone").is_none());
        }

        let detail = message_args(
            42,
            TimelineDateStyle::Detail,
            TimelineDateBucket::Detail,
            "UTC",
        );
        assert!(has_datetime_arg(&detail, "event", 42));
        assert!(detail.get("minutes").is_none());
        assert!(detail.get("timezone").is_none());

        let audit = message_args(
            42,
            TimelineDateStyle::Audit,
            TimelineDateBucket::Audit,
            "America/Vancouver",
        );
        assert!(has_datetime_arg(&audit, "event", 42));
        assert!(audit.get("minutes").is_none());
        assert!(has_string_arg(&audit, "timezone", "America/Vancouver"));
    }

    #[test]
    fn missing_message_key_maps_to_format_error() {
        let runtime = embedded_runtime().expect("runtime");
        let backend = mf2_i18n::embedded::BasicFormatBackend;
        let error = format_key_with_backend(
            runtime,
            "en",
            "timeline_date.missing",
            &mf2_i18n::Args::new(),
            &backend,
        )
        .expect_err("missing key");
        assert_eq!(
            error,
            TimelineDateError::I18nFormat("invalid input: missing message".to_owned())
        );
    }

    #[test]
    fn runtime_backend_failures_map_to_format_error() {
        let runtime = embedded_runtime().expect("runtime");
        let backend = mf2_i18n::embedded::UnsupportedFormatBackend;
        let args = message_args(
            42,
            TimelineDateStyle::Feed,
            TimelineDateBucket::Today,
            "UTC",
        );
        let error = format_key_with_backend(
            runtime,
            "en",
            "timeline_date.today_at_time",
            &args,
            &backend,
        )
        .expect_err("backend failure");
        assert_eq!(
            error,
            TimelineDateError::I18nFormat(
                "unsupported: time formatting requires a format backend".to_owned()
            )
        );
    }

    #[test]
    fn exact_numeric_plural_strategy_formats_singular_and_plural_labels() {
        let cases = [
            ("en", "1 min ago", "2 min ago"),
            ("es", "hace 1 min", "hace 2 min"),
            ("fr", "il y a 1 min", "il y a 2 min"),
        ];

        for (locale, singular, plural) in cases {
            let formatter = TimelineDateFormatter::new(
                TimelineDateOptions::new(120_000, "UTC").with_locale_preferences([locale]),
            )
            .expect("formatter");
            assert_eq!(
                formatter
                    .format_millis(60_000, TimelineDateStyle::Feed)
                    .expect("singular"),
                singular
            );
            assert_eq!(
                formatter
                    .format_millis(0, TimelineDateStyle::Feed)
                    .expect("plural"),
                plural
            );
        }
    }

    #[test]
    fn bundled_catalogs_do_not_use_category_plural_cases() {
        for source in CATALOG_SOURCES {
            for forbidden in ["[zero]", "[one]", "[two]", "[few]", "[many]"] {
                assert!(!source.contains(forbidden));
            }
        }
    }

    #[test]
    fn backend_creation_failures_are_typed() {
        let error = format_selected(
            "not locale",
            "UTC",
            HourCycle::LocaleDefault,
            42,
            TimelineDateStyle::Feed,
            TimelineDateBucket::MinutesAgo { minutes: 1 },
        )
        .expect_err("backend");
        assert!(matches!(
            error,
            TimelineDateError::InvalidLocale(message)
                if message.contains("invalid locale tag")
        ));
    }

    fn has_datetime_arg(args: &mf2_i18n::Args, name: &str, expected: i64) -> bool {
        matches!(
            args.get(name),
            Some(mf2_i18n::Value::DateTime(value))
                if *value == mf2_i18n::DateTimeValue::unix_milliseconds(expected)
        )
    }

    fn has_number_arg(args: &mf2_i18n::Args, name: &str, expected: f64) -> bool {
        matches!(args.get(name), Some(mf2_i18n::Value::Num(value)) if *value == expected)
    }

    fn has_string_arg(args: &mf2_i18n::Args, name: &str, expected: &str) -> bool {
        matches!(args.get(name), Some(mf2_i18n::Value::Str(value)) if value == expected)
    }
}
