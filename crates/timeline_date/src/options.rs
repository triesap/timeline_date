#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TimelineDateStyle {
    Feed,
    Detail,
    Audit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HourCycle {
    LocaleDefault,
    H12,
    H24,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FuturePolicy {
    pub skew_seconds: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OldDateTimePolicy {
    DateOnly,
    DateTime,
}

impl Default for FuturePolicy {
    fn default() -> Self {
        Self { skew_seconds: 30 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimelineDateOptions {
    pub now_unix_ms: i64,
    pub timezone: String,
    pub locale_preferences: Vec<String>,
    pub hour_cycle: HourCycle,
    pub future_policy: FuturePolicy,
    pub old_date_time_policy: OldDateTimePolicy,
}

impl TimelineDateOptions {
    pub fn new(now_unix_ms: i64, timezone: impl Into<String>) -> Self {
        Self {
            now_unix_ms,
            timezone: timezone.into(),
            locale_preferences: vec!["en".to_owned()],
            hour_cycle: HourCycle::LocaleDefault,
            future_policy: FuturePolicy::default(),
            old_date_time_policy: OldDateTimePolicy::DateOnly,
        }
    }

    pub fn with_locale_preferences<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let values = values
            .into_iter()
            .map(Into::into)
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        if !values.is_empty() {
            self.locale_preferences = values;
        }
        self
    }

    pub fn with_hour_cycle(mut self, hour_cycle: HourCycle) -> Self {
        self.hour_cycle = hour_cycle;
        self
    }

    pub fn with_future_policy(mut self, policy: FuturePolicy) -> Self {
        self.future_policy = policy;
        self
    }

    pub fn with_old_date_time_policy(mut self, policy: OldDateTimePolicy) -> Self {
        self.old_date_time_policy = policy;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FuturePolicy, HourCycle, OldDateTimePolicy, TimelineDateOptions, TimelineDateStyle,
    };

    #[test]
    fn new_sets_default_options() {
        let options = TimelineDateOptions::new(1_780_958_400_000, "America/Vancouver");
        assert_eq!(options.now_unix_ms, 1_780_958_400_000);
        assert_eq!(options.timezone, "America/Vancouver");
        assert_eq!(options.locale_preferences, ["en"]);
        assert_eq!(options.hour_cycle, HourCycle::LocaleDefault);
        assert_eq!(options.future_policy, FuturePolicy { skew_seconds: 30 });
        assert_eq!(options.old_date_time_policy, OldDateTimePolicy::DateOnly);
    }

    #[test]
    fn locale_preferences_trim_and_drop_empty_values() {
        let options =
            TimelineDateOptions::new(0, "UTC").with_locale_preferences([" fr-CA ", "", " ", "fr"]);
        assert_eq!(options.locale_preferences, ["fr-CA", "fr"]);
    }

    #[test]
    fn empty_locale_preferences_keep_default() {
        let options = TimelineDateOptions::new(0, "UTC").with_locale_preferences(["", " "]);
        assert_eq!(options.locale_preferences, ["en"]);
    }

    #[test]
    fn hour_cycle_builder_sets_every_variant() {
        let h12 = TimelineDateOptions::new(0, "UTC").with_hour_cycle(HourCycle::H12);
        let h24 = TimelineDateOptions::new(0, "UTC").with_hour_cycle(HourCycle::H24);
        let locale = TimelineDateOptions::new(0, "UTC").with_hour_cycle(HourCycle::LocaleDefault);
        assert_eq!(h12.hour_cycle, HourCycle::H12);
        assert_eq!(h24.hour_cycle, HourCycle::H24);
        assert_eq!(locale.hour_cycle, HourCycle::LocaleDefault);
    }

    #[test]
    fn future_policy_builder_sets_policy() {
        let policy = FuturePolicy { skew_seconds: 45 };
        let options = TimelineDateOptions::new(0, "UTC").with_future_policy(policy);
        assert_eq!(options.future_policy, policy);
    }

    #[test]
    fn old_date_time_policy_builder_sets_every_variant() {
        let date_only = TimelineDateOptions::new(0, "UTC")
            .with_old_date_time_policy(OldDateTimePolicy::DateOnly);
        let date_time = TimelineDateOptions::new(0, "UTC")
            .with_old_date_time_policy(OldDateTimePolicy::DateTime);
        assert_eq!(date_only.old_date_time_policy, OldDateTimePolicy::DateOnly);
        assert_eq!(date_time.old_date_time_policy, OldDateTimePolicy::DateTime);
    }

    #[test]
    fn styles_compare_by_value() {
        assert_eq!(TimelineDateStyle::Feed, TimelineDateStyle::Feed);
        assert_ne!(TimelineDateStyle::Feed, TimelineDateStyle::Detail);
        assert_ne!(TimelineDateStyle::Detail, TimelineDateStyle::Audit);
    }
}
