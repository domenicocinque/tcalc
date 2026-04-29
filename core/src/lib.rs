pub mod evaluator;
pub mod lexer;
pub mod parser;

use crate::evaluator::eval_with_calendar;
use crate::lexer::Lexer;
use crate::parser::{Expr, parse};
use toml::Value;

pub use crate::evaluator::Calendar;

pub fn run(input: &str, calendar: Option<&Calendar>) -> Result<String, String> {
    let default_calendar = Calendar::default();
    let calendar = calendar.unwrap_or(&default_calendar);
    let tokens = Lexer::new(input);
    let ast = parse(tokens).map_err(|err| format!("failed to parse expression: {}", err))?;
    let result = eval_with_calendar(&ast, calendar)
        .map_err(|err| format!("failed to evaluate expression: {}", err))?;
    Ok(result.to_string())
}

pub fn calendar_from_holidays(holidays: &[String]) -> Result<Calendar, String> {
    let mut calendar = Calendar::new();

    for holiday in holidays {
        add_holiday_to_calendar(&mut calendar, holiday)?;
    }

    Ok(calendar)
}

pub fn calendar_from_toml(input: &str, calendar_name: Option<&str>) -> Result<Calendar, String> {
    let value = input
        .parse::<Value>()
        .map_err(|err| format!("failed to parse calendar file: {}", err))?;

    let table = match calendar_name {
        Some(name) => value
            .get(name)
            .ok_or_else(|| format!("calendar '{}' not found", name))?,
        None => &value,
    };

    let holidays = table
        .get("holidays")
        .ok_or_else(|| missing_holidays_error(calendar_name))?
        .as_array()
        .ok_or_else(|| holidays_type_error(calendar_name))?;

    let mut calendar = Calendar::new();
    for holiday in holidays {
        let holiday = holiday
            .as_str()
            .ok_or_else(|| holidays_type_error(calendar_name))?;
        add_holiday_to_calendar(&mut calendar, holiday)?;
    }

    Ok(calendar)
}

fn missing_holidays_error(calendar_name: Option<&str>) -> String {
    match calendar_name {
        Some(name) => format!("calendar '{}' must define holidays", name),
        None => "calendar file must define top-level holidays or use --calendar-name".to_string(),
    }
}

fn holidays_type_error(calendar_name: Option<&str>) -> String {
    match calendar_name {
        Some(name) => format!(
            "calendar '{}' holidays must be an array of date strings",
            name
        ),
        None => "calendar holidays must be an array of date strings".to_string(),
    }
}

fn add_holiday_to_calendar(calendar: &mut Calendar, holiday: &str) -> Result<(), String> {
    let tokens = Lexer::new(holiday);
    let ast =
        parse(tokens).map_err(|err| format!("failed to parse holiday '{}': {}", holiday, err))?;

    match ast {
        Expr::Date(year, month, day) => calendar
            .add_holiday_ymd(year, month, day)
            .map_err(|err| format!("invalid holiday '{}': {}", holiday, err)),
        _ => Err(format!("holiday '{}' must be a date", holiday)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_skips_holidays_with_calendar() {
        let holidays = vec!["2024/04/29".to_string()];
        let calendar = calendar_from_holidays(&holidays).unwrap();

        let result = run("2024/04/26 + 1wd", Some(&calendar)).unwrap();

        assert_eq!(result, "2024-04-30");
    }

    #[test]
    fn calendar_from_holidays_rejects_non_date() {
        let holidays = vec!["2h".to_string()];

        assert!(calendar_from_holidays(&holidays).is_err());
    }

    #[test]
    fn calendar_from_toml_reads_top_level_holidays() {
        let calendar = calendar_from_toml(
            r#"
            holidays = ["2024/04/29"]
            "#,
            None,
        )
        .unwrap();

        let result = run("2024/04/26 + 1wd", Some(&calendar)).unwrap();

        assert_eq!(result, "2024-04-30");
    }

    #[test]
    fn calendar_from_toml_reads_named_calendar() {
        let calendar = calendar_from_toml(
            r#"
            [italy]
            holidays = ["2024/04/29"]
            "#,
            Some("italy"),
        )
        .unwrap();

        let result = run("2024/04/26 + 1wd", Some(&calendar)).unwrap();

        assert_eq!(result, "2024-04-30");
    }

    #[test]
    fn calendar_from_toml_requires_name_for_named_calendar_only_file() {
        let result = calendar_from_toml(
            r#"
            [italy]
            holidays = ["2024/04/29"]
            "#,
            None,
        );

        assert!(result.is_err());
    }
}
