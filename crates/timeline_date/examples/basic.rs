use timeline_date::{HourCycle, TimelineDateFormatter, TimelineDateOptions, TimelineDateStyle};

fn main() -> Result<(), timeline_date::TimelineDateError> {
    let formatter = TimelineDateFormatter::new(
        TimelineDateOptions::new(1_780_945_200_000, "America/Vancouver")
            .with_locale_preferences(["en-CA", "en"])
            .with_hour_cycle(HourCycle::H24),
    )?;

    let feed = formatter.format_millis(1_780_944_720_000, TimelineDateStyle::Feed)?;
    let audit = formatter.format_millis(1_780_945_200_250, TimelineDateStyle::Audit)?;

    assert_eq!(feed, "8 min ago");
    assert_eq!(audit, "2026-06-08T12:00:00.250 America/Vancouver");

    println!("{feed}");

    Ok(())
}
