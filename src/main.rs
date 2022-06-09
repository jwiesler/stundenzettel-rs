use std::num::NonZeroU32;

use clap::{CommandFactory, ErrorKind, Parser};
use rand::thread_rng;

use crate::calendar::{non_holidays_of_month, Month, Year};
use crate::generate::{generate_times, Parameters};

mod calendar;
mod generate;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Arguments {
    /// The month
    #[clap(parse(try_from_str = month_in_range))]
    month: NonZeroU32,
    /// The year
    #[clap(parse(try_from_str = year_in_range))]
    year: u32,
    /// Hours to assign
    hours: NonZeroU32,

    /// Output csv
    #[clap(long)]
    csv: bool,

    /// Maximum assignable hours per day
    #[clap(parse(try_from_str = hour_in_range), default_value_t = 8)]
    max_per_day: u32,
    /// Earliest assignable starting hour
    #[clap(parse(try_from_str = hour_in_range), default_value_t = 8)]
    earliest: u32,
    /// Latest assignable stopping hour
    #[clap(parse(try_from_str = hour_in_range), default_value_t = 20)]
    latest: u32,
}

fn hour_in_range(s: &str) -> Result<u32, String> {
    let hour: u32 = s.parse().map_err(|_| format!("`{}` isn't an hour", s))?;
    if hour <= 24 {
        Ok(hour)
    } else {
        Err("Hour has to be <= 24".into())
    }
}

fn year_in_range(s: &str) -> Result<u32, String> {
    let year: u32 = s.parse().map_err(|_| format!("`{}` isn't a year", s))?;
    if year >= 1970 {
        Ok(year)
    } else {
        Err("Year has to be >= 1970".into())
    }
}

fn month_in_range(s: &str) -> Result<NonZeroU32, String> {
    let month: NonZeroU32 = s.parse().map_err(|_| format!("`{}` isn't a month", s))?;
    if month.get() <= 12 {
        Ok(month)
    } else {
        Err("Month has to be <= 12".into())
    }
}

fn main() {
    let Arguments {
        month,
        year,
        hours,
        max_per_day,
        earliest,
        latest,
        csv,
    } = Arguments::parse();

    if latest < earliest {
        Arguments::command()
            .error(
                ErrorKind::ArgumentConflict,
                &format!("Earliest has to be before latest"),
            )
            .exit();
    }

    let max_per_day = if earliest + max_per_day <= latest {
        max_per_day
    } else {
        let max_per_day = latest - earliest;
        println!(
            "Reducing max hours per day to {} (time restrictions)",
            max_per_day
        );
        max_per_day
    };

    let year = Year::new(year);
    let month = Month::new(month, &year);
    let days = non_holidays_of_month(&month, &year);

    if max_per_day.saturating_mul(days.len().try_into().unwrap()) < hours.get() {
        Arguments::command()
            .error(
                ErrorKind::ArgumentConflict,
                &format!(
                    "Can't distribute {} hours into {} days with at most {} hours per day",
                    hours.get(),
                    days.len(),
                    max_per_day
                ),
            )
            .exit();
    }

    let mut rng = thread_rng();
    let times = generate_times(
        Parameters {
            hours: hours.get(),
            days: days.len().try_into().unwrap(),
            from: earliest,
            to: latest,
            max_per_day,
        },
        &mut rng,
    );

    let check = times
        .iter()
        .map(|v| v.as_ref().map(|t| t.to - t.from).unwrap_or_default())
        .sum::<u32>();
    assert_eq!(check, hours.get());

    times.iter().zip(&days).for_each(|(time, day)| {
        if let Some(time) = time {
            if csv {
                println!(
                    "{}.{}.{},{}:00,{}:00",
                    day.day_of_month,
                    month.month(),
                    year.year(),
                    time.from,
                    time.to
                );
            } else {
                println!(
                    "{}.{}.{}: {}:00-{}:00",
                    day.day_of_month,
                    month.month(),
                    year.year(),
                    time.from,
                    time.to
                );
            }
        }
    });
}
