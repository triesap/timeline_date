include!(concat!(env!("OUT_DIR"), "/timeline_date_i18n_runtime.rs"));

use crate::{TimelineDateError, TimelineDateResult};

pub(crate) fn embedded_runtime() -> TimelineDateResult<&'static mf2_i18n::EmbeddedRuntime> {
    runtime().map_err(TimelineDateError::I18nInit)
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_LOCALE, SUPPORTED_LOCALES, embedded_runtime};

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
}
