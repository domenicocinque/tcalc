use crate::parser::{Expr, Op};
use crate::parser::{Keyword, Unit};

use std::fmt;
use time::{Date, Duration, Month, OffsetDateTime, Time, UtcOffset};

const DAYS_PER_MONTH_APPROX: i64 = 30;
const DAYS_PER_YEAR_APPROX: i64 = 365;

#[derive(Debug)]
pub enum EvalError {
    InvalidDate(u32, u8, u8),
    InvalidMonth(u8),
    InvalidTime(u8, u8, u8),
    InvalidOp(Op, Value, Value),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EvalError::InvalidDate(year, month, day) => {
                write!(f, "invalid date '{}-{}-{}'", year, month, day)
            }
            EvalError::InvalidMonth(month) => write!(f, "invalid month '{}'", month),
            EvalError::InvalidTime(hour, minute, second) => {
                write!(f, "invalid time '{}:{}:{}'", hour, minute, second)
            }
            EvalError::InvalidOp(op, left, right) => {
                write!(
                    f,
                    "invalid operation '{}' for '{}' and '{}'",
                    op,
                    left.type_name(),
                    right.type_name(),
                )
            }
        }
    }
}

impl std::error::Error for EvalError {}

#[derive(Debug, Copy, Clone)]
pub enum Value {
    Date(Date),
    DateTime(OffsetDateTime),
    Duration(Duration),
    Time(Time),
}

impl Value {
    fn from_date(year: u32, month: u8, day: u8) -> Result<Self, EvalError> {
        let month = Month::try_from(month).map_err(|_| EvalError::InvalidMonth(month))?;
        let date = Date::from_calendar_date(
            year.try_into()
                .map_err(|_| EvalError::InvalidDate(year, month.into(), day))?,
            month,
            day,
        )
        .map_err(|_| EvalError::InvalidDate(year, month.into(), day))?;
        Ok(Value::Date(date))
    }

    fn from_time(hour: u8, minute: u8, second: u8) -> Result<Self, EvalError> {
        let time = Time::from_hms(hour, minute, second)
            .map_err(|_| EvalError::InvalidTime(hour, minute, second))?;
        Ok(Value::Time(time))
    }

    fn from_duration(value: i64, unit: &Unit) -> Result<Self, EvalError> {
        let duration = match unit {
            Unit::Years => Duration::days(value * DAYS_PER_YEAR_APPROX),
            Unit::Months => Duration::days(value * DAYS_PER_MONTH_APPROX),
            Unit::Days => Duration::days(value),
            Unit::Hours => Duration::hours(value),
            Unit::Minutes => Duration::minutes(value),
            Unit::Seconds => Duration::seconds(value),
        };
        Ok(Value::Duration(duration))
    }

    fn from_keyword(keyword: &Keyword) -> Result<Self, EvalError> {
        match keyword {
            Keyword::Now => {
                let now = OffsetDateTime::now_utc();
                Ok(Value::DateTime(now))
            }
            Keyword::Today => {
                let now = OffsetDateTime::now_utc();
                Ok(Value::Date(now.date()))
            }
            Keyword::Tomorrow => {
                let now = OffsetDateTime::now_utc();
                Ok(Value::Date(now.date() + Duration::days(1)))
            }
            Keyword::Yesterday => {
                let now = OffsetDateTime::now_utc();
                Ok(Value::Date(now.date() - Duration::days(1)))
            }
        }
    }

    fn from_datetime(
        year: u32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
    ) -> Result<Self, EvalError> {
        let month = Month::try_from(month).map_err(|_| EvalError::InvalidMonth(month))?;
        let date = Date::from_calendar_date(year as i32, month, day)
            .map_err(|_| EvalError::InvalidDate(year, month.into(), day))?;
        let time =
            Time::from_hms(hour, minute, 0).map_err(|_| EvalError::InvalidTime(hour, minute, 0))?;
        let offset = UtcOffset::UTC;
        Ok(Value::DateTime(OffsetDateTime::new_in_offset(
            date, time, offset,
        )))
    }

    fn add(self, other: Value) -> Result<Value, EvalError> {
        match (self, other) {
            (Value::Date(left), Value::Duration(right)) => Ok(Value::Date(left + right)),
            (Value::DateTime(left), Value::Duration(right)) => Ok(Value::DateTime(left + right)),
            (Value::Time(left), Value::Duration(right)) => Ok(Value::Time(left + right)),
            (Value::Duration(left), Value::Duration(right)) => Ok(Value::Duration(left + right)),
            _ => Err(EvalError::InvalidOp(Op::Add, self, other)),
        }
    }

    fn sub(self, other: Value) -> Result<Value, EvalError> {
        match (self, other) {
            (Value::Date(left), Value::Duration(right)) => Ok(Value::Date(left - right)),
            (Value::DateTime(left), Value::Duration(right)) => Ok(Value::DateTime(left - right)),
            (Value::Time(left), Value::Duration(right)) => Ok(Value::Time(left - right)),
            (Value::Duration(left), Value::Duration(right)) => Ok(Value::Duration(left - right)),
            (Value::Date(left), Value::Date(right)) => Ok(Value::Duration(left - right)),
            _ => Err(EvalError::InvalidOp(Op::Sub, self, other)),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Date(_) => "Date",
            Value::DateTime(_) => "DateTime",
            Value::Duration(_) => "Duration",
            Value::Time(_) => "Time",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Date(d) => write_date(f, *d),
            Value::DateTime(dt) => write_datetime(f, *dt),
            Value::Duration(dur) => dur.fmt(f),
            Value::Time(t) => write_time(f, *t),
        }
    }
}

fn write_date(f: &mut fmt::Formatter, date: Date) -> fmt::Result {
    write!(
        f,
        "{:04}-{:02}-{:02}",
        date.year(),
        date.month() as u8,
        date.day()
    )
}

fn write_time(f: &mut fmt::Formatter, time: Time) -> fmt::Result {
    write!(f, "{:02}:{:02}", time.hour(), time.minute())?;

    let second = time.second();
    let nanosecond = time.nanosecond();

    if second != 0 || nanosecond != 0 {
        write!(f, ":{:02}", second)?;

        if nanosecond != 0 {
            let mut subseconds = format!("{:09}", nanosecond);
            while subseconds.ends_with('0') {
                subseconds.pop();
            }
            write!(f, ".{}", subseconds)?;
        }
    }

    Ok(())
}

fn write_datetime(f: &mut fmt::Formatter, datetime: OffsetDateTime) -> fmt::Result {
    write_date(f, datetime.date())?;
    write!(f, " ")?;
    write_time(f, datetime.time())?;

    let offset = datetime.offset();
    if offset.whole_seconds() != 0 {
        write!(f, " {}", format_offset(offset))?;
    } else {
        write!(f, " +00:00")?;
    }

    Ok(())
}

fn format_offset(offset: UtcOffset) -> String {
    let total_seconds = offset.whole_seconds();
    let sign = if total_seconds >= 0 { '+' } else { '-' };
    let total_seconds = total_seconds.abs();

    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if seconds == 0 {
        format!("{}{:02}:{:02}", sign, hours, minutes)
    } else {
        format!("{}{:02}:{:02}:{:02}", sign, hours, minutes, seconds)
    }
}

pub fn eval(expr: &Expr) -> Result<Value, EvalError> {
    match expr {
        Expr::BinOp(left, op, right) => {
            let left = eval(left)?;
            let right = eval(right)?;

            match op {
                Op::Add => left.add(right),
                Op::Sub => left.sub(right),
            }
        }
        Expr::Time(hour, minute) => Ok(Value::from_time(*hour, *minute, 0)?),
        Expr::Date(year, month, day) => Ok(Value::from_date(*year, *month, *day)?),
        Expr::Duration(value, unit) => Ok(Value::from_duration(*value, unit)?),
        Expr::Keyword(keyword) => Ok(Value::from_keyword(keyword)?),
        Expr::DateTime(year, month, day, hour, minute) => {
            Ok(Value::from_datetime(*year, *month, *day, *hour, *minute)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Expr, Op};
    use time::{Date, Duration, Month, OffsetDateTime, Time, UtcOffset};

    #[test]
    fn test_literal_date() {
        let expr = Expr::Date(2025, 9, 27);
        let val = eval(&expr).unwrap();
        match val {
            Value::Date(date) => assert_eq!(
                date,
                Date::from_calendar_date(2025, Month::September, 27).unwrap()
            ),
            _ => panic!("Expected Value::Date"),
        }
    }

    #[test]
    fn test_literal_time() {
        let expr = Expr::Time(12, 30);
        let val = eval(&expr).unwrap();
        match val {
            Value::Time(time) => assert_eq!(time, Time::from_hms(12, 30, 0).unwrap()),
            _ => panic!("Expected Value::Time"),
        }
    }

    #[test]
    fn test_duration_days() {
        let expr = Expr::Duration(3, Unit::Days);
        let val = eval(&expr).unwrap();
        match val {
            Value::Duration(dur) => assert_eq!(dur, Duration::days(3)),
            _ => panic!("Expected Value::Duration"),
        }
    }

    #[test]
    fn test_add_date_duration() {
        let expr = Expr::BinOp(
            Box::new(Expr::Date(2025, 9, 27)),
            Op::Add,
            Box::new(Expr::Duration(2, Unit::Days)),
        );
        let val = eval(&expr).unwrap();
        match val {
            Value::Date(date) => assert_eq!(
                date,
                Date::from_calendar_date(2025, Month::September, 29).unwrap()
            ),
            _ => panic!("Expected Value::Date"),
        }
    }

    #[test]
    fn test_sub_date_duration() {
        let expr = Expr::BinOp(
            Box::new(Expr::Date(2025, 9, 27)),
            Op::Sub,
            Box::new(Expr::Duration(7, Unit::Days)),
        );
        let val = eval(&expr).unwrap();
        match val {
            Value::Date(date) => assert_eq!(
                date,
                Date::from_calendar_date(2025, Month::September, 20).unwrap()
            ),
            _ => panic!("Expected Value::Date"),
        }
    }

    #[test]
    fn test_keyword_today() {
        let expr = Expr::Keyword(Keyword::Today);
        let val = eval(&expr).unwrap();
        match val {
            Value::Date(_) => {}
            _ => panic!("Expected Value::Date"),
        }
    }

    #[test]
    fn test_invalid_addition() {
        let expr = Expr::BinOp(
            Box::new(Expr::Date(2025, 9, 27)),
            Op::Add,
            Box::new(Expr::Date(2025, 9, 28)),
        );
        let val = eval(&expr);
        assert!(val.is_err());
    }

    #[test]
    fn test_display_date_formats_consistently() {
        let date = Date::from_calendar_date(2024, Month::January, 5).unwrap();
        assert_eq!(Value::Date(date).to_string(), "2024-01-05");
    }

    #[test]
    fn test_display_time_omits_seconds_when_zero() {
        let time = Time::from_hms(2, 0, 0).unwrap();
        assert_eq!(Value::Time(time).to_string(), "02:00");
    }

    #[test]
    fn test_display_time_includes_seconds() {
        let time = Time::from_hms(2, 0, 30).unwrap();
        assert_eq!(Value::Time(time).to_string(), "02:00:30");
    }

    #[test]
    fn test_display_time_includes_fractional_seconds() {
        let time = Time::from_hms_nano(2, 0, 30, 120_000_000).unwrap();
        assert_eq!(Value::Time(time).to_string(), "02:00:30.12");
    }

    #[test]
    fn test_display_datetime_utc_offset() {
        let date = Date::from_calendar_date(2024, Month::January, 5).unwrap();
        let time = Time::from_hms(8, 15, 0).unwrap();
        let dt = OffsetDateTime::new_in_offset(date, time, UtcOffset::UTC);
        assert_eq!(Value::DateTime(dt).to_string(), "2024-01-05 08:15 +00:00");
    }

    #[test]
    fn test_display_datetime_nonzero_offset() {
        let date = Date::from_calendar_date(2024, Month::January, 5).unwrap();
        let time = Time::from_hms(8, 15, 0).unwrap();
        let offset = UtcOffset::from_hms(5, 30, 0).unwrap();
        let dt = OffsetDateTime::new_in_offset(date, time, offset);
        assert_eq!(Value::DateTime(dt).to_string(), "2024-01-05 08:15 +05:30");
    }
}
