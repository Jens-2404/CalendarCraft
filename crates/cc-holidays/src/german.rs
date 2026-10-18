//! Gesetzliche Feiertage in Deutschland (Arbeitsstand: Hessen).
//!
//! Die vollständige Länderliste folgt (siehe docs/03-kalender-und-feiertage.md,
//! 2.4); hier nur, was der Gemeindekalender für Hessen braucht.

use cc_calendar::{Day, day_from_gregorian};

/// Ostersonntag (westlich, gregorianisch) nach dem Algorithmus von Meeus,
/// Jones und Butcher.
pub fn easter(year: i32) -> Day {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = (h + l - 7 * m + 114) % 31 + 1;
    day_from_gregorian(year, month as u8, day as u8).expect("gültiges Osterdatum")
}

/// Gesetzliche Feiertage Hessens in einem gregorianischen Jahr.
pub fn holidays_hesse(year: i32) -> Vec<(Day, &'static str)> {
    let e = easter(year);
    let fixed = |m, d| day_from_gregorian(year, m, d).expect("gültiges Datum");
    vec![
        (fixed(1, 1), "Neujahr"),
        (e - 2, "Karfreitag"),
        (e + 1, "Ostermontag"),
        (fixed(5, 1), "Tag der Arbeit"),
        (e + 39, "Christi Himmelfahrt"),
        (e + 50, "Pfingstmontag"),
        (e + 60, "Fronleichnam"),
        (fixed(10, 3), "Tag der Deutschen Einheit"),
        (fixed(12, 25), "1. Weihnachtstag"),
        (fixed(12, 26), "2. Weihnachtstag"),
    ]
}

/// Feiertage Hessens im Bereich `from..=to` (Tageszahlen), nach Datum sortiert.
pub fn holidays_hesse_between(from: Day, to: Day) -> Vec<(Day, &'static str)> {
    let (y0, _, _) = cc_calendar::gregorian_from_day(from);
    let (y1, _, _) = cc_calendar::gregorian_from_day(to);
    let mut all: Vec<_> = (y0..=y1)
        .flat_map(holidays_hesse)
        .filter(|(d, _)| (from..=to).contains(d))
        .collect();
    all.sort();
    all
}

#[cfg(test)]
mod tests {
    use super::*;
    use cc_calendar::gregorian_from_day;

    #[test]
    fn easter_dates() {
        for (y, m, d) in [
            (2023, 4, 9),
            (2024, 3, 31),
            (2025, 4, 20),
            (2026, 4, 5),
            (2027, 3, 28),
            (2038, 4, 25),
        ] {
            assert_eq!(gregorian_from_day(easter(y)), (y, m, d), "{y}");
        }
    }

    #[test]
    fn hesse_2024_after_march() {
        // Gegenprobe zum Kalender 5784: Seite 2 nennt andere Daten; die
        // gesetzlichen Feiertage Hessens zwischen 11.3. und 2.10.2024 sind:
        let from = day_from_gregorian(2024, 3, 11).unwrap();
        let to = day_from_gregorian(2024, 10, 2).unwrap();
        let got: Vec<_> = holidays_hesse_between(from, to)
            .into_iter()
            .map(|(d, _)| gregorian_from_day(d))
            .collect();
        assert_eq!(
            got,
            [
                (2024, 3, 29),
                (2024, 4, 1),
                (2024, 5, 1),
                (2024, 5, 9),
                (2024, 5, 20),
                (2024, 5, 30),
            ]
        );
    }
}
