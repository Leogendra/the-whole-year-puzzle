use crate::board::square::{Square, Date};

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use chrono::{Local, Datelike, Duration, Month};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Mode of use
    #[arg(value_enum, default_value_t = Mode::Browse)]
    pub mode: Mode,
    /// Date to show solutions for
    #[arg(short, long, value_parser = parse_date_or_today, default_value = "today")]
    pub date: Date,
    /// Where to look for solutions
    #[arg(short, long)]
    pub file: Option<PathBuf>,
}

#[derive(Clone, ValueEnum, Debug)]
pub enum Mode {
    /// Generate solution set
    Generate,
    /// Browse solutions
    Browse,
    /// Show random solution
    Random,
}

fn parse_today(offset: &str) -> Result<Date, String> {
    let mut chars = offset.trim_start().chars();
    let days = match chars.next() {
        Some('+') => chars.as_str().trim().parse::<i64>()
            .map_err(|_| "date offset must be an integer")?,
        Some('-') => -chars.as_str().trim().parse::<i64>()
            .map_err(|_| "date offset must be an integer")?,
        Some(_) => Err("could not parse date offset")?,
        None => 0,
    };

    let date = Local::now().date_naive() + Duration::days(days);

    Ok(Date {
        month: Square::month(date.month() as usize)
            .ok_or("problem determining month from system time")?,
        day: Square::day(date.day() as usize)
            .ok_or("problem determining day from system time")?,
    })
}

fn parse_date(value: &str) -> Result<Date, String> {
    let month_str = value.split(|c: char| !c.is_alphabetic()).next().ok_or("could not determine month")?;
    let month = month_str.parse::<Month>().map_err(|_| "invalid month")?;

    let rest = value.strip_prefix(month_str).expect("prefix should be known");

    let day_str = rest.strip_prefix(|c: char| c.is_whitespace() || c == ',').unwrap_or(rest);
    let day = day_str.parse::<usize>().map_err(|_| "could not determine day")?;

    if day == 0 || day > 31 {
        Err("day must be in range 1-31")?
    }

    let date = Date {
        month: Square::month(month.number_from_month() as usize).expect("chrono months should be in range 1-12"),
        day: Square::day(day).expect("day check should limit to range 1-31"),
    };

    if date.is_valid() {
        Ok(date)
    } else {
        Err("invalid date".to_string())
    }
}

pub fn parse_date_or_today(value: &str) -> Result<Date, String> {
    let normalized = value.trim().to_lowercase();
    match normalized.as_str().strip_prefix("today") {
        Some(offset) => parse_today(offset),
        _ if value.starts_with(|c| c == '+' || c == '-') => parse_today(normalized.as_str()),
        _ => parse_date(normalized.as_str()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_after_the_28th_are_parsed() {
        assert_eq!(parse_date_or_today("jan 29"), Ok(Date { month: Square::Jan, day: Square::D29 }));
        assert_eq!(parse_date_or_today("July 31"), Ok(Date { month: Square::Jul, day: Square::D31 }));
        assert_eq!(parse_date_or_today("dec 1"), Ok(Date { month: Square::Dec, day: Square::D01 }));
    }

    #[test]
    fn invalid_dates_are_rejected() {
        assert!(parse_date_or_today("feb 30").is_err());
        assert!(parse_date_or_today("apr 31").is_err());
        assert!(parse_date_or_today("jan 32").is_err());
    }
}
