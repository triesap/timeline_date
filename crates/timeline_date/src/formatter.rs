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
}
