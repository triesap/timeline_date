use crate::{HourCycle, TimelineDateError, TimelineDateResult};

pub(crate) struct TimelineDateBackend {
    locale: String,
    timezone: String,
    hour_cycle: HourCycle,
    std_backend: mf2_i18n::StdFormatBackend,
}

impl TimelineDateBackend {
    pub(crate) fn new(
        locale: &str,
        timezone: &str,
        hour_cycle: HourCycle,
    ) -> TimelineDateResult<Self> {
        let std_backend = mf2_i18n::StdFormatBackend::new(locale)
            .map_err(|error| TimelineDateError::InvalidLocale(error.to_string()))?;

        Ok(Self {
            locale: locale.to_owned(),
            timezone: timezone.to_owned(),
            hour_cycle,
            std_backend,
        })
    }

    fn format_datetime_request(
        &self,
        value: mf2_i18n::DateTimeValue,
        request: DateTimeRequest,
        options: &[mf2_i18n::FormatterOption],
    ) -> mf2_i18n::CoreResult<String> {
        let parsed = parse_datetime_options(options)?;
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
            &self.timezone,
            self.hour_cycle,
        );
        Err(mf2_i18n::CoreError::Unsupported(
            datetime_unsupported_message(),
        ))
    }
}

pub(crate) fn datetime_unsupported_message() -> &'static str {
    #[cfg(feature = "icu")]
    {
        "timezone-aware datetime formatting is not implemented"
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
    use crate::{HourCycle, TimelineDateError};

    #[test]
    fn new_stores_boundary_state() {
        let backend =
            TimelineDateBackend::new("en", "America/Vancouver", HourCycle::H24).expect("backend");
        assert_eq!(backend.locale, "en");
        assert_eq!(backend.timezone, "America/Vancouver");
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
}
