use proptest::prelude::*;
use tcalc_core::run;
use time::{Date, Duration, Month, Weekday};

fn date(year: i32, month: u8, day: u8) -> Date {
    Date::from_calendar_date(year, Month::try_from(month).unwrap(), day).unwrap()
}

fn is_working_day(date: Date) -> bool {
    !matches!(date.weekday(), Weekday::Saturday | Weekday::Sunday)
}

fn add_working_days(mut date: Date, days: i64) -> Date {
    let step = if days >= 0 { 1 } else { -1 };
    let mut remaining = days.abs();

    while remaining > 0 {
        date += Duration::days(step);

        if is_working_day(date) {
            remaining -= 1;
        }
    }

    date
}

proptest! {
    #[test]
    fn invalid_months_do_not_parse(
        year in 1900i64..=2100,
        month in 13i64..=1_000,
        day in 1i64..=28,
    ) {
        let input = format!("{year}/{month}/{day}");
        prop_assert!(run(&input, None).is_err());
    }

    #[test]
    fn invalid_times_do_not_parse(
        hour in 24i64..=1_000,
        minute in 0i64..=59,
    ) {
        let input = format!("{hour}:{minute}");
        prop_assert!(run(&input, None).is_err());
    }

    #[test]
    fn trailing_ident_after_valid_date_does_not_parse(
        year in 1900i64..=2100,
        month in 1i64..=12,
        day in 1i64..=28,
        suffix in "[a-zA-Z]{1,12}",
    ) {
        let input = format!("{year}/{month}/{day}{suffix}");
        prop_assert!(run(&input, None).is_err());
    }

    #[test]
    fn date_duration_round_trips(
        year in 1900i32..=2100,
        month in 1u8..=12,
        day in 1u8..=28,
        days in 0i64..=365,
    ) {
        let input = format!("{year}/{month}/{day} + {days}d - {days}d");
        let expected = date(year, month, day).to_string();
        prop_assert_eq!(run(&input, None).unwrap(), expected);
    }

    #[test]
    fn working_day_addition_matches_weekend_skipping_model(
        year in 1900i32..=2100,
        month in 1u8..=12,
        day in 1u8..=28,
        days in 0i64..=100,
    ) {
        let start = date(year, month, day);
        let expected = add_working_days(start, days).to_string();
        let input = format!("{year}/{month}/{day} + {days}wd");

        prop_assert_eq!(run(&input, None).unwrap(), expected);
    }

    #[test]
    fn working_day_subtraction_matches_weekend_skipping_model(
        year in 1900i32..=2100,
        month in 1u8..=12,
        day in 1u8..=28,
        days in 0i64..=100,
    ) {
        let start = date(year, month, day);
        let expected = add_working_days(start, -days).to_string();
        let input = format!("{year}/{month}/{day} - {days}wd");

        prop_assert_eq!(run(&input, None).unwrap(), expected);
    }
}
