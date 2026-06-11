use crate::{HourCycle, TimelineDateError, TimelineDateResult};

pub(crate) struct TimelineDateBackend {
    locale: String,
    hour_cycle: HourCycle,
    std_backend: mf2_i18n::StdFormatBackend,
    #[cfg(feature = "jiff")]
    timezone_data: jiff::tz::TimeZone,
}

impl TimelineDateBackend {
    pub(crate) fn new(
        locale: &str,
        timezone: &str,
        hour_cycle: HourCycle,
    ) -> TimelineDateResult<Self> {
        let std_backend = mf2_i18n::StdFormatBackend::new(locale)
            .map_err(|error| TimelineDateError::InvalidLocale(error.to_string()))?;
        #[cfg(feature = "jiff")]
        let timezone_data = crate::time::timezone_from_id(timezone)?;
        #[cfg(not(feature = "jiff"))]
        let _ = timezone;

        Ok(Self {
            locale: locale.to_owned(),
            hour_cycle,
            std_backend,
            #[cfg(feature = "jiff")]
            timezone_data,
        })
    }

    fn format_datetime_request(
        &self,
        value: mf2_i18n::DateTimeValue,
        request: DateTimeRequest,
        options: &[mf2_i18n::FormatterOption],
    ) -> mf2_i18n::CoreResult<String> {
        let parsed = parse_datetime_options(options)?;

        #[cfg(all(feature = "jiff", feature = "icu"))]
        {
            let local = self.local_datetime_parts(value)?;
            self.format_datetime_with_icu(local, request, parsed)
        }

        #[cfg(not(all(feature = "jiff", feature = "icu")))]
        {
            #[cfg(feature = "jiff")]
            let local = self.local_datetime_parts(value)?;
            let _ = (
                value,
                request,
                parsed.style,
                parsed.weekday,
                parsed.month,
                parsed.day,
                parsed.year,
                parsed.date_style,
                parsed.time_style,
                &self.locale,
                self.hour_cycle,
                #[cfg(feature = "jiff")]
                local.year,
                #[cfg(feature = "jiff")]
                local.month,
                #[cfg(feature = "jiff")]
                local.day,
                #[cfg(feature = "jiff")]
                local.hour,
                #[cfg(feature = "jiff")]
                local.minute,
                #[cfg(feature = "jiff")]
                local.second,
                #[cfg(feature = "jiff")]
                local.millisecond,
            );
            Err(mf2_i18n::CoreError::Unsupported(
                datetime_unsupported_message(),
            ))
        }
    }

    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn format_datetime_with_icu(
        &self,
        local: LocalDateTimeParts,
        request: DateTimeRequest,
        options: DateTimeOptions,
    ) -> mf2_i18n::CoreResult<String> {
        match select_icu_format(request, &options)? {
            IcuDateTimeFormat::ShortTime => self.format_icu_short_time(local),
            IcuDateTimeFormat::MediumDate => self.format_icu_medium_date(local),
            IcuDateTimeFormat::LongWeekday => self.format_icu_long_weekday(local),
            IcuDateTimeFormat::MediumMonthDay => self.format_icu_medium_month_day(local),
            IcuDateTimeFormat::LongWeekdayDate => self.format_icu_long_weekday_date(local),
            IcuDateTimeFormat::MediumDateShortTime => self.format_icu_medium_date_short_time(local),
        }
    }

    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn icu_preferences(&self) -> mf2_i18n::CoreResult<icu_datetime::DateTimeFormatterPreferences> {
        let locale = icu_locale::Locale::try_from_str(&self.locale)
            .map_err(|_| mf2_i18n::CoreError::InvalidInput("invalid ICU locale"))?;
        let mut preferences = icu_datetime::DateTimeFormatterPreferences::from(&locale);
        preferences.hour_cycle = match self.hour_cycle {
            HourCycle::LocaleDefault => preferences.hour_cycle,
            HourCycle::H12 => Some(icu_datetime::preferences::HourCycle::H12),
            HourCycle::H24 => Some(icu_datetime::preferences::HourCycle::H23),
        };
        Ok(preferences)
    }

    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn format_icu_short_time(&self, local: LocalDateTimeParts) -> mf2_i18n::CoreResult<String> {
        let formatter = icu_datetime::NoCalendarFormatter::try_new(
            self.icu_preferences()?,
            icu_datetime::fieldsets::T::hm(),
        )
        .expect("compiled ICU datetime formatter data should load");
        Ok(formatter.format(&icu_time(local)?).to_string())
    }

    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn format_icu_medium_date(&self, local: LocalDateTimeParts) -> mf2_i18n::CoreResult<String> {
        let formatter = icu_datetime::DateTimeFormatter::try_new(
            self.icu_preferences()?,
            icu_datetime::fieldsets::YMD::medium(),
        )
        .expect("compiled ICU datetime formatter data should load");
        Ok(formatter.format(&icu_date(local)?).to_string())
    }

    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn format_icu_long_weekday(&self, local: LocalDateTimeParts) -> mf2_i18n::CoreResult<String> {
        let formatter = icu_datetime::DateTimeFormatter::try_new(
            self.icu_preferences()?,
            icu_datetime::fieldsets::E::long(),
        )
        .expect("compiled ICU datetime formatter data should load");
        Ok(formatter.format(&icu_date(local)?).to_string())
    }

    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn format_icu_medium_month_day(
        &self,
        local: LocalDateTimeParts,
    ) -> mf2_i18n::CoreResult<String> {
        let formatter = icu_datetime::DateTimeFormatter::try_new(
            self.icu_preferences()?,
            icu_datetime::fieldsets::MD::medium(),
        )
        .expect("compiled ICU datetime formatter data should load");
        Ok(formatter.format(&icu_date(local)?).to_string())
    }

    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn format_icu_long_weekday_date(
        &self,
        local: LocalDateTimeParts,
    ) -> mf2_i18n::CoreResult<String> {
        let formatter = icu_datetime::DateTimeFormatter::try_new(
            self.icu_preferences()?,
            icu_datetime::fieldsets::YMDE::long(),
        )
        .expect("compiled ICU datetime formatter data should load");
        Ok(formatter.format(&icu_date(local)?).to_string())
    }

    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn format_icu_medium_date_short_time(
        &self,
        local: LocalDateTimeParts,
    ) -> mf2_i18n::CoreResult<String> {
        let formatter = icu_datetime::DateTimeFormatter::try_new(
            self.icu_preferences()?,
            icu_datetime::fieldsets::YMD::medium().with_time_hm(),
        )
        .expect("compiled ICU datetime formatter data should load");
        Ok(formatter.format(&icu_datetime(local)?).to_string())
    }

    #[cfg(all(test, feature = "jiff", feature = "icu"))]
    fn invalid_icu_locale_for_tests(&mut self) -> mf2_i18n::CoreResult<()> {
        self.locale = "not locale".to_owned();
        self.icu_preferences().map(|_| ())
    }

    #[cfg(feature = "jiff")]
    fn local_datetime_parts(
        &self,
        value: mf2_i18n::DateTimeValue,
    ) -> mf2_i18n::CoreResult<LocalDateTimeParts> {
        let timestamp = timestamp_from_datetime_value(value)?;
        let zoned = timestamp.to_zoned(self.timezone_data.clone());
        Ok(LocalDateTimeParts {
            year: zoned.year(),
            month: zoned.month(),
            day: zoned.day(),
            hour: zoned.hour(),
            minute: zoned.minute(),
            second: zoned.second(),
            millisecond: zoned.millisecond(),
        })
    }
}

#[cfg(any(not(feature = "icu"), not(feature = "jiff")))]
pub(crate) fn datetime_unsupported_message() -> &'static str {
    #[cfg(all(feature = "icu", not(feature = "jiff")))]
    {
        "datetime formatting requires the jiff feature for timezone conversion"
    }
    #[cfg(not(feature = "icu"))]
    {
        "datetime formatting requires the icu feature for localized output"
    }
}

impl mf2_i18n::FormatBackend for TimelineDateBackend {
    fn plural_category(&self, value: f64) -> mf2_i18n::CoreResult<mf2_i18n::PluralCategory> {
        self.std_backend.plural_category(value)
    }

    fn format_number(
        &self,
        value: f64,
        options: &[mf2_i18n::FormatterOption],
    ) -> mf2_i18n::CoreResult<String> {
        self.std_backend.format_number(value, options)
    }

    fn format_date(
        &self,
        value: mf2_i18n::DateTimeValue,
        options: &[mf2_i18n::FormatterOption],
    ) -> mf2_i18n::CoreResult<String> {
        self.format_datetime_request(value, DateTimeRequest::Date, options)
    }

    fn format_time(
        &self,
        value: mf2_i18n::DateTimeValue,
        options: &[mf2_i18n::FormatterOption],
    ) -> mf2_i18n::CoreResult<String> {
        self.format_datetime_request(value, DateTimeRequest::Time, options)
    }

    fn format_datetime(
        &self,
        value: mf2_i18n::DateTimeValue,
        options: &[mf2_i18n::FormatterOption],
    ) -> mf2_i18n::CoreResult<String> {
        self.format_datetime_request(value, DateTimeRequest::DateTime, options)
    }

    fn format_unit(
        &self,
        value: f64,
        unit_id: u32,
        options: &[mf2_i18n::FormatterOption],
    ) -> mf2_i18n::CoreResult<String> {
        self.std_backend.format_unit(value, unit_id, options)
    }

    fn format_currency(
        &self,
        value: f64,
        code: [u8; 3],
        options: &[mf2_i18n::FormatterOption],
    ) -> mf2_i18n::CoreResult<String> {
        self.std_backend.format_currency(value, code, options)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DateTimeRequest {
    Date,
    Time,
    DateTime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DateTimeStyle {
    Short,
    Medium,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TextWidth {
    Long,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MonthWidth {
    Short,
    Long,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NumericField {
    Numeric,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct DateTimeOptions {
    style: Option<DateTimeStyle>,
    weekday: Option<TextWidth>,
    month: Option<MonthWidth>,
    day: Option<NumericField>,
    year: Option<NumericField>,
    date_style: Option<DateTimeStyle>,
    time_style: Option<DateTimeStyle>,
}

#[cfg(feature = "jiff")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LocalDateTimeParts {
    year: i16,
    month: i8,
    day: i8,
    hour: i8,
    minute: i8,
    second: i8,
    millisecond: i16,
}

#[cfg(feature = "jiff")]
fn timestamp_from_datetime_value(
    value: mf2_i18n::DateTimeValue,
) -> mf2_i18n::CoreResult<jiff::Timestamp> {
    let timestamp = match value {
        mf2_i18n::DateTimeValue::UnixSeconds(value) => jiff::Timestamp::from_second(value),
        mf2_i18n::DateTimeValue::UnixMilliseconds(value) => {
            jiff::Timestamp::from_millisecond(value)
        }
    };
    timestamp.map_err(|_| mf2_i18n::CoreError::InvalidInput("invalid datetime value"))
}

#[cfg(all(feature = "jiff", feature = "icu"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IcuDateTimeFormat {
    ShortTime,
    MediumDate,
    LongWeekday,
    MediumMonthDay,
    LongWeekdayDate,
    MediumDateShortTime,
}

#[cfg(all(feature = "jiff", feature = "icu"))]
fn select_icu_format(
    request: DateTimeRequest,
    options: &DateTimeOptions,
) -> mf2_i18n::CoreResult<IcuDateTimeFormat> {
    match (
        request,
        options.style,
        options.weekday,
        options.month,
        options.day,
        options.year,
        options.date_style,
        options.time_style,
    ) {
        (DateTimeRequest::Time, Some(DateTimeStyle::Short), None, None, None, None, None, None) => {
            Ok(IcuDateTimeFormat::ShortTime)
        }
        (
            DateTimeRequest::Date,
            Some(DateTimeStyle::Medium),
            None,
            None,
            None,
            None,
            None,
            None,
        ) => Ok(IcuDateTimeFormat::MediumDate),
        (DateTimeRequest::DateTime, None, Some(TextWidth::Long), None, None, None, None, None) => {
            Ok(IcuDateTimeFormat::LongWeekday)
        }
        (
            DateTimeRequest::DateTime,
            None,
            None,
            Some(MonthWidth::Short),
            Some(NumericField::Numeric),
            None,
            None,
            None,
        ) => Ok(IcuDateTimeFormat::MediumMonthDay),
        (
            DateTimeRequest::DateTime,
            None,
            Some(TextWidth::Long),
            Some(MonthWidth::Long),
            Some(NumericField::Numeric),
            Some(NumericField::Numeric),
            None,
            None,
        ) => Ok(IcuDateTimeFormat::LongWeekdayDate),
        (
            DateTimeRequest::DateTime,
            None,
            None,
            None,
            None,
            None,
            Some(DateTimeStyle::Medium),
            Some(DateTimeStyle::Short),
        ) => Ok(IcuDateTimeFormat::MediumDateShortTime),
        _ => Err(mf2_i18n::CoreError::Unsupported(
            "datetime formatter option combination not supported",
        )),
    }
}

#[cfg(all(feature = "jiff", feature = "icu"))]
fn icu_date(
    local: LocalDateTimeParts,
) -> mf2_i18n::CoreResult<icu_datetime::input::Date<icu_calendar::Iso>> {
    icu_datetime::input::Date::try_new_iso(
        i32::from(local.year),
        local.month as u8,
        local.day as u8,
    )
    .map_err(|_| mf2_i18n::CoreError::InvalidInput("invalid local date"))
}

#[cfg(all(feature = "jiff", feature = "icu"))]
fn icu_time(local: LocalDateTimeParts) -> mf2_i18n::CoreResult<icu_datetime::input::Time> {
    let nanosecond = u32::try_from(local.millisecond)
        .ok()
        .and_then(|millisecond| millisecond.checked_mul(1_000_000))
        .ok_or(mf2_i18n::CoreError::InvalidInput("invalid local time"))?;
    icu_datetime::input::Time::try_new(
        local.hour as u8,
        local.minute as u8,
        local.second as u8,
        nanosecond,
    )
    .map_err(|_| mf2_i18n::CoreError::InvalidInput("invalid local time"))
}

#[cfg(all(feature = "jiff", feature = "icu"))]
fn icu_datetime(
    local: LocalDateTimeParts,
) -> mf2_i18n::CoreResult<icu_datetime::input::DateTime<icu_calendar::Iso>> {
    Ok(icu_datetime::input::DateTime {
        date: icu_date(local)?,
        time: icu_time(local)?,
    })
}

fn parse_datetime_options(
    options: &[mf2_i18n::FormatterOption],
) -> mf2_i18n::CoreResult<DateTimeOptions> {
    let mut parsed = DateTimeOptions::default();

    for option in options {
        let value = string_option(option)?;
        match option.key.as_str() {
            "style" => parsed.style = Some(parse_one_of(value, DATE_TIME_STYLES)?),
            "weekday" => parsed.weekday = Some(parse_one_of(value, TEXT_WIDTHS)?),
            "month" => parsed.month = Some(parse_one_of(value, MONTH_WIDTHS)?),
            "day" => parsed.day = Some(parse_one_of(value, NUMERIC_FIELDS)?),
            "year" => parsed.year = Some(parse_one_of(value, NUMERIC_FIELDS)?),
            "dateStyle" => parsed.date_style = Some(parse_one_of(value, DATE_TIME_STYLES)?),
            "timeStyle" => parsed.time_style = Some(parse_one_of(value, DATE_TIME_STYLES)?),
            _ => {
                return Err(mf2_i18n::CoreError::Unsupported(
                    "datetime formatter option not supported",
                ));
            }
        }
    }

    Ok(parsed)
}

fn string_option(option: &mf2_i18n::FormatterOption) -> mf2_i18n::CoreResult<&str> {
    match &option.value {
        mf2_i18n::FormatterOptionValue::Str(value) => Ok(value.as_str()),
        _ => Err(mf2_i18n::CoreError::InvalidInput(
            "datetime formatter option must be a string",
        )),
    }
}

fn parse_one_of<T: Copy>(value: &str, supported: &[(&'static str, T)]) -> mf2_i18n::CoreResult<T> {
    for (candidate, parsed) in supported {
        if value == *candidate {
            return Ok(*parsed);
        }
    }

    Err(mf2_i18n::CoreError::Unsupported(
        "datetime formatter option value not supported",
    ))
}

const DATE_TIME_STYLES: &[(&str, DateTimeStyle)] = &[
    ("short", DateTimeStyle::Short),
    ("medium", DateTimeStyle::Medium),
];
const TEXT_WIDTHS: &[(&str, TextWidth)] = &[("long", TextWidth::Long)];
const MONTH_WIDTHS: &[(&str, MonthWidth)] =
    &[("short", MonthWidth::Short), ("long", MonthWidth::Long)];
const NUMERIC_FIELDS: &[(&str, NumericField)] = &[("numeric", NumericField::Numeric)];

#[cfg(test)]
mod tests {
    use mf2_i18n::{FormatBackend, FormatterOption, FormatterOptionValue};

    use super::{
        DateTimeOptions, DateTimeStyle, MonthWidth, NumericField, TEXT_WIDTHS, TextWidth,
        TimelineDateBackend, parse_datetime_options, parse_one_of,
    };
    #[cfg(all(feature = "jiff", feature = "icu"))]
    use super::{
        DateTimeRequest, IcuDateTimeFormat, icu_date, icu_datetime, icu_time, select_icu_format,
    };
    #[cfg(feature = "jiff")]
    use super::{LocalDateTimeParts, timestamp_from_datetime_value};
    use crate::{HourCycle, TimelineDateError};

    #[test]
    fn new_stores_boundary_state() {
        let backend =
            TimelineDateBackend::new("en", "America/Vancouver", HourCycle::H24).expect("backend");
        assert_eq!(backend.locale, "en");
        assert_eq!(backend.hour_cycle, HourCycle::H24);
    }

    #[test]
    fn new_maps_std_backend_locale_errors() {
        assert!(matches!(
            TimelineDateBackend::new("not locale", "UTC", HourCycle::LocaleDefault),
            Err(TimelineDateError::InvalidLocale(message))
                if message.contains("invalid locale tag")
        ));
    }

    #[test]
    #[cfg(feature = "jiff")]
    fn new_rejects_invalid_timezone() {
        assert!(matches!(
            TimelineDateBackend::new("en", "Mars/Base", HourCycle::LocaleDefault),
            Err(TimelineDateError::InvalidTimezone(timezone)) if timezone == "Mars/Base"
        ));
    }

    #[test]
    fn delegates_plural_and_number_formatting() {
        let backend =
            TimelineDateBackend::new("en", "UTC", HourCycle::LocaleDefault).expect("backend");
        assert_eq!(
            backend.plural_category(1.0).expect("plural"),
            mf2_i18n::PluralCategory::One
        );
        assert_eq!(
            backend.format_number(1234.5, &[]).expect("number"),
            "1,234.5"
        );
    }

    #[test]
    #[cfg(feature = "jiff")]
    fn local_datetime_parts_convert_unix_milliseconds_in_timezone() {
        let backend = TimelineDateBackend::new("en", "America/Vancouver", HourCycle::LocaleDefault)
            .expect("backend");
        assert_eq!(
            backend
                .local_datetime_parts(millis_value("2026-06-08T07:30:15.250Z"))
                .expect("parts"),
            LocalDateTimeParts {
                year: 2026,
                month: 6,
                day: 8,
                hour: 0,
                minute: 30,
                second: 15,
                millisecond: 250,
            }
        );
    }

    #[test]
    #[cfg(feature = "jiff")]
    fn local_datetime_parts_accept_unix_seconds() {
        let backend = TimelineDateBackend::new("en", "Asia/Tokyo", HourCycle::LocaleDefault)
            .expect("backend");
        assert_eq!(
            backend
                .local_datetime_parts(mf2_i18n::DateTimeValue::unix_seconds(0))
                .expect("parts"),
            LocalDateTimeParts {
                year: 1970,
                month: 1,
                day: 1,
                hour: 9,
                minute: 0,
                second: 0,
                millisecond: 0,
            }
        );
    }

    #[test]
    #[cfg(feature = "jiff")]
    fn local_datetime_parts_preserve_local_date_when_utc_differs() {
        let backend = TimelineDateBackend::new("en", "America/Vancouver", HourCycle::LocaleDefault)
            .expect("backend");
        let parts = backend
            .local_datetime_parts(millis_value("2026-06-08T01:30:00Z"))
            .expect("parts");
        assert_eq!(
            (parts.year, parts.month, parts.day, parts.hour, parts.minute),
            (2026, 6, 7, 18, 30)
        );
    }

    #[test]
    #[cfg(feature = "jiff")]
    fn local_datetime_parts_cover_vancouver_dst_edges() {
        let backend = TimelineDateBackend::new("en", "America/Vancouver", HourCycle::LocaleDefault)
            .expect("backend");
        let spring = backend
            .local_datetime_parts(millis_value("2026-03-08T10:30:00Z"))
            .expect("spring");
        let fall = backend
            .local_datetime_parts(millis_value("2026-11-01T09:30:00Z"))
            .expect("fall");
        assert_eq!(
            (
                spring.year,
                spring.month,
                spring.day,
                spring.hour,
                spring.minute
            ),
            (2026, 3, 8, 3, 30)
        );
        assert_eq!(
            (fall.year, fall.month, fall.day, fall.hour, fall.minute),
            (2026, 11, 1, 2, 30)
        );
    }

    #[test]
    #[cfg(feature = "jiff")]
    fn invalid_datetime_values_are_typed() {
        assert_eq!(
            timestamp_from_datetime_value(mf2_i18n::DateTimeValue::unix_milliseconds(i64::MAX)),
            Err(mf2_i18n::CoreError::InvalidInput("invalid datetime value"))
        );
    }

    #[test]
    #[cfg(feature = "jiff")]
    fn format_datetime_request_propagates_invalid_timestamp() {
        let backend = TimelineDateBackend::new("en", "America/Vancouver", HourCycle::LocaleDefault)
            .expect("backend");
        assert_eq!(
            backend
                .format_time(
                    mf2_i18n::DateTimeValue::unix_milliseconds(i64::MAX),
                    &[string_option("style", "short")],
                )
                .expect_err("invalid"),
            mf2_i18n::CoreError::InvalidInput("invalid datetime value")
        );
    }

    #[test]
    fn delegates_unit_and_currency_formatting() {
        let backend =
            TimelineDateBackend::new("en", "UTC", HourCycle::LocaleDefault).expect("backend");
        assert_eq!(
            backend
                .format_currency(3.5, *b"USD", &[])
                .expect("currency"),
            "USD 3.5"
        );
        assert_eq!(
            backend.format_unit(3.5, 7, &[]).expect_err("unit"),
            mf2_i18n::CoreError::Unsupported("unit formatting requires unit label data")
        );
    }

    #[test]
    fn parses_supported_datetime_options() {
        let parsed = parse_datetime_options(&[
            string_option("style", "short"),
            string_option("weekday", "long"),
            string_option("month", "long"),
            string_option("day", "numeric"),
            string_option("year", "numeric"),
            string_option("dateStyle", "medium"),
            string_option("timeStyle", "short"),
        ])
        .expect("options");
        assert_eq!(
            parsed,
            DateTimeOptions {
                style: Some(DateTimeStyle::Short),
                weekday: Some(TextWidth::Long),
                month: Some(MonthWidth::Long),
                day: Some(NumericField::Numeric),
                year: Some(NumericField::Numeric),
                date_style: Some(DateTimeStyle::Medium),
                time_style: Some(DateTimeStyle::Short),
            }
        );

        assert_eq!(
            parse_datetime_options(&[string_option("month", "short")]).expect("month"),
            DateTimeOptions {
                month: Some(MonthWidth::Short),
                ..DateTimeOptions::default()
            }
        );
    }

    #[test]
    fn rejects_unsupported_datetime_option_key() {
        let error = parse_datetime_options(&[string_option("era", "short")]).expect_err("key");
        assert_eq!(
            error,
            mf2_i18n::CoreError::Unsupported("datetime formatter option not supported")
        );
    }

    #[test]
    fn rejects_non_string_datetime_option_value() {
        let error = parse_datetime_options(&[number_option("style", 1.0)]).expect_err("value");
        assert_eq!(
            error,
            mf2_i18n::CoreError::InvalidInput("datetime formatter option must be a string")
        );
    }

    #[test]
    fn rejects_unsupported_datetime_option_value() {
        let error = parse_datetime_options(&[string_option("day", "2-digit")]).expect_err("value");
        assert_eq!(
            error,
            mf2_i18n::CoreError::Unsupported("datetime formatter option value not supported")
        );
    }

    #[test]
    #[cfg(not(all(feature = "jiff", feature = "icu")))]
    fn date_time_and_datetime_return_explicit_unsupported() {
        let backend =
            TimelineDateBackend::new("en", "America/Vancouver", HourCycle::H12).expect("backend");
        let value = mf2_i18n::DateTimeValue::unix_milliseconds(0);
        let expected = mf2_i18n::CoreError::Unsupported(super::datetime_unsupported_message());

        assert_eq!(
            backend
                .format_date(value, &[string_option("style", "medium")])
                .expect_err("date"),
            expected
        );
        assert_eq!(
            backend
                .format_time(value, &[string_option("style", "short")])
                .expect_err("time"),
            expected
        );
        assert_eq!(
            backend
                .format_datetime(
                    value,
                    &[
                        string_option("dateStyle", "medium"),
                        string_option("timeStyle", "short"),
                    ],
                )
                .expect_err("datetime"),
            expected
        );
    }

    #[test]
    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn select_icu_format_maps_catalog_option_shapes() {
        let cases = [
            (
                DateTimeRequest::Time,
                DateTimeOptions {
                    style: Some(DateTimeStyle::Short),
                    ..DateTimeOptions::default()
                },
                IcuDateTimeFormat::ShortTime,
            ),
            (
                DateTimeRequest::Date,
                DateTimeOptions {
                    style: Some(DateTimeStyle::Medium),
                    ..DateTimeOptions::default()
                },
                IcuDateTimeFormat::MediumDate,
            ),
            (
                DateTimeRequest::DateTime,
                DateTimeOptions {
                    weekday: Some(TextWidth::Long),
                    ..DateTimeOptions::default()
                },
                IcuDateTimeFormat::LongWeekday,
            ),
            (
                DateTimeRequest::DateTime,
                DateTimeOptions {
                    month: Some(MonthWidth::Short),
                    day: Some(NumericField::Numeric),
                    ..DateTimeOptions::default()
                },
                IcuDateTimeFormat::MediumMonthDay,
            ),
            (
                DateTimeRequest::DateTime,
                DateTimeOptions {
                    weekday: Some(TextWidth::Long),
                    month: Some(MonthWidth::Long),
                    day: Some(NumericField::Numeric),
                    year: Some(NumericField::Numeric),
                    ..DateTimeOptions::default()
                },
                IcuDateTimeFormat::LongWeekdayDate,
            ),
            (
                DateTimeRequest::DateTime,
                DateTimeOptions {
                    date_style: Some(DateTimeStyle::Medium),
                    time_style: Some(DateTimeStyle::Short),
                    ..DateTimeOptions::default()
                },
                IcuDateTimeFormat::MediumDateShortTime,
            ),
        ];

        for (request, options, expected) in cases {
            assert_eq!(select_icu_format(request, &options), Ok(expected));
        }
    }

    #[test]
    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn select_icu_format_rejects_off_catalog_option_shapes() {
        let cases = [
            (
                DateTimeRequest::Time,
                DateTimeOptions {
                    style: Some(DateTimeStyle::Medium),
                    ..DateTimeOptions::default()
                },
            ),
            (
                DateTimeRequest::Date,
                DateTimeOptions {
                    style: Some(DateTimeStyle::Short),
                    ..DateTimeOptions::default()
                },
            ),
            (
                DateTimeRequest::DateTime,
                DateTimeOptions {
                    date_style: Some(DateTimeStyle::Short),
                    time_style: Some(DateTimeStyle::Short),
                    ..DateTimeOptions::default()
                },
            ),
        ];

        for (request, options) in cases {
            assert_eq!(
                select_icu_format(request, &options),
                Err(mf2_i18n::CoreError::Unsupported(
                    "datetime formatter option combination not supported"
                ))
            );
        }
    }

    #[test]
    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn icu_conversions_reject_invalid_synthetic_parts() {
        let valid = local_parts();
        assert!(icu_date(valid).is_ok());
        assert!(icu_time(valid).is_ok());
        assert!(icu_datetime(valid).is_ok());

        assert_eq!(
            icu_date(LocalDateTimeParts { month: 13, ..valid }),
            Err(mf2_i18n::CoreError::InvalidInput("invalid local date"))
        );
        assert_eq!(
            icu_time(LocalDateTimeParts { hour: 24, ..valid }),
            Err(mf2_i18n::CoreError::InvalidInput("invalid local time"))
        );
        assert_eq!(
            icu_time(LocalDateTimeParts {
                millisecond: -1,
                ..valid
            }),
            Err(mf2_i18n::CoreError::InvalidInput("invalid local time"))
        );
    }

    #[test]
    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn formats_supported_icu_datetime_shapes() {
        let backend =
            TimelineDateBackend::new("en", "America/Vancouver", HourCycle::H12).expect("backend");
        let value = millis_value("2026-06-08T07:30:15.250Z");

        let time = backend
            .format_time(value, &[string_option("style", "short")])
            .expect("time");
        assert!(time.contains("12:30"));
        assert!(time.contains("AM"));

        let date = backend
            .format_date(value, &[string_option("style", "medium")])
            .expect("date");
        assert_has_parts(&date, &["Jun", "8", "2026"]);

        let weekday = backend
            .format_datetime(value, &[string_option("weekday", "long")])
            .expect("weekday");
        assert_eq!(weekday, "Monday");

        let month_day = backend
            .format_datetime(
                value,
                &[
                    string_option("month", "short"),
                    string_option("day", "numeric"),
                ],
            )
            .expect("month day");
        assert_has_parts(&month_day, &["Jun", "8"]);

        let detail_date = backend
            .format_datetime(
                value,
                &[
                    string_option("weekday", "long"),
                    string_option("month", "long"),
                    string_option("day", "numeric"),
                    string_option("year", "numeric"),
                ],
            )
            .expect("detail");
        assert_has_parts(&detail_date, &["Monday", "June", "8", "2026"]);

        let date_time = backend
            .format_datetime(
                value,
                &[
                    string_option("dateStyle", "medium"),
                    string_option("timeStyle", "short"),
                ],
            )
            .expect("datetime");
        assert_has_parts(&date_time, &["Jun", "8", "2026", "12:30", "AM"]);
    }

    #[test]
    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn icu_time_formatting_honors_locale_default_hour_cycles() {
        let value = millis_value("2026-06-08T07:30:00Z");
        let english = format_short_time("en", HourCycle::LocaleDefault, value);
        let french = format_short_time("fr", HourCycle::LocaleDefault, value);
        let spanish = format_short_time("es", HourCycle::LocaleDefault, value);

        assert!(english.contains("12:30"));
        assert!(english.contains("AM"));
        assert!(french.contains("00:30"));
        assert_lacks_day_period(&french);
        assert!(spanish.starts_with('0'));
        assert!(spanish.contains(":30"));
        assert_lacks_day_period(&spanish);
    }

    #[test]
    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn icu_time_formatting_honors_explicit_hour_cycles() {
        let value = millis_value("2026-06-08T07:30:00Z");

        for locale in ["en", "fr", "es"] {
            let h12 = format_short_time(locale, HourCycle::H12, value);
            let h24 = format_short_time(locale, HourCycle::H24, value);

            assert!(h12.contains("12:30"));
            assert!(h24.contains("00:30"));
            assert_lacks_day_period(&h24);
        }
    }

    #[test]
    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn icu_backend_rejects_invalid_internal_locale() {
        let mut backend =
            TimelineDateBackend::new("en", "UTC", HourCycle::LocaleDefault).expect("backend");
        assert_eq!(
            backend.invalid_icu_locale_for_tests(),
            Err(mf2_i18n::CoreError::InvalidInput("invalid ICU locale"))
        );
    }

    #[test]
    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn backend_surfaces_unsupported_datetime_option_combinations() {
        let backend =
            TimelineDateBackend::new("en", "UTC", HourCycle::LocaleDefault).expect("backend");
        let value = mf2_i18n::DateTimeValue::unix_milliseconds(0);

        for (request, options) in [
            (DateTimeRequest::Date, vec![string_option("style", "short")]),
            (
                DateTimeRequest::Time,
                vec![string_option("style", "medium")],
            ),
            (
                DateTimeRequest::DateTime,
                vec![
                    string_option("dateStyle", "short"),
                    string_option("timeStyle", "short"),
                ],
            ),
        ] {
            let error = match request {
                DateTimeRequest::Date => backend.format_date(value, &options),
                DateTimeRequest::Time => backend.format_time(value, &options),
                DateTimeRequest::DateTime => backend.format_datetime(value, &options),
            }
            .expect_err("unsupported combination");
            assert_eq!(
                error,
                mf2_i18n::CoreError::Unsupported(
                    "datetime formatter option combination not supported"
                )
            );
        }
    }

    #[test]
    fn unsupported_datetime_options_surface_through_backend() {
        let backend =
            TimelineDateBackend::new("en", "UTC", HourCycle::LocaleDefault).expect("backend");
        let error = backend
            .format_time(
                mf2_i18n::DateTimeValue::unix_milliseconds(0),
                &[string_option("era", "short")],
            )
            .expect_err("option");
        assert_eq!(
            error,
            mf2_i18n::CoreError::Unsupported("datetime formatter option not supported")
        );
    }

    #[test]
    #[cfg(not(feature = "icu"))]
    fn reduced_mode_datetime_error_names_icu_feature() {
        let backend =
            TimelineDateBackend::new("en", "UTC", HourCycle::LocaleDefault).expect("backend");
        let error = backend
            .format_time(
                mf2_i18n::DateTimeValue::unix_milliseconds(0),
                &[string_option("style", "short")],
            )
            .expect_err("time");
        assert_eq!(
            error,
            mf2_i18n::CoreError::Unsupported(
                "datetime formatting requires the icu feature for localized output"
            )
        );
    }

    #[test]
    fn parse_one_of_accepts_supported_values() {
        assert_eq!(
            parse_one_of("long", TEXT_WIDTHS).expect("width"),
            TextWidth::Long
        );
    }

    fn string_option(key: &str, value: &str) -> FormatterOption {
        FormatterOption {
            key: key.to_owned(),
            value: FormatterOptionValue::Str(value.to_owned()),
        }
    }

    fn number_option(key: &str, value: f64) -> FormatterOption {
        FormatterOption {
            key: key.to_owned(),
            value: FormatterOptionValue::Num(value),
        }
    }

    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn assert_has_parts(value: &str, parts: &[&str]) {
        for part in parts {
            assert!(value.contains(part), "{value:?} should contain {part:?}");
        }
    }

    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn assert_lacks_day_period(value: &str) {
        for part in ["AM", "PM", "a. m.", "p. m."] {
            assert!(
                !value.contains(part),
                "{value:?} should not contain {part:?}"
            );
        }
    }

    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn format_short_time(
        locale: &str,
        hour_cycle: HourCycle,
        value: mf2_i18n::DateTimeValue,
    ) -> String {
        TimelineDateBackend::new(locale, "America/Vancouver", hour_cycle)
            .expect("backend")
            .format_time(value, &[string_option("style", "short")])
            .expect("time")
    }

    #[cfg(all(feature = "jiff", feature = "icu"))]
    fn local_parts() -> LocalDateTimeParts {
        LocalDateTimeParts {
            year: 2026,
            month: 6,
            day: 8,
            hour: 0,
            minute: 30,
            second: 15,
            millisecond: 250,
        }
    }

    #[cfg(feature = "jiff")]
    fn millis_value(value: &str) -> mf2_i18n::DateTimeValue {
        mf2_i18n::DateTimeValue::unix_milliseconds(
            value
                .parse::<jiff::Timestamp>()
                .expect("timestamp")
                .as_millisecond(),
        )
    }
}
