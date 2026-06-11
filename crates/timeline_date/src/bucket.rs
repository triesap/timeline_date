#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TimelineDateBucket {
    JustNow,
    MinutesAgo { minutes: u32 },
    Today,
    Yesterday,
    Weekday,
    SameYear,
    Older,
    Future,
}

#[cfg(test)]
mod tests {
    use super::TimelineDateBucket;

    #[test]
    fn bucket_variants_compare_by_value() {
        assert_eq!(TimelineDateBucket::JustNow, TimelineDateBucket::JustNow);
        assert_eq!(
            TimelineDateBucket::MinutesAgo { minutes: 8 },
            TimelineDateBucket::MinutesAgo { minutes: 8 }
        );
        assert_ne!(
            TimelineDateBucket::MinutesAgo { minutes: 8 },
            TimelineDateBucket::MinutesAgo { minutes: 9 }
        );
        assert_ne!(TimelineDateBucket::Today, TimelineDateBucket::Yesterday);
        assert_ne!(TimelineDateBucket::Weekday, TimelineDateBucket::SameYear);
        assert_ne!(TimelineDateBucket::Older, TimelineDateBucket::Future);
    }
}
