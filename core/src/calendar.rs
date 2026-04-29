use std::collections::HashSet;

use time::{Date, Duration, Month, OffsetDateTime, Weekday};

use crate::evaluator::EvalError;

#[derive(Debug, Clone, Default)]
pub struct Calendar {
    holidays: HashSet<Date>,
}

impl Calendar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_holiday(&mut self, date: Date) {
        self.holidays.insert(date);
    }

    pub fn extend(&mut self, other: &Calendar) {
        self.holidays.extend(other.holidays.iter().copied());
    }

    pub fn add_holiday_ymd(&mut self, year: u32, month: u8, day: u8) -> Result<(), EvalError> {
        self.add_holiday(date_from_parts(year, month, day)?);
        Ok(())
    }

    fn is_working_day(&self, date: Date) -> bool {
        !self.holidays.contains(&date)
            && !matches!(date.weekday(), Weekday::Saturday | Weekday::Sunday)
    }
}

pub fn add_datetime_working_days(
    datetime: OffsetDateTime,
    days: i64,
    calendar: &Calendar,
) -> OffsetDateTime {
    let date = add_working_days(datetime.date(), days, calendar);
    OffsetDateTime::new_in_offset(date, datetime.time(), datetime.offset())
}

pub fn add_working_days(mut date: Date, days: i64, calendar: &Calendar) -> Date {
    let step = if days >= 0 { 1 } else { -1 };
    let mut remaining = days.abs();

    while remaining > 0 {
        date += Duration::days(step);

        if calendar.is_working_day(date) {
            remaining -= 1;
        }
    }

    date
}

pub fn date_from_parts(year: u32, month: u8, day: u8) -> Result<Date, EvalError> {
    let month = Month::try_from(month).map_err(|_| EvalError::Month(month))?;
    Date::from_calendar_date(
        year.try_into()
            .map_err(|_| EvalError::Date(year, month.into(), day))?,
        month,
        day,
    )
    .map_err(|_| EvalError::Date(year, month.into(), day))
}
