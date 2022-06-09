use std::num::NonZeroU32;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DayOfWeek {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl DayOfWeek {
    fn is_weekend(&self) -> bool {
        match self {
            DayOfWeek::Sunday => true,
            DayOfWeek::Monday => false,
            DayOfWeek::Tuesday => false,
            DayOfWeek::Wednesday => false,
            DayOfWeek::Thursday => false,
            DayOfWeek::Friday => false,
            DayOfWeek::Saturday => true,
        }
    }

    fn next(&self) -> Self {
        match self {
            DayOfWeek::Sunday => DayOfWeek::Monday,
            DayOfWeek::Monday => DayOfWeek::Tuesday,
            DayOfWeek::Tuesday => DayOfWeek::Wednesday,
            DayOfWeek::Wednesday => DayOfWeek::Thursday,
            DayOfWeek::Thursday => DayOfWeek::Friday,
            DayOfWeek::Friday => DayOfWeek::Saturday,
            DayOfWeek::Saturday => DayOfWeek::Sunday,
        }
    }
}

#[derive(Debug)]
pub struct NotADayOfWeek;

impl TryFrom<u32> for DayOfWeek {
    type Error = NotADayOfWeek;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DayOfWeek::Sunday),
            1 => Ok(DayOfWeek::Monday),
            2 => Ok(DayOfWeek::Tuesday),
            3 => Ok(DayOfWeek::Wednesday),
            4 => Ok(DayOfWeek::Thursday),
            5 => Ok(DayOfWeek::Friday),
            6 => Ok(DayOfWeek::Saturday),
            _ => Err(NotADayOfWeek),
        }
    }
}

fn is_leap_year(year: u32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_of_month(month: u32, leap_year: bool) -> u32 {
    let days = match month {
        2 => {
            if leap_year {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    days
}

mod codes {
    use std::num::NonZeroU32;

    use crate::calendar::DayOfWeek;

    pub fn get_month(month: NonZeroU32) -> u32 {
        const MONTH_CODES: [u32; 12] = [0, 3, 3, 6, 1, 4, 6, 2, 5, 0, 3, 5];
        MONTH_CODES[month.get() as usize - 1]
    }

    pub fn get_year(year: u32) -> u32 {
        let y = year % 100;
        (y + (y / 4)) % 7
    }

    pub fn get_century(year: u32) -> u32 {
        ((year / 100 + 1) * 6) % 8
    }

    pub fn day_of_week(day: u32, month_code: u32) -> DayOfWeek {
        DayOfWeek::try_from((day + month_code) % 7).unwrap()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DateOfYear {
    pub day: NonZeroU32,
    pub month: NonZeroU32,
}

impl DateOfYear {
    pub const fn new(day: NonZeroU32, month: NonZeroU32) -> Self {
        DateOfYear { day, month }
    }

    pub fn new_checked(day: u32, month: u32) -> Option<Self> {
        Some(DateOfYear {
            day: NonZeroU32::new(day)?,
            month: NonZeroU32::new(month)?,
        })
    }

    pub fn add_days(&self, days: i32, leap_year: bool) -> Self {
        const DAYS_TO_MONTH: [[u32; 12]; 2] = [
            [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334],
            [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335],
        ];
        let days_to_month = DAYS_TO_MONTH[leap_year as usize];
        let day_of_year: i32 = (days_to_month[self.month.get() as usize - 1] + self.day.get())
            .try_into()
            .unwrap();
        let day_of_year = (day_of_year + days).try_into().unwrap();
        let month = match days_to_month.binary_search(&day_of_year) {
            Ok(v) | Err(v) => v.checked_sub(1).unwrap(),
        };
        return Self {
            day: NonZeroU32::new(day_of_year - days_to_month[month]).unwrap(),
            month: NonZeroU32::new(month as u32 + 1).unwrap(),
        };
    }
}

pub struct Year {
    year: u32,
    is_leap: bool,
    combined_code: u32,
}

impl Year {
    pub fn new(year: u32) -> Self {
        let year_code = codes::get_year(year);
        let century_code = codes::get_century(year);
        let is_leap = is_leap_year(year);
        let combined_code = year_code + century_code - is_leap as u32;
        Self {
            year,
            is_leap,
            combined_code,
        }
    }

    pub fn year(&self) -> u32 {
        self.year
    }

    pub fn days_of_month(&self, month: NonZeroU32) -> u32 {
        days_of_month(month.get(), self.is_leap)
    }

    pub fn easter(&self) -> DateOfYear {
        let k = self.year as i32 / 100;
        let m = 15 + (3 * k + 3) / 4 - (8 * k + 13) / 25;
        let s = 2 - (3 * k + 3) / 4;
        let a = self.year as i32 % 19;
        let d = (19 * a + m) % 30;
        let r = (d + a / 11) / 29;
        let og = 21 + d - r;
        let sz = 7 - (self.year as i32 + self.year as i32 / 4 + s) % 7;
        let oe = 7 - (og - sz) % 7;
        let os = og + oe;
        return DateOfYear::new(NonZeroU32::new(1).unwrap(), NonZeroU32::new(3).unwrap())
            .add_days((os - 1).try_into().unwrap(), self.is_leap);
    }

    pub fn holidays(&self) -> [DateOfYear; 13] {
        let easter = self.easter();
        let new_years_day = DateOfYear::new_checked(1, 1).unwrap();
        let epiphany = DateOfYear::new_checked(6, 1).unwrap();
        let good_friday = easter.add_days(-2, self.is_leap);
        let easter_monday = easter.add_days(1, self.is_leap);
        let labor_day = DateOfYear::new_checked(1, 5).unwrap();
        let ascension_day = easter.add_days(39, self.is_leap);
        let whit_monday = easter.add_days(50, self.is_leap);
        let corpus_christi = easter.add_days(60, self.is_leap);
        let assumption_day = DateOfYear::new_checked(15, 8).unwrap();
        let german_unity_day = DateOfYear::new_checked(3, 10).unwrap();
        let all_saints = DateOfYear::new_checked(1, 11).unwrap();
        let christmas_day = DateOfYear::new_checked(25, 12).unwrap();
        let boxing_day = DateOfYear::new_checked(26, 12).unwrap();

        [
            new_years_day,
            epiphany,
            good_friday,
            easter_monday,
            labor_day,
            ascension_day,
            whit_monday,
            corpus_christi,
            assumption_day,
            german_unity_day,
            all_saints,
            christmas_day,
            boxing_day,
        ]
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DayOfMonth {
    pub day_of_week: DayOfWeek,
    pub day_of_month: NonZeroU32,
}

pub struct Month {
    month: NonZeroU32,
    combined_code: u32,
    num_days: u32,
}

impl Month {
    pub fn new(month: NonZeroU32, year: &Year) -> Self {
        let combined_code = year.combined_code + codes::get_month(month);
        let num_days = year.days_of_month(month);
        Self {
            month,
            combined_code,
            num_days,
        }
    }

    pub fn month(&self) -> NonZeroU32 {
        self.month
    }

    pub fn day_of_week(&self, day: u32) -> DayOfWeek {
        codes::day_of_week(day, self.combined_code)
    }

    pub fn days(&self) -> impl Iterator<Item = DayOfMonth> {
        let first_day = DayOfMonth {
            day_of_week: self.day_of_week(1),
            day_of_month: NonZeroU32::new(1).unwrap(),
        };
        let num_days = self.num_days;
        std::iter::successors(Some(first_day), move |day| {
            if day.day_of_month.get() + 1 < num_days {
                Some(DayOfMonth {
                    day_of_week: day.day_of_week.next(),
                    day_of_month: NonZeroU32::new(day.day_of_month.get() + 1).unwrap(),
                })
            } else {
                None
            }
        })
    }
}

pub fn non_holidays_of_month(month: &Month, year: &Year) -> Vec<DayOfMonth> {
    let holidays = year.holidays();
    month
        .days()
        .filter(|day| {
            !day.day_of_week.is_weekend()
                && !holidays.contains(&DateOfYear {
                    day: day.day_of_month,
                    month: month.month,
                })
        })
        .collect()
}

#[cfg(test)]
mod test {
    use std::num::NonZeroU32;

    use crate::calendar::DateOfYear;

    #[test]
    fn test_add_days() {
        let first = DateOfYear::new(NonZeroU32::new(1).unwrap(), NonZeroU32::new(1).unwrap());
        assert_eq!(
            first.add_days(1, false),
            DateOfYear::new(NonZeroU32::new(2).unwrap(), NonZeroU32::new(1).unwrap())
        );
        assert_eq!(
            first.add_days(31, false),
            DateOfYear::new(NonZeroU32::new(1).unwrap(), NonZeroU32::new(2).unwrap())
        );
    }
}
