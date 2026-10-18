//! Welche Zeiten an welchem Tag im Kalender stehen.
//!
//! Die Regeln folgen dem Gemeindekalender Frankfurt 5784: Kerzenzünden am
//! Vorabend, Schabbat-/Feiertagsende, Chanukka-Lichter, Fastenzeiten und die
//! Zeiten um Pessach.

use cc_astro::{self as astro, Location, Profile};
use cc_holidays::{Class, DayInfo};

/// Die eine Hauptzeit einer Zeile. Das Linienzeichen im Druck:
/// Linie über der Zeit = Schabbat-/Feiertagsbeginn, Linie darunter = Ende.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainTime {
    /// Kerzenzünden vor Schabbat oder Jom Tov (Linie darüber)
    Candles(i32),
    /// Kerzenzünden in der zweiten Nacht, nach Einbruch der Nacht (Linie darüber)
    SecondNightCandles(i32),
    /// Schabbat- bzw. Feiertagsende (Linie darunter)
    End(i32),
    /// Chanukka-Lichter (Kerzensymbol, ohne Linie)
    ChanukkaCandles(i32),
}

impl MainTime {
    pub fn minutes(&self) -> i32 {
        match *self {
            MainTime::Candles(m)
            | MainTime::SecondNightCandles(m)
            | MainTime::End(m)
            | MainTime::ChanukkaCandles(m) => m,
        }
    }
}

/// Weitere Zeiten einer Zeile.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DayTimes {
    pub main: Option<MainTime>,
    /// Fastenbeginn am Morgen (FB)
    pub fast_begin: Option<i32>,
    /// Fastenende am Abend (FE)
    pub fast_end: Option<i32>,
    /// Bdikat Chametz (Suche), am Abend vor Erew Pessach
    pub bdikat_chametz: Option<i32>,
    /// Ende der Essenszeit für Chametz (Erew Pessach)
    pub chametz_eating_ends: Option<i32>,
    /// Ende der Verbrennungszeit für Chametz (Erew Pessach)
    pub chametz_burning_ends: Option<i32>,
    /// Mitternacht der Sederabend-Nacht (am Erew Pessach)
    pub seder_midnight: Option<i32>,
}

fn has(d: &DayInfo, f: impl Fn(&cc_holidays::Holiday) -> bool) -> bool {
    d.holidays.iter().any(f)
}

fn has_name(d: &DayInfo, name: &str) -> bool {
    has(d, |h| h.name_de == name)
}

fn is_rest_day(d: &DayInfo) -> bool {
    d.is_shabbat() || d.yom_tov
}

fn is_chanukka_light_day(d: &DayInfo) -> bool {
    // Erew Chanukka und 1.–7. Tag: Am Abend wird für die nächste Nacht gezündet
    has_name(d, "Erew Chanukka")
        || has(d, |h| {
            h.class == Class::Chanukka && !h.name_de.starts_with("8.")
        })
}

/// Zeiten für Tag `i` einer Tagesliste (Nachbartage werden mitbenutzt).
pub fn day_times(days: &[DayInfo], i: usize, loc: &Location, p: &Profile) -> DayTimes {
    let d = &days[i];
    let next = days.get(i + 1);
    let mut t = DayTimes::default();

    // Am Listenende (Erew Rosch HaSchana) folgt Rosch HaSchana selbst.
    let tomorrow_rest = match next {
        Some(n) => is_rest_day(n),
        None => has_name(d, "Erew Rosch Haschana"),
    };
    let friday = d.weekday == cc_calendar::FRIDAY;

    // Hauptzeit
    t.main = if is_chanukka_light_day(d) {
        if friday {
            astro::candle_lighting(loc, p, d.day).map(MainTime::Candles)
        } else if d.is_shabbat() {
            astro::shabbat_end(loc, p, d.day).map(MainTime::ChanukkaCandles)
        } else {
            astro::chanukka_candles(loc, p, d.day).map(MainTime::ChanukkaCandles)
        }
    } else if !is_rest_day(d) && (friday || tomorrow_rest) {
        astro::candle_lighting(loc, p, d.day).map(MainTime::Candles)
    } else if is_rest_day(d) && tomorrow_rest {
        astro::shabbat_end(loc, p, d.day).map(MainTime::SecondNightCandles)
    } else if is_rest_day(d) {
        astro::shabbat_end(loc, p, d.day).map(MainTime::End)
    } else {
        None
    };

    // Fasttage
    let fast = has(d, |h| h.class == Class::Fast);
    if fast {
        t.fast_end = astro::fast_end(loc, p, d.day);
        // Tischa beAw beginnt am Vorabend bei Sonnenuntergang (Zeile des Vortags)
        if !has_name(d, "Fasten 9. Aw") {
            t.fast_begin = astro::fast_begin(loc, p, d.day);
        }
    }
    if next.is_some_and(|n| has_name(n, "Fasten 9. Aw")) && !d.is_shabbat() {
        t.fast_begin = astro::fast_begin_at_sunset(loc, p, d.day);
    }

    // Pessach
    if has_name(d, "Erew Pessach") {
        t.chametz_eating_ends = astro::chametz_eating_ends(loc, p, d.day);
        t.chametz_burning_ends = astro::chametz_burning_ends(loc, p, d.day);
        t.seder_midnight = astro::midnight_after(loc, d.day);
    }
    // Bdikat Chametz: Abend vor Erew Pessach; fällt Erew Pessach auf Schabbat,
    // am Donnerstag davor.
    if let Some(n) = next
        && has_name(n, "Erew Pessach")
        && !n.is_shabbat()
    {
        t.bdikat_chametz = astro::bdikat_chametz(loc, p, d.day);
    }
    if let Some(n2) = days.get(i + 2)
        && n2.is_shabbat()
        && has_name(n2, "Erew Pessach")
        && d.weekday == cc_calendar::THURSDAY
    {
        t.bdikat_chametz = astro::bdikat_chametz(loc, p, d.day);
    }
    t
}

/// Minuten seit Mitternacht als "h.mm", wie im Gemeindekalender.
pub fn fmt(m: i32) -> String {
    format!("{}.{:02}", m.div_euclid(60), m.rem_euclid(60))
}
