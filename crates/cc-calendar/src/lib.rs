//! Kalendersysteme und Datumsumrechnung.
//!
//! Stand Phase 0: nur die Umrechnung gregorianisch → hebräisch über ICU4X,
//! damit die Spikes einen echten Aufruf in den Kern haben. Die eigentliche
//! Implementierung (Tageszahl, Prüfimplementierung, Jahrestypen) folgt in
//! Phase 1.

use icu_calendar::cal::Hebrew;
use icu_calendar::{Date, Gregorian};
use serde::{Deserialize, Serialize};

/// Ein Datum im hebräischen Kalender.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HebrewDate {
    pub year: i32,
    /// Monatscode nach CLDR, z. B. "M01" für Tischri, "M05L" für Adar I,
    /// "M06" für Adar bzw. Adar II.
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

/// Rechnet ein gregorianisches Datum in das hebräische Datum um, dessen
/// Tageslicht auf diesen Tag fällt (also ohne Berücksichtigung des
/// Sonnenuntergangs).
pub fn gregorian_to_hebrew(year: i32, month: u8, day: u8) -> Result<HebrewDate, DateError> {
    let date = Date::try_new_gregorian(year, month, day)
        .map_err(|e| DateError(format!("ungültiges Datum {year}-{month}-{day}: {e:?}")))?;
    let hebrew = date.to_calendar(Hebrew);
    Ok(HebrewDate {
        year: hebrew.year().extended_year(),
        month_code: hebrew.month().to_input().code().0.to_string(),
        day: hebrew.day_of_month().0,
        leap_year: hebrew.is_in_leap_year(),
    })
}

/// Hilfsfunktion für Tests und Beispiele.
pub fn is_gregorian_leap_year(year: i32) -> bool {
    Date::try_new_gregorian(year, 1, 1)
        .map(|d| d.to_calendar(Gregorian).is_in_leap_year())
        .unwrap_or(false)
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
    }

    #[test]
    fn leap_years() {
        assert!(is_gregorian_leap_year(2024));
        assert!(!is_gregorian_leap_year(2100));
        assert!(is_gregorian_leap_year(2000));
    }
}
