use crate::{FuturePolicy, TimelineDateBucket, TimelineDateResult, time};

#[cfg(feature = "jiff")]
pub(crate) fn classify_feed_millis(
    event_unix_ms: i64,
    clock: &time::ValidatedClock,
    future_policy: FuturePolicy,
) -> TimelineDateResult<TimelineDateBucket> {
    let event = time::timestamp_from_millis(event_unix_ms)?;
    Ok(classify_feed_timestamps(
        event,
        clock.now(),
        clock.timezone(),
        future_policy,
    ))
}

#[cfg(feature = "jiff")]
pub(crate) fn classify_fixed_millis(
    event_unix_ms: i64,
    bucket: TimelineDateBucket,
) -> TimelineDateResult<TimelineDateBucket> {
    time::timestamp_from_millis(event_unix_ms)?;
    Ok(bucket)
}

#[cfg(feature = "jiff")]
fn classify_feed_timestamps(
    event: jiff::Timestamp,
    now: jiff::Timestamp,
    timezone: jiff::tz::TimeZone,
    future_policy: FuturePolicy,
) -> TimelineDateBucket {
    let millis_delta = i128::from(now.as_millisecond()) - i128::from(event.as_millisecond());

    if millis_delta < 0 {
        let skew_millis = i128::from(future_policy.skew_seconds.max(0)) * 1_000;
        return if -millis_delta <= skew_millis {
            TimelineDateBucket::JustNow
        } else {
            TimelineDateBucket::Future
        };
    }

    if millis_delta < 60_000 {
        return TimelineDateBucket::JustNow;
    }

    if millis_delta < 3_600_000 {
        return TimelineDateBucket::MinutesAgo {
            minutes: (millis_delta / 60_000) as u32,
        };
    }

    let event = event.to_zoned(timezone.clone());
    let now = now.to_zoned(timezone);
    let event_date = event.date();
    let now_date = now.date();

    if event_date == now_date {
        return TimelineDateBucket::Today;
    }

    let day_delta = event_date.duration_until(now_date).as_secs() / 86_400;

    if day_delta == 1 {
        return TimelineDateBucket::Yesterday;
    }

    if (2..=6).contains(&day_delta) {
        return TimelineDateBucket::Weekday;
    }

    if event.year() == now.year() {
        return TimelineDateBucket::SameYear;
    }

    TimelineDateBucket::Older
}

#[cfg(not(feature = "jiff"))]
pub(crate) fn classify_feed_millis(
    _event_unix_ms: i64,
    _clock: &time::ValidatedClock,
    _future_policy: FuturePolicy,
) -> TimelineDateResult<TimelineDateBucket> {
    Err(crate::TimelineDateError::FormattingUnsupported(
        "feed classification requires the jiff feature".to_owned(),
    ))
}

#[cfg(not(feature = "jiff"))]
pub(crate) fn classify_fixed_millis(
    _event_unix_ms: i64,
    _bucket: TimelineDateBucket,
) -> TimelineDateResult<TimelineDateBucket> {
    Err(crate::TimelineDateError::FormattingUnsupported(
        "classification requires the jiff feature".to_owned(),
    ))
}

#[cfg(all(test, feature = "jiff"))]
mod tests {
    use super::classify_feed_timestamps;
    use crate::{
        FuturePolicy, TimelineDateBucket, TimelineDateError, TimelineDateFormatter,
        TimelineDateOptions, TimelineDateStyle,
    };

    fn ms(value: &str) -> i64 {
        value
            .parse::<jiff::Timestamp>()
            .expect("timestamp")
            .as_millisecond()
    }

    fn formatter(now: i64, timezone: &str) -> TimelineDateFormatter {
        TimelineDateFormatter::new(TimelineDateOptions::new(now, timezone)).expect("formatter")
    }

    #[test]
    fn classifies_feed_matrix() {
        let now = ms("2026-06-08T19:00:00Z");
        let formatter = formatter(now, "America/Vancouver");
        let cases = [
            (now - 10_000, TimelineDateBucket::JustNow),
            (
                now - 8 * 60_000,
                TimelineDateBucket::MinutesAgo { minutes: 8 },
            ),
            (ms("2026-06-08T17:00:00Z"), TimelineDateBucket::Today),
            (ms("2026-06-08T06:30:00Z"), TimelineDateBucket::Yesterday),
            (ms("2026-06-02T19:00:00Z"), TimelineDateBucket::Weekday),
            (ms("2026-06-01T19:00:00Z"), TimelineDateBucket::SameYear),
            (ms("2025-12-31T20:00:00Z"), TimelineDateBucket::Older),
            (now + 15_000, TimelineDateBucket::JustNow),
            (now + 2 * 60 * 60_000, TimelineDateBucket::Future),
        ];

        for (event, expected) in cases {
            assert_eq!(
                formatter
                    .classify_millis(event, TimelineDateStyle::Feed)
                    .expect("bucket"),
                expected
            );
        }
    }

    #[test]
    fn same_vancouver_local_date_wins_when_utc_date_differs() {
        let formatter = formatter(ms("2026-06-08T01:00:00Z"), "America/Vancouver");
        assert_eq!(
            formatter
                .classify_millis(ms("2026-06-07T23:30:00Z"), TimelineDateStyle::Feed)
                .expect("bucket"),
            TimelineDateBucket::Today
        );
    }

    #[test]
    fn previous_tokyo_local_date_wins_when_utc_date_matches() {
        let formatter = formatter(ms("2026-06-08T16:30:00Z"), "Asia/Tokyo");
        assert_eq!(
            formatter
                .classify_millis(ms("2026-06-08T14:30:00Z"), TimelineDateStyle::Feed)
                .expect("bucket"),
            TimelineDateBucket::Yesterday
        );
    }

    #[test]
    fn local_midnight_boundary_uses_civil_dates() {
        let formatter = formatter(ms("2026-06-08T08:30:00Z"), "America/Vancouver");
        assert_eq!(
            formatter
                .classify_millis(ms("2026-06-08T06:55:00Z"), TimelineDateStyle::Feed)
                .expect("bucket"),
            TimelineDateBucket::Yesterday
        );
    }

    #[test]
    fn vancouver_spring_forward_uses_local_civil_dates() {
        let formatter = formatter(ms("2026-03-09T08:30:00Z"), "America/Vancouver");
        assert_eq!(
            formatter
                .classify_millis(ms("2026-03-08T09:45:00Z"), TimelineDateStyle::Feed)
                .expect("bucket"),
            TimelineDateBucket::Yesterday
        );
    }

    #[test]
    fn vancouver_fall_back_uses_local_civil_dates() {
        let formatter = formatter(ms("2026-11-02T08:30:00Z"), "America/Vancouver");
        assert_eq!(
            formatter
                .classify_millis(ms("2026-11-01T08:30:00Z"), TimelineDateStyle::Feed)
                .expect("bucket"),
            TimelineDateBucket::Yesterday
        );
    }

    #[test]
    fn future_policy_controls_skew() {
        let now = ms("2026-06-08T19:00:00Z");
        let timezone = jiff::tz::TimeZone::get("America/Vancouver").expect("timezone");
        assert_eq!(
            classify_feed_timestamps(
                jiff::Timestamp::from_millisecond(now + 5_000).expect("event"),
                jiff::Timestamp::from_millisecond(now).expect("now"),
                timezone.clone(),
                FuturePolicy { skew_seconds: 0 }
            ),
            TimelineDateBucket::Future
        );
        assert_eq!(
            classify_feed_timestamps(
                jiff::Timestamp::from_millisecond(now + 5_000).expect("event"),
                jiff::Timestamp::from_millisecond(now).expect("now"),
                timezone,
                FuturePolicy { skew_seconds: 5 }
            ),
            TimelineDateBucket::JustNow
        );
    }

    #[test]
    fn invalid_event_timestamp_is_typed() {
        let formatter = formatter(0, "UTC");
        let error = formatter
            .classify_millis(i64::MAX, TimelineDateStyle::Feed)
            .expect_err("invalid event");
        assert_eq!(error, TimelineDateError::InvalidTimestamp(i64::MAX));
    }

    #[test]
    fn detail_and_audit_have_explicit_buckets() {
        let formatter = formatter(0, "UTC");
        assert_eq!(
            formatter
                .classify_millis(0, TimelineDateStyle::Detail)
                .expect("detail bucket"),
            TimelineDateBucket::Detail
        );
        assert_eq!(
            formatter
                .classify_millis(0, TimelineDateStyle::Audit)
                .expect("audit bucket"),
            TimelineDateBucket::Audit
        );
    }

    #[test]
    fn detail_and_audit_validate_event_timestamp() {
        let formatter = formatter(0, "UTC");
        assert_eq!(
            formatter.classify_millis(i64::MAX, TimelineDateStyle::Detail),
            Err(TimelineDateError::InvalidTimestamp(i64::MAX))
        );
        assert_eq!(
            formatter.classify_millis(i64::MAX, TimelineDateStyle::Audit),
            Err(TimelineDateError::InvalidTimestamp(i64::MAX))
        );
    }
}
