use crate::{TimelineDateError, TimelineDateResult};

#[cfg(feature = "jiff")]
#[derive(Clone, Debug)]
pub(crate) struct ValidatedClock {
    _timezone: jiff::tz::TimeZone,
    _now: jiff::Timestamp,
}

#[cfg(feature = "jiff")]
impl ValidatedClock {
    pub(crate) fn new(now_unix_ms: i64, timezone_id: &str) -> TimelineDateResult<Self> {
        Ok(Self {
            _timezone: timezone_from_id(timezone_id)?,
            _now: timestamp_from_millis(now_unix_ms)?,
        })
    }
}

#[cfg(feature = "jiff")]
pub(crate) fn timezone_from_id(timezone_id: &str) -> TimelineDateResult<jiff::tz::TimeZone> {
    jiff::tz::TimeZone::get(timezone_id)
        .map_err(|_| TimelineDateError::InvalidTimezone(timezone_id.to_owned()))
}

#[cfg(feature = "jiff")]
pub(crate) fn timestamp_from_millis(unix_ms: i64) -> TimelineDateResult<jiff::Timestamp> {
    jiff::Timestamp::from_millisecond(unix_ms)
        .map_err(|_| TimelineDateError::InvalidTimestamp(unix_ms))
}

#[cfg(not(feature = "jiff"))]
#[derive(Clone, Debug)]
pub(crate) struct ValidatedClock;

#[cfg(not(feature = "jiff"))]
impl ValidatedClock {
    pub(crate) fn new(_now_unix_ms: i64, _timezone_id: &str) -> TimelineDateResult<Self> {
        Err(TimelineDateError::FormattingUnsupported(
            "time validation requires the jiff feature".to_owned(),
        ))
    }
}

#[cfg(all(test, feature = "jiff"))]
mod tests {
    use super::{ValidatedClock, timestamp_from_millis, timezone_from_id};
    use crate::TimelineDateError;

    #[test]
    fn valid_timezone_is_accepted() {
        let timezone = timezone_from_id("America/Vancouver").expect("timezone");
        let _clock = ValidatedClock::new(1_780_958_400_000, "America/Vancouver").expect("clock");
        assert_eq!(
            jiff::Timestamp::from_millisecond(0)
                .expect("timestamp")
                .to_zoned(timezone)
                .year(),
            1969
        );
    }

    #[test]
    fn invalid_timezone_is_typed() {
        let error = timezone_from_id("Mars/Base").expect_err("invalid timezone");
        assert_eq!(
            error,
            TimelineDateError::InvalidTimezone("Mars/Base".to_owned())
        );
    }

    #[test]
    fn representable_timestamp_edges_are_valid() {
        let min = jiff::Timestamp::MIN.as_millisecond();
        let max = jiff::Timestamp::MAX.as_second() * 1_000;
        assert_eq!(
            timestamp_from_millis(min).expect("min").as_millisecond(),
            min
        );
        assert_eq!(
            timestamp_from_millis(max).expect("max").as_millisecond(),
            max
        );
        assert_eq!(
            timestamp_from_millis(max + 1),
            Err(TimelineDateError::InvalidTimestamp(max + 1))
        );
    }

    #[test]
    fn out_of_range_timestamps_are_typed() {
        assert_eq!(
            timestamp_from_millis(i64::MIN),
            Err(TimelineDateError::InvalidTimestamp(i64::MIN))
        );
        assert_eq!(
            timestamp_from_millis(i64::MAX),
            Err(TimelineDateError::InvalidTimestamp(i64::MAX))
        );
    }

    #[test]
    fn cloned_clock_keeps_validated_state() {
        let clock = ValidatedClock::new(0, "UTC").expect("clock");
        let cloned = clock.clone();
        assert_eq!(cloned._now.as_millisecond(), 0);
        assert_eq!(
            jiff::Timestamp::from_millisecond(0)
                .expect("timestamp")
                .to_zoned(cloned._timezone)
                .year(),
            1970
        );
    }
}
