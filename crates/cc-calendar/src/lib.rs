//! Kalendersysteme und Datumsumrechnung.
//!
//! Stand Phase 0/1: Tageszahl (Rata Die), gregorianisch, hebräisch über
//! ICU4X. Die unabhängige Prüfimplementierung nach *Calendrical
//! Calculations* (ADR-003) folgt in Phase 1.

use icu_calendar::cal::Hebrew;
use icu_calendar::types::RataDie;
use icu_calendar::{Date, Gregorian};
use serde::{Deserialize, Serialize};

/// Fortlaufende Tageszahl (Rata Die): 1 = 1. Januar 0001 (gregorianisch,
/// proleptisch). Tag 0 war ein Sonntag.
pub type Day = i64;

/// Wochentag, 0 = Sonntag … 6 = Schabbat.
pub type Weekday = u8;

pub const SUNDAY: Weekday = 0;
pub const MONDAY: Weekday = 1;
pub const TUESDAY: Weekday = 2;
pub const WEDNESDAY: Weekday = 3;
pub const THURSDAY: Weekday = 4;
pub const FRIDAY: Weekday = 5;
pub const SATURDAY: Weekday = 6;

/// Ein Datum im hebräischen Kalender.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HebrewDate {
    pub year: i32,
    /// Monatscode nach CLDR, z. B. "M01" für Tischri, "M05L" für Adar I,
    /// "M06" für Adar bzw. Adar II, "M07" für Nisan, "M12" für Elul.
    pub month_code: String,
    pub day: u8,
    /// Schaltjahr mit Adar I und Adar II. Nötig für den Monatsnamen:
    /// "M06" heißt im Schaltjahr Adar II, sonst Adar.
    pub leap_year: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateError(pub String);

impl std::fmt::Display for DateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DateError {}

fn is_gregorian_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_gregorian_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if is_gregorian_leap(year) => 29,
        _ => 28,
    }
}

/// Tageszahl eines gregorianischen Datums.
pub fn day_from_gregorian(year: i32, month: u8, day: u8) -> Result<Day, DateError> {
    if !(1..=12).contains(&month) || day == 0 || day > days_in_gregorian_month(year, month) {
        return Err(DateError(format!("ungültiges Datum {year}-{month}-{day}")));
    }
    let y = i64::from(year) - 1;
    let m = i64::from(month);
    let correction = match m {
        1 | 2 => 0,
        _ if is_gregorian_leap(year) => -1,
        _ => -2,
    };
    Ok(365 * y + y.div_euclid(4) - y.div_euclid(100)
        + y.div_euclid(400)
        + (367 * m - 362).div_euclid(12)
        + correction
        + i64::from(day))
}

/// Gregorianisches Datum (Jahr, Monat, Tag) zur Tageszahl.
pub fn gregorian_from_day(day: Day) -> (i32, u8, u8) {
    let date = Date::from_rata_die(RataDie::new(day), Gregorian);
    (
        date.year().extended_year(),
        date.month().ordinal,
        date.day_of_month().0,
    )
}

/// Wochentag zur Tageszahl, 0 = Sonntag.
pub fn weekday(day: Day) -> Weekday {
    day.rem_euclid(7) as Weekday
}

/// Hebräisches Datum zur Tageszahl (Tageslicht dieses Tages, ohne
/// Berücksichtigung des Sonnenuntergangs).
pub fn hebrew_from_day(day: Day) -> HebrewDate {
    let hebrew = Date::from_rata_die(RataDie::new(day), Hebrew);
    HebrewDate {
        year: hebrew.year().extended_year(),
        month_code: hebrew.month().to_input().code().0.to_string(),
        day: hebrew.day_of_month().0,
        leap_year: hebrew.is_in_leap_year(),
    }
}

/// Rechnet ein gregorianisches Datum in das hebräische Datum um.
pub fn gregorian_to_hebrew(year: i32, month: u8, day: u8) -> Result<HebrewDate, DateError> {
    Ok(hebrew_from_day(day_from_gregorian(year, month, day)?))
}

/// Ist das hebräische Jahr ein Schaltjahr (mit Adar I und Adar II)?
pub fn is_hebrew_leap_year(year: i32) -> bool {
    (7 * i64::from(year) + 1).rem_euclid(19) < 7
}

/// Tageszahl des 1. Tischri (Rosch HaSchana) des hebräischen Jahres.
pub fn rosh_hashana(year: i32) -> Day {
    // Rosch HaSchana liegt zwischen dem 5. September und dem 5. Oktober des
    // gregorianischen Jahres year - 3761.
    let greg_year = year - 3761;
    let start = day_from_gregorian(greg_year, 9, 1).expect("gültiges Datum");
    (start..start + 40)
        .find(|&d| {
            let h = hebrew_from_day(d);
            h.year == year && h.month_code == "M01" && h.day == 1
        })
        .expect("Rosch HaSchana liegt im Zeitfenster")
}

/// Anzahl der Tage des hebräischen Jahres (353–355 bzw. 383–385).
pub fn hebrew_year_length(year: i32) -> i64 {
    rosh_hashana(year + 1) - rosh_hashana(year)
}

/// Alle Tageszahlen des hebräischen Jahres von 1. Tischri bis 29. Elul.
pub fn hebrew_year_days(year: i32) -> std::ops::Range<Day> {
    rosh_hashana(year)..rosh_hashana(year + 1)
}

/// Hilfsfunktion für Tests und Beispiele.
pub fn is_gregorian_leap_year(year: i32) -> bool {
    is_gregorian_leap(year)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rosch_haschana_5787() {
        // Rosch HaSchana 5787 = 12. September 2026
        let d = gregorian_to_hebrew(2026, 9, 12).unwrap();
        assert_eq!(
            d,
            HebrewDate {
                year: 5787,
                month_code: "M01".into(),
                day: 1,
                leap_year: true
            }
        );
    }

    #[test]
    fn purim_5787_in_adar_ii() {
        // 5787 ist Schaltjahr (Jahr 11 im Zyklus); Purim = 14. Adar II = 23. März 2027
        let d = gregorian_to_hebrew(2027, 3, 23).unwrap();
        assert_eq!(
            (d.month_code.as_str(), d.day, d.leap_year),
            ("M06", 14, true)
        );
        // Purim Katan = 14. Adar I = 21. Februar 2027
        let d = gregorian_to_hebrew(2027, 2, 21).unwrap();
        assert_eq!((d.month_code.as_str(), d.day), ("M05L", 14));
    }

    #[test]
    fn adar_in_common_year() {
        // 5786 ist Gemeinjahr; Purim = 14. Adar = 3. März 2026
        let d = gregorian_to_hebrew(2026, 3, 3).unwrap();
        assert_eq!(
            (d.month_code.as_str(), d.day, d.leap_year),
            ("M06", 14, false)
        );
    }

    /// Daten aus dem Gemeindekalender Frankfurt 5784 (Kalender_5784.pdf).
    /// Die Gemeinde gibt gregorianische Tage mit hebräischem Monat an;
    /// hier die Stichproben, die im Text des PDFs eindeutig stehen.
    #[test]
    fn frankfurt_kalender_5784() {
        let cases = [
            // (gregorianisch, erwartet: Tag, Monatscode, Jahr, Bezeichnung)
            ((2023, 9, 15), (29, "M12", 5783)), // Elul-Ende vor Rosch HaSchana
            ((2023, 9, 16), (1, "M01", 5784)),  // 1. Tag Rosch HaSchana
            ((2023, 9, 25), (10, "M01", 5784)), // Jom Kippur
            ((2023, 10, 1), (16, "M01", 5784)), // Tag nach Sukkot-Anfang
            ((2024, 3, 24), (14, "M06", 5784)), // Purim (Adar II im Schaltjahr)
            ((2024, 4, 23), (15, "M07", 5784)), // 1. Tag Pessach
        ];
        for ((y, m, d), (day, month, year)) in cases {
            let h = gregorian_to_hebrew(y, m, d).unwrap();
            assert_eq!(
                (h.day, h.month_code.as_str(), h.year),
                (day, month, year),
                "{y}-{m:02}-{d:02}"
            );
        }
    }

    #[test]
    fn pessach_5786() {
        // 15. Nisan 5786 = 2. April 2026
        let d = gregorian_to_hebrew(2026, 4, 2).unwrap();
        assert_eq!(d.year, 5786);
        assert_eq!(d.day, 15);
    }

    #[test]
    fn invalid_date() {
        assert!(gregorian_to_hebrew(2026, 2, 30).is_err());
        assert!(gregorian_to_hebrew(2026, 13, 1).is_err());
    }

    #[test]
    fn leap_years() {
        assert!(is_gregorian_leap_year(2024));
        assert!(!is_gregorian_leap_year(2100));
        assert!(is_gregorian_leap_year(2000));
    }

    #[test]
    fn day_numbers_roundtrip() {
        assert_eq!(day_from_gregorian(1, 1, 1).unwrap(), 1);
        for (y, m, d) in [(2026, 9, 12), (2000, 2, 29), (1900, 3, 1), (2100, 12, 31)] {
            let n = day_from_gregorian(y, m, d).unwrap();
            assert_eq!(gregorian_from_day(n), (y, m, d));
        }
    }

    #[test]
    fn weekdays() {
        let sat = day_from_gregorian(2026, 9, 12).unwrap();
        assert_eq!(weekday(sat), SATURDAY); // 12.9.2026 war ein Samstag
        assert_eq!(weekday(day_from_gregorian(2023, 9, 15).unwrap()), FRIDAY);
        assert_eq!(weekday(day_from_gregorian(2000, 1, 1).unwrap()), SATURDAY);
    }

    #[test]
    fn rosch_haschana_days() {
        assert_eq!(gregorian_from_day(rosh_hashana(5784)), (2023, 9, 16));
        assert_eq!(gregorian_from_day(rosh_hashana(5787)), (2026, 9, 12));
        assert_eq!(gregorian_from_day(rosh_hashana(5788)), (2027, 10, 2));
    }

    #[test]
    fn year_lengths() {
        assert_eq!(hebrew_year_length(5784), 383); // Schaltjahr, mangelhaft
        assert_eq!(hebrew_year_length(5787), 385); // Schaltjahr, vollständig
        for y in 5700..5900 {
            let len = hebrew_year_length(y);
            assert!([353, 354, 355, 383, 384, 385].contains(&len), "{y}: {len}");
            assert_eq!(len > 360, is_hebrew_leap_year(y), "{y}");
            // Rosch HaSchana nie Sonntag, Mittwoch, Freitag
            assert!(
                ![SUNDAY, WEDNESDAY, FRIDAY].contains(&weekday(rosh_hashana(y))),
                "{y}"
            );
        }
    }
}
