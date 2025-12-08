use std::iter::Peekable;

use crate::lexer::{Lexer, Token};

const HOURS_IN_HALF_DAY: i64 = 12;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Date(u32, u8, u8),
    Time(u8, u8),
    DateTime(u32, u8, u8, u8, u8),
    Keyword(Keyword),
    Duration(i64, Unit),
    BinOp(Box<Expr>, Op, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Add,
    Sub,
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Op::Add => write!(f, "+"),
            Op::Sub => write!(f, "-"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Keyword {
    Today,
    Now,
    Tomorrow,
    Yesterday,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Unit {
    Years,
    Months,
    Days,
    Hours,
    Minutes,
    Seconds,
}

impl TryFrom<&str> for Unit {
    type Error = ParsingError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "years" | "year" | "y" => Ok(Unit::Years),
            "months" | "month" => Ok(Unit::Months),
            "days" | "day" | "d" => Ok(Unit::Days),
            "hours" | "hour" | "h" => Ok(Unit::Hours),
            "minutes" | "minute" | "m" => Ok(Unit::Minutes),
            "seconds" | "second" | "s" => Ok(Unit::Seconds),
            _ => Err(ParsingError::UnknownKeyword(value.to_string())),
        }
    }
}

#[derive(Debug)]
pub enum ParsingError {
    UnexpectedToken(Token),
    UnknownKeyword(String),
    UnexpectedIdent(String),
    UnexpectedEof,
    ExpectedIdent,
    ExpectedNumber,
    ExpectedSlash,
    ExpectedColon,
    ExpectedUnit,
    InvalidYear(i64),
    InvalidTime(String),
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParsingError::UnexpectedToken(token) => write!(f, "unexpected token '{}'", token),
            ParsingError::UnknownKeyword(keyword) => write!(f, "unknown keyword '{}'", keyword),
            ParsingError::UnexpectedIdent(ident) => write!(f, "unexpected identifier '{}'", ident),
            ParsingError::UnexpectedEof => write!(f, "unexpected end of input"),
            ParsingError::ExpectedIdent => write!(f, "expected identifier"),
            ParsingError::ExpectedNumber => write!(f, "expected number"),
            ParsingError::ExpectedSlash => write!(f, "expected slash"),
            ParsingError::ExpectedColon => write!(f, "expected colon"),
            ParsingError::ExpectedUnit => write!(f, "expected unit"),
            ParsingError::InvalidYear(year) => write!(f, "invalid year '{}'", year),
            ParsingError::InvalidTime(time_string) => write!(f, "invalid time '{}'", time_string),
        }
    }
}

impl std::error::Error for ParsingError {}

/// Grammar
///
/// <expr> ::= <primary> (('+' | '-') <primary>)*
/// <primary> ::= <datetime> | <time> | <duration> | <keyword>
/// <datetime> ::= <date> <time>?
/// <date> ::= NUMBER '/' NUMBER '/' NUMBER
/// <time> ::= NUMBER ':' NUMBER | NUMBER ("am" | "pm")
pub fn parse(lexer: Lexer) -> Result<Expr, ParsingError> {
    let mut tokens = lexer.into_iter().peekable();
    parse_expr(&mut tokens)
}

fn parse_expr(tokens: &mut Peekable<Lexer>) -> Result<Expr, ParsingError> {
    let mut left = parse_primary(tokens)?;

    while let Some(Token::Plus | Token::Minus) = tokens.peek() {
        let op = match tokens.next() {
            Some(Token::Plus) => Op::Add,
            Some(Token::Minus) => Op::Sub,
            Some(token) => return Err(ParsingError::UnexpectedToken(token)),
            None => return Err(ParsingError::UnexpectedEof),
        };

        let right = parse_primary(tokens)?;
        left = Expr::BinOp(Box::new(left), op, Box::new(right));
    }

    Ok(left)
}

fn parse_primary(tokens: &mut Peekable<Lexer>) -> Result<Expr, ParsingError> {
    match tokens.peek() {
        Some(Token::Number(_)) => parse_number(tokens),
        Some(Token::Ident(_)) => parse_ident(tokens),
        Some(token) => Err(ParsingError::UnexpectedToken(token.clone())),
        None => Err(ParsingError::UnexpectedEof),
    }
}

fn parse_ident(tokens: &mut Peekable<Lexer>) -> Result<Expr, ParsingError> {
    match tokens.next() {
        Some(Token::Ident(s)) => match s.as_str() {
            "today" => Ok(Expr::Keyword(Keyword::Today)),
            "tomorrow" => Ok(Expr::Keyword(Keyword::Tomorrow)),
            "yesterday" => Ok(Expr::Keyword(Keyword::Yesterday)),
            "now" => Ok(Expr::Keyword(Keyword::Now)),
            _ => Err(ParsingError::UnknownKeyword(s)),
        },
        _ => Err(ParsingError::ExpectedIdent),
    }
}

fn parse_number(tokens: &mut Peekable<Lexer>) -> Result<Expr, ParsingError> {
    let first_num = expect_number(tokens)?;

    match tokens.peek() {
        Some(Token::Slash) => parse_date(tokens, first_num),
        Some(Token::Colon) => parse_time(tokens, first_num),
        Some(Token::Ident(ident)) => match ident.as_str() {
            "am" => {
                tokens.next();
                match first_num {
                    1..=11 => return Ok(Expr::Time(first_num as u8, 0)),
                    12 => return Ok(Expr::Time(0, 0)),
                    _ => return Err(ParsingError::InvalidTime(format!("{first_num} am"))),
                }
            }
            "pm" => {
                tokens.next();
                match first_num {
                    1..=11 => return Ok(Expr::Time((first_num + HOURS_IN_HALF_DAY) as u8, 0)),
                    12 => return Ok(Expr::Time(12, 0)),
                    _ => return Err(ParsingError::InvalidTime(format!("{first_num} pm"))),
                }
            }
            _ => parse_duration(tokens, first_num),
        },
        Some(token) => Err(ParsingError::UnexpectedToken(token.clone())),
        None => Err(ParsingError::UnexpectedEof),
    }
}

fn parse_date(tokens: &mut Peekable<Lexer>, year: i64) -> Result<Expr, ParsingError> {
    expect_token(tokens, Token::Slash, ParsingError::ExpectedSlash)?;
    let month = expect_number(tokens)?;
    expect_token(tokens, Token::Slash, ParsingError::ExpectedSlash)?;
    let day = expect_number(tokens)?;

    if let Some(Token::Number(_)) = tokens.peek() {
        let hour = expect_number(tokens)?;
        expect_token(tokens, Token::Colon, ParsingError::ExpectedColon)?;
        let minute = expect_number(tokens)?;
        Ok(Expr::DateTime(
            year as u32,
            month as u8,
            day as u8,
            hour as u8,
            minute as u8,
        ))
    } else {
        Ok(Expr::Date(year as u32, month as u8, day as u8))
    }
}

fn parse_time(tokens: &mut Peekable<Lexer>, hour: i64) -> Result<Expr, ParsingError> {
    expect_token(tokens, Token::Colon, ParsingError::ExpectedColon)?;
    let minute = expect_number(tokens)?;
    Ok(Expr::Time(hour as u8, minute as u8))
}

fn parse_duration(tokens: &mut Peekable<Lexer>, value: i64) -> Result<Expr, ParsingError> {
    match tokens.next() {
        Some(Token::Ident(u)) => Ok(Expr::Duration(value, Unit::try_from(u.as_str())?)),
        _ => Err(ParsingError::ExpectedUnit),
    }
}

fn expect_token(
    tokens: &mut Peekable<Lexer>,
    expected: Token,
    err: ParsingError,
) -> Result<(), ParsingError> {
    match tokens.next() {
        Some(t) if t == expected => Ok(()),
        Some(t) => Err(ParsingError::UnexpectedToken(t)),
        None => Err(err),
    }
}

fn expect_number(tokens: &mut Peekable<Lexer>) -> Result<i64, ParsingError> {
    match tokens.next() {
        Some(Token::Number(n)) => Ok(n),
        _ => Err(ParsingError::ExpectedNumber),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date() {
        let lexer = Lexer::new("2023/01/01");
        let expr = parse(lexer).unwrap();
        assert_eq!(expr, Expr::Date(2023, 1, 1));
    }

    #[test]
    fn test_parse_time_24h() {
        let lexer = Lexer::new("14:30");
        let expr = parse(lexer).unwrap();
        assert_eq!(expr, Expr::Time(14, 30));
    }

    #[test]
    fn test_parse_time_am() {
        let lexer = Lexer::new("2am");
        let expr = parse(lexer).unwrap();
        assert_eq!(expr, Expr::Time(2, 0));
    }

    #[test]
    fn test_parse_time_pm() {
        let lexer = Lexer::new("2pm");
        let expr = parse(lexer).unwrap();
        assert_eq!(expr, Expr::Time(14, 0));
    }

    #[test]
    fn test_parse_time_12am() {
        let lexer = Lexer::new("12am");
        let expr = parse(lexer).unwrap();
        assert_eq!(expr, Expr::Time(0, 0));
    }

    #[test]
    fn test_parse_time_12pm() {
        let lexer = Lexer::new("12pm");
        let expr = parse(lexer).unwrap();
        assert_eq!(expr, Expr::Time(12, 0));
    }

    #[test]
    fn test_parse_time_invalid_hour_overflow() {
        let lexer = Lexer::new("34pm");
        assert!(parse(lexer).is_err());
    }

    #[test]
    fn test_parse_time_invalid_hour_zero() {
        let lexer = Lexer::new("0am");
        assert!(parse(lexer).is_err());
    }

    #[test]
    fn test_parse_duration_hours() {
        let lexer = Lexer::new("2h");
        let expr = parse(lexer).unwrap();
        assert_eq!(expr, Expr::Duration(2, Unit::Hours));
    }

    #[test]
    fn test_parse_duration_minutes() {
        let lexer = Lexer::new("30m");
        let expr = parse(lexer).unwrap();
        assert_eq!(expr, Expr::Duration(30, Unit::Minutes));
    }

    #[test]
    fn test_parse_keyword_today() {
        let lexer = Lexer::new("today");
        let expr = parse(lexer).unwrap();
        assert_eq!(expr, Expr::Keyword(Keyword::Today));
    }

    #[test]
    fn test_parse_keyword_tomorrow() {
        let lexer = Lexer::new("tomorrow");
        let expr = parse(lexer).unwrap();
        assert_eq!(expr, Expr::Keyword(Keyword::Tomorrow));
    }

    #[test]
    fn test_parse_datetime() {
        let lexer = Lexer::new("2023/01/01 14:30");
        let expr = parse(lexer).unwrap();
        assert_eq!(expr, Expr::DateTime(2023, 1, 1, 14, 30));
    }

    #[test]
    fn test_parse_addition() {
        let lexer = Lexer::new("today + 2h");
        let expr = parse(lexer).unwrap();
        assert_eq!(
            expr,
            Expr::BinOp(
                Box::new(Expr::Keyword(Keyword::Today)),
                Op::Add,
                Box::new(Expr::Duration(2, Unit::Hours))
            )
        );
    }

    #[test]
    fn test_parse_subtraction() {
        let lexer = Lexer::new("2am - 30m");
        let expr = parse(lexer).unwrap();
        assert_eq!(
            expr,
            Expr::BinOp(
                Box::new(Expr::Time(2, 0)),
                Op::Sub,
                Box::new(Expr::Duration(30, Unit::Minutes))
            )
        );
    }

    #[test]
    fn test_parse_chained_operations() {
        let lexer = Lexer::new("today - 2h + 30m");
        let expr = parse(lexer).unwrap();
        // Should be: ((today - 2h) + 30m)
        assert_eq!(
            expr,
            Expr::BinOp(
                Box::new(Expr::BinOp(
                    Box::new(Expr::Keyword(Keyword::Today)),
                    Op::Sub,
                    Box::new(Expr::Duration(2, Unit::Hours))
                )),
                Op::Add,
                Box::new(Expr::Duration(30, Unit::Minutes))
            )
        );
    }

    #[test]
    fn test_parse_duration_addition() {
        let lexer = Lexer::new("2h + 30m");
        let expr = parse(lexer).unwrap();
        assert_eq!(
            expr,
            Expr::BinOp(
                Box::new(Expr::Duration(2, Unit::Hours)),
                Op::Add,
                Box::new(Expr::Duration(30, Unit::Minutes))
            )
        );
    }

    #[test]
    fn test_parse_date_arithmetic() {
        let lexer = Lexer::new("2023/12/25 + 7d");
        let expr = parse(lexer).unwrap();
        assert_eq!(
            expr,
            Expr::BinOp(
                Box::new(Expr::Date(2023, 12, 25)),
                Op::Add,
                Box::new(Expr::Duration(7, Unit::Days))
            )
        );
    }
}
