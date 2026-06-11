use std::sync::Arc;

use crate::{TimelineDateOptions, TimelineDateResult, locale};

#[derive(Clone, Debug)]
pub struct TimelineDateFormatter {
    inner: Arc<TimelineDateFormatterInner>,
}

#[derive(Debug)]
struct TimelineDateFormatterInner {
    options: TimelineDateOptions,
    selected_locale: String,
}

impl TimelineDateFormatter {
    pub fn new(options: TimelineDateOptions) -> TimelineDateResult<Self> {
        let selected_locale = locale::select_locale(&options.locale_preferences)?;
        Ok(Self {
            inner: Arc::new(TimelineDateFormatterInner {
                options,
                selected_locale,
            }),
        })
    }

    pub fn options(&self) -> &TimelineDateOptions {
        &self.inner.options
    }

    pub fn selected_locale(&self) -> &str {
        &self.inner.selected_locale
    }
}

#[cfg(test)]
mod tests {
    use super::TimelineDateFormatter;
    use crate::{HourCycle, TimelineDateError, TimelineDateOptions};

    #[test]
    #[cfg(feature = "mf2")]
    fn new_stores_options() {
        let options = TimelineDateOptions::new(1_780_958_400_000, "America/Vancouver")
            .with_locale_preferences(["fr-CA", "fr"])
            .with_hour_cycle(HourCycle::H24);
        let formatter = TimelineDateFormatter::new(options.clone()).expect("formatter");
        assert_eq!(formatter.options(), &options);
        assert_eq!(formatter.selected_locale(), "fr");
    }

    #[test]
    fn cloned_formatter_keeps_options() {
        let options = TimelineDateOptions::new(1, "UTC");
        let formatter = TimelineDateFormatter::new(options.clone()).expect("formatter");
        let cloned = formatter.clone();
        assert_eq!(cloned.options(), &options);
        assert_eq!(cloned.selected_locale(), "en");
    }

    #[test]
    #[cfg(feature = "mf2")]
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
    #[cfg(feature = "mf2")]
    fn new_rejects_malformed_locale() {
        let options = TimelineDateOptions::new(1, "UTC").with_locale_preferences(["en--US"]);
        let error = TimelineDateFormatter::new(options).expect_err("invalid locale");
        assert_eq!(error, TimelineDateError::InvalidLocale("en--US".to_owned()));
    }
}
