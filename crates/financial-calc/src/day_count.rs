//! Day Count Conventions for financial calculations.
//!
//! Day count conventions determine how interest accrues over time by defining
//! how to calculate the fraction of a year between two dates.

use precision_core::{ArithmeticError, Decimal};

/// Day count convention variants used in fixed income markets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayCountConvention {
    /// Actual/360: Actual days divided by 360.
    /// Common for money market instruments, LIBOR-based products.
    Actual360,
    /// Actual/365: Actual days divided by 365 (ignores leap years).
    /// Common for GBP instruments.
    Actual365Fixed,
    /// Actual/Actual: Actual days in period divided by actual days in year.
    /// Used for government bonds.
    ActualActual,
    /// 30/360: Assumes 30 days per month, 360 days per year.
    /// Common for corporate bonds and swaps.
    Thirty360,
    /// 30E/360: European 30/360 variant.
    Thirty360E,
}

/// Represents a simple date as days since epoch (for no_std compatibility).
/// In production, this would integrate with a proper date library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    /// Year (e.g., 2024)
    pub year: i32,
    /// Month (1-12)
    pub month: u8,
    /// Day of month (1-31)
    pub day: u8,
}

impl Date {
    /// Creates a new date.
    pub const fn new(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    /// Returns true if the year is a leap year.
    pub fn is_leap_year(&self) -> bool {
        (self.year % 4 == 0 && self.year % 100 != 0) || (self.year % 400 == 0)
    }

    /// Days in the year containing this date.
    pub fn days_in_year(&self) -> u32 {
        if self.is_leap_year() {
            366
        } else {
            365
        }
    }

    /// Days in the given month for this date's year.
    pub fn days_in_month(&self) -> u8 {
        match self.month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if self.is_leap_year() {
                    29
                } else {
                    28
                }
            }
            _ => 30, // fallback
        }
    }

    /// Converts date to a day number (simple calculation for date arithmetic).
    /// Uses a simplified algorithm suitable for no_std.
    pub fn to_day_number(&self) -> i64 {
        let y = self.year as i64;
        let m = self.month as i64;
        let d = self.day as i64;

        // Adjust for months
        let (y_adj, m_adj) = if m <= 2 { (y - 1, m + 12) } else { (y, m) };

        // Julian day number approximation
        let jdn = d + (153 * (m_adj - 3) + 2) / 5 + 365 * y_adj + y_adj / 4 - y_adj / 100
            + y_adj / 400
            - 32045;

        jdn
    }

    /// Calculates actual days between two dates.
    pub fn days_between(&self, other: &Date) -> i64 {
        other.to_day_number() - self.to_day_number()
    }
}

impl DayCountConvention {
    /// Calculates the year fraction between two dates using this convention.
    ///
    /// Returns a `Decimal` representing the fraction of a year.
    pub fn year_fraction(&self, start: Date, end: Date) -> Result<Decimal, ArithmeticError> {
        match self {
            DayCountConvention::Actual360 => {
                let days = start.days_between(&end);
                Decimal::from(days).try_div(Decimal::from(360i64))
            }
            DayCountConvention::Actual365Fixed => {
                let days = start.days_between(&end);
                Decimal::from(days).try_div(Decimal::from(365i64))
            }
            DayCountConvention::ActualActual => {
                let days = start.days_between(&end);
                // Use average of start and end year's day count
                let avg_days = (start.days_in_year() + end.days_in_year()) / 2;
                Decimal::from(days).try_div(Decimal::from(avg_days as i64))
            }
            DayCountConvention::Thirty360 => {
                let day_fraction = thirty_360_days(start, end, false);
                day_fraction.try_div(Decimal::from(360i64))
            }
            DayCountConvention::Thirty360E => {
                let day_fraction = thirty_360_days(start, end, true);
                day_fraction.try_div(Decimal::from(360i64))
            }
        }
    }
}

/// Calculates 30/360 day count.
fn thirty_360_days(start: Date, end: Date, european: bool) -> Decimal {
    let mut d1 = start.day as i64;
    let mut d2 = end.day as i64;
    let m1 = start.month as i64;
    let m2 = end.month as i64;
    let y1 = start.year as i64;
    let y2 = end.year as i64;

    if european {
        // 30E/360: If day is 31, change to 30
        if d1 == 31 {
            d1 = 30;
        }
        if d2 == 31 {
            d2 = 30;
        }
    } else {
        // US 30/360
        if d1 == 31 {
            d1 = 30;
        }
        if d2 == 31 && d1 >= 30 {
            d2 = 30;
        }
    }

    let days = 360 * (y2 - y1) + 30 * (m2 - m1) + (d2 - d1);
    Decimal::from(days)
}

/// A year fraction represented as a Decimal.
/// This is the fundamental unit for time in term structure calculations.
pub type YearFraction = Decimal;

/// Helper function to create a year fraction directly from a decimal number.
pub fn year_fraction_from_decimal(years: Decimal) -> YearFraction {
    years
}

/// Helper function to create a year fraction from days and convention.
pub fn year_fraction_from_days(
    days: i64,
    convention: DayCountConvention,
) -> Result<YearFraction, ArithmeticError> {
    let divisor = match convention {
        DayCountConvention::Actual360 | DayCountConvention::Thirty360 | DayCountConvention::Thirty360E => 360i64,
        DayCountConvention::Actual365Fixed => 365i64,
        DayCountConvention::ActualActual => 365i64, // approximation
    };
    Decimal::from(days).try_div(Decimal::from(divisor))
}

#[cfg(test)]
mod tests {
    use super::*;
    use precision_core::RoundingMode;

    #[test]
    fn test_date_days_between() {
        let d1 = Date::new(2024, 1, 1);
        let d2 = Date::new(2024, 12, 31);
        // 2024 is a leap year, so 366 days - 1 = 365
        assert_eq!(d1.days_between(&d2), 365);
    }

    #[test]
    fn test_leap_year() {
        assert!(Date::new(2024, 1, 1).is_leap_year());
        assert!(!Date::new(2023, 1, 1).is_leap_year());
        assert!(!Date::new(2100, 1, 1).is_leap_year());
        assert!(Date::new(2000, 1, 1).is_leap_year());
    }

    #[test]
    fn test_actual_360() {
        let start = Date::new(2024, 1, 1);
        let end = Date::new(2024, 7, 1);
        let days = start.days_between(&end); // 182 days

        let fraction = DayCountConvention::Actual360
            .year_fraction(start, end)
            .unwrap();
        let expected = Decimal::from(days).try_div(Decimal::from(360i64)).unwrap();
        assert_eq!(fraction, expected);
    }

    #[test]
    fn test_thirty_360() {
        let start = Date::new(2024, 1, 15);
        let end = Date::new(2024, 4, 15);
        // 3 months = 90 days in 30/360

        let fraction = DayCountConvention::Thirty360
            .year_fraction(start, end)
            .unwrap();
        let rounded = fraction.round(4, RoundingMode::HalfEven);
        // 90/360 = 0.25
        assert_eq!(rounded, Decimal::new(25, 2));
    }

    #[test]
    fn test_thirty_360_end_of_month() {
        let start = Date::new(2024, 1, 31);
        let end = Date::new(2024, 3, 31);
        // 30/360: Jan 31 -> Jan 30, Mar 31 -> Mar 30
        // = 360*(0) + 30*(2) + (30-30) = 60 days

        let fraction = DayCountConvention::Thirty360
            .year_fraction(start, end)
            .unwrap();
        let rounded = fraction.round(6, RoundingMode::HalfEven);
        // 60/360 = 0.166667
        let expected = Decimal::from(60i64)
            .try_div(Decimal::from(360i64))
            .unwrap()
            .round(6, RoundingMode::HalfEven);
        assert_eq!(rounded, expected);
    }
}
