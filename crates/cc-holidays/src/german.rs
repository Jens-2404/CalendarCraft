//! Gesetzliche Feiertage in Deutschland je Bundesland.
//!
//! Berücksichtigt sind landesweite Feiertage. Feiertage, die nur in Teilen
//! eines Landes gelten (Mariä Himmelfahrt in Bayern, Fronleichnam in Teilen
//! Sachsens und Thüringens, Augsburger Friedensfest), fehlen bewusst.

use cc_calendar::{Day, WEDNESDAY, day_from_gregorian, weekday};

/// Bundesland.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    BadenWuerttemberg,
    Bayern,
    Berlin,
    Brandenburg,
    Bremen,
    Hamburg,
    Hessen,
    MecklenburgVorpommern,
    Niedersachsen,
    NordrheinWestfalen,
    RheinlandPfalz,
    Saarland,
    Sachsen,
    SachsenAnhalt,
    SchleswigHolstein,
    Thueringen,
}

use State::*;

impl State {
    pub const ALL: [State; 16] = [
        BadenWuerttemberg,
        Bayern,
        Berlin,
        Brandenburg,
        Bremen,
        Hamburg,
        Hessen,
        MecklenburgVorpommern,
        Niedersachsen,
        NordrheinWestfalen,
        RheinlandPfalz,
        Saarland,
        Sachsen,
        SachsenAnhalt,
        SchleswigHolstein,
        Thueringen,
    ];

    /// Kürzel nach ISO 3166-2:DE ohne Ländercode, z. B. "HE".
    pub fn code(self) -> &'static str {
        match self {
            BadenWuerttemberg => "BW",
            Bayern => "BY",
            Berlin => "BE",
            Brandenburg => "BB",
            Bremen => "HB",
            Hamburg => "HH",
            Hessen => "HE",
            MecklenburgVorpommern => "MV",
            Niedersachsen => "NI",
            NordrheinWestfalen => "NW",
            RheinlandPfalz => "RP",
            Saarland => "SL",
            Sachsen => "SN",
            SachsenAnhalt => "ST",
            SchleswigHolstein => "SH",
            Thueringen => "TH",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            BadenWuerttemberg => "Baden-Württemberg",
            Bayern => "Bayern",
            Berlin => "Berlin",
            Brandenburg => "Brandenburg",
            Bremen => "Bremen",
            Hamburg => "Hamburg",
            Hessen => "Hessen",
            MecklenburgVorpommern => "Mecklenburg-Vorpommern",
            Niedersachsen => "Niedersachsen",
            NordrheinWestfalen => "Nordrhein-Westfalen",
            RheinlandPfalz => "Rheinland-Pfalz",
            Saarland => "Saarland",
            Sachsen => "Sachsen",
            SachsenAnhalt => "Sachsen-Anhalt",
            SchleswigHolstein => "Schleswig-Holstein",
            Thueringen => "Thüringen",
        }
    }

    pub fn from_code(code: &str) -> Option<State> {
        Self::ALL
            .into_iter()
            .find(|s| s.code().eq_ignore_ascii_case(code))
    }
}

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

/// Gesetzliche Feiertage eines Bundeslandes in einem gregorianischen Jahr,
/// nach Datum sortiert.
pub fn holidays(year: i32, state: State) -> Vec<(Day, &'static str)> {
    let e = easter(year);
    let fixed = |m, d| day_from_gregorian(year, m, d).expect("gültiges Datum");
    let is = |list: &[State]| list.contains(&state);
    let mut v = vec![(fixed(1, 1), "Neujahr")];
    if is(&[BadenWuerttemberg, Bayern, SachsenAnhalt]) {
        v.push((fixed(1, 6), "Heilige Drei Könige"));
    }
    if (state == Berlin && year >= 2019) || (state == MecklenburgVorpommern && year >= 2023) {
        v.push((fixed(3, 8), "Internationaler Frauentag"));
    }
    v.push((e - 2, "Karfreitag"));
    if state == Brandenburg {
        v.push((e, "Ostersonntag"));
    }
    v.push((e + 1, "Ostermontag"));
    v.push((fixed(5, 1), "Tag der Arbeit"));
    if state == Berlin && (year == 2020 || year == 2025) {
        v.push((fixed(5, 8), "Tag der Befreiung"));
    }
    v.push((e + 39, "Christi Himmelfahrt"));
    if state == Brandenburg {
        v.push((e + 49, "Pfingstsonntag"));
    }
    v.push((e + 50, "Pfingstmontag"));
    if is(&[
        BadenWuerttemberg,
        Bayern,
        Hessen,
        NordrheinWestfalen,
        RheinlandPfalz,
        Saarland,
    ]) {
        v.push((e + 60, "Fronleichnam"));
    }
    if state == Saarland {
        v.push((fixed(8, 15), "Mariä Himmelfahrt"));
    }
    if state == Thueringen && year >= 2019 {
        v.push((fixed(9, 20), "Weltkindertag"));
    }
    v.push((fixed(10, 3), "Tag der Deutschen Einheit"));
    // 2017 (500 Jahre Reformation) bundesweit
    let reformation = year == 2017
        || is(&[
            Brandenburg,
            MecklenburgVorpommern,
            Sachsen,
            SachsenAnhalt,
            Thueringen,
        ])
        || (year >= 2018 && is(&[Bremen, Hamburg, Niedersachsen, SchleswigHolstein]));
    if reformation {
        v.push((fixed(10, 31), "Reformationstag"));
    }
    if is(&[
        BadenWuerttemberg,
        Bayern,
        NordrheinWestfalen,
        RheinlandPfalz,
        Saarland,
    ]) {
        v.push((fixed(11, 1), "Allerheiligen"));
    }
    if state == Sachsen {
        // Mittwoch vor dem 23. November
        let d22 = fixed(11, 22);
        let back = (i64::from(weekday(d22)) - i64::from(WEDNESDAY)).rem_euclid(7);
        v.push((d22 - back, "Buß- und Bettag"));
    }
    v.push((fixed(12, 25), "1. Weihnachtstag"));
    v.push((fixed(12, 26), "2. Weihnachtstag"));
    v.sort();
    v
}

/// Feiertage eines Bundeslandes im Bereich `from..=to` (Tageszahlen), nach
/// Datum sortiert.
pub fn holidays_between(from: Day, to: Day, state: State) -> Vec<(Day, &'static str)> {
    let (y0, _, _) = cc_calendar::gregorian_from_day(from);
    let (y1, _, _) = cc_calendar::gregorian_from_day(to);
    let mut all: Vec<_> = (y0..=y1)
        .flat_map(|y| holidays(y, state))
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
        let got: Vec<_> = holidays_between(from, to, Hessen)
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

    fn dates(year: i32, state: State) -> Vec<(u8, u8)> {
        holidays(year, state)
            .into_iter()
            .map(|(d, _)| {
                let (_, m, t) = gregorian_from_day(d);
                (m, t)
            })
            .collect()
    }

    #[test]
    fn states_2026() {
        assert_eq!(dates(2026, Hessen).len(), 10);
        assert_eq!(dates(2026, Bayern).len(), 12);
        assert_eq!(dates(2026, Berlin).len(), 10);
        assert_eq!(dates(2026, Brandenburg).len(), 12);
        assert_eq!(dates(2026, Hamburg).len(), 10);
        assert_eq!(dates(2026, Saarland).len(), 12);
        assert_eq!(dates(2026, Thueringen).len(), 11);
        assert_eq!(dates(2026, Sachsen).len(), 11);
        assert!(dates(2026, Sachsen).contains(&(11, 18)));
        assert!(dates(2027, Sachsen).contains(&(11, 17)));
        assert!(dates(2026, Berlin).contains(&(3, 8)));
        assert!(!dates(2026, Hessen).contains(&(10, 31)));
        assert!(dates(2017, Hessen).contains(&(10, 31)));
    }

    #[test]
    fn codes_round_trip() {
        for s in State::ALL {
            assert_eq!(State::from_code(s.code()), Some(s));
        }
        assert_eq!(State::from_code("he"), Some(Hessen));
    }
}
