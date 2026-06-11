use std::sync::Arc;

use crate::{TimelineDateOptions, TimelineDateResult};

#[derive(Clone, Debug)]
pub struct TimelineDateFormatter {
    inner: Arc<TimelineDateFormatterInner>,
}

#[derive(Debug)]
struct TimelineDateFormatterInner {
    options: TimelineDateOptions,
}

impl TimelineDateFormatter {
    pub fn new(options: TimelineDateOptions) -> TimelineDateResult<Self> {
        Ok(Self {
            inner: Arc::new(TimelineDateFormatterInner { options }),
        })
    }

    pub fn options(&self) -> &TimelineDateOptions {
        &self.inner.options
    }
}

#[cfg(test)]
mod tests {
    use super::TimelineDateFormatter;
    use crate::{HourCycle, TimelineDateOptions};

    #[test]
    fn new_stores_options() {
        let options = TimelineDateOptions::new(1_780_958_400_000, "America/Vancouver")
            .with_locale_preferences(["fr-CA", "fr"])
            .with_hour_cycle(HourCycle::H24);
        let formatter = TimelineDateFormatter::new(options.clone()).expect("formatter");
        assert_eq!(formatter.options(), &options);
    }

    #[test]
    fn cloned_formatter_keeps_options() {
        let options = TimelineDateOptions::new(1, "UTC");
        let formatter = TimelineDateFormatter::new(options.clone()).expect("formatter");
        let cloned = formatter.clone();
        assert_eq!(cloned.options(), &options);
    }
}
