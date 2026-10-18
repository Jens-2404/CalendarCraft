//! Sonnenstand, Sonnenauf- und -untergang und daraus abgeleitete Zeiten.
//!
//! Grundlage sind die Formeln des NOAA Solar Calculator (nach Meeus,
//! *Astronomical Algorithms*). Zeiten sind Minuten seit Mitternacht der
//! Ortszeit; `None` heißt: Die Sonne erreicht den Winkel an diesem Tag nicht.
//!
//! Stand Prototyp: Meereshöhe, Sommerzeit nach EU-Regel ab 1980.

use cc_calendar::{Day, SUNDAY, gregorian_from_day, weekday};

/// Zeitzone mit einfacher Sommerzeitregel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeZone {
    /// Feste Abweichung von UTC in Minuten
    Fixed(i32),
    /// Mitteleuropa-artig: Normalzeit-Abweichung in Minuten; Sommerzeit +60
    /// von letztem Sonntag im März 01:00 UTC bis letztem Sonntag im Oktober
    /// 01:00 UTC (ab 1996; 1980–1995 bis letzter Sonntag im September).
    Eu { standard_offset: i32 },
}

impl TimeZone {
    /// Frankfurt, Berlin, Wien, Zürich …
    pub const CET: TimeZone = TimeZone::Eu {
        standard_offset: 60,
    };

    /// UTC-Abweichung in Minuten für einen Zeitpunkt (Tag, Minuten UTC).
    pub fn offset_minutes(&self, day: Day, utc_minutes: f64) -> i32 {
        match *self {
            TimeZone::Fixed(m) => m,
            TimeZone::Eu { standard_offset } => {
                let (year, _, _) = gregorian_from_day(day);
                if year < 1980 {
                    return standard_offset;
                }
                let last_sunday = |month: u8| {
                    let last = cc_calendar::day_from_gregorian(year, month, 31)
                        .or_else(|_| cc_calendar::day_from_gregorian(year, month, 30))
                        .expect("gültiger Monat");
                    last - i64::from(weekday(last) - SUNDAY)
                };
                let start = last_sunday(3);
                let end = last_sunday(if year >= 1996 { 10 } else { 9 });
                let t = day as f64 * 1440.0 + utc_minutes;
                if t >= start as f64 * 1440.0 + 60.0 && t < end as f64 * 1440.0 + 60.0 {
                    standard_offset + 60
                } else {
                    standard_offset
                }
            }
        }
    }
}

/// Ort für die Zeitberechnung.
#[derive(Debug, Clone)]
pub struct Location {
    pub name: String,
    /// Breite in Grad, Nord positiv
    pub latitude: f64,
    /// Länge in Grad, Ost positiv
    pub longitude: f64,
    pub time_zone: TimeZone,
}

impl Location {
    pub fn frankfurt() -> Self {
        Self {
            name: "Frankfurt am Main".into(),
            latitude: 50.11552,
            longitude: 8.68417,
            time_zone: TimeZone::CET,
        }
    }
}

/// Zenitwinkel für Sonnenaufgang/-untergang (Oberrand der Sonne, mit
/// Refraktion): 90° 50′.
pub const ZENITH_SUNRISE: f64 = 90.0 + 50.0 / 60.0;

fn rad(d: f64) -> f64 {
    d.to_radians()
}

/// Julianischer Tag um 0 Uhr UTC des Tages.
fn julian_day(day: Day) -> f64 {
    // Tag 1 (1.1.0001, proleptisch gregorianisch) = JD 1721425.5 um 0 Uhr
    day as f64 + 1721424.5
}

struct Sun {
    declination: f64, // Radiant
    eq_time: f64,     // Minuten
}

fn sun(jd: f64) -> Sun {
    let t = (jd - 2451545.0) / 36525.0;
    let l0 = (280.46646 + t * (36000.76983 + t * 0.0003032)).rem_euclid(360.0);
    let m = 357.52911 + t * (35999.05029 - 0.0001537 * t);
    let e = 0.016708634 - t * (0.000042037 + 0.0000001267 * t);
    let mr = rad(m);
    let c = mr.sin() * (1.914602 - t * (0.004817 + 0.000014 * t))
        + (2.0 * mr).sin() * (0.019993 - 0.000101 * t)
        + (3.0 * mr).sin() * 0.000289;
    let true_long = l0 + c;
    let omega = 125.04 - 1934.136 * t;
    let lambda = true_long - 0.00569 - 0.00478 * rad(omega).sin();
    let seconds = 21.448 - t * (46.8150 + t * (0.00059 - t * 0.001813));
    let e0 = 23.0 + (26.0 + seconds / 60.0) / 60.0;
    let epsilon = e0 + 0.00256 * rad(omega).cos();
    let declination = (rad(epsilon).sin() * rad(lambda).sin()).asin();
    let y = (rad(epsilon) / 2.0).tan().powi(2);
    let l0r = rad(l0);
    let eq_time = 4.0
        * (y * (2.0 * l0r).sin() - 2.0 * e * mr.sin() + 4.0 * e * y * mr.sin() * (2.0 * l0r).cos()
            - 0.5 * y * y * (4.0 * l0r).sin()
            - 1.25 * e * e * (2.0 * mr).sin())
        .to_degrees();
    Sun {
        declination,
        eq_time,
    }
}

/// Minuten UTC (seit 0 Uhr UTC des Tages) für den Durchgang der Sonnenmitte
/// durch den Zenitwinkel; `rising` = aufsteigend (Morgen).
fn event_utc(day: Day, lat: f64, lon: f64, zenith: f64, rising: bool) -> Option<f64> {
    let mut jd = julian_day(day);
    let mut result = None;
    for _ in 0..2 {
        let s = sun(jd);
        let cos_h = rad(zenith).cos() / (rad(lat).cos() * s.declination.cos())
            - rad(lat).tan() * s.declination.tan();
        if !(-1.0..=1.0).contains(&cos_h) {
            return None;
        }
        let mut h = cos_h.acos().to_degrees();
        if !rising {
            h = -h;
        }
        let minutes = 720.0 - 4.0 * (lon + h) - s.eq_time;
        result = Some(minutes);
        jd = julian_day(day) + minutes / 1440.0;
    }
    result
}

/// Ortszeit in Minuten seit Mitternacht des Tages.
fn local(loc: &Location, day: Day, utc_minutes: f64) -> f64 {
    utc_minutes + f64::from(loc.time_zone.offset_minutes(day, utc_minutes))
}

/// Zeitpunkt, an dem die Sonnenmitte morgens (`rising`) oder abends den
/// Zenitwinkel erreicht. Ortszeit, Minuten seit Mitternacht.
pub fn sun_at_zenith(loc: &Location, day: Day, zenith: f64, rising: bool) -> Option<f64> {
    event_utc(day, loc.latitude, loc.longitude, zenith, rising).map(|u| local(loc, day, u))
}

pub fn sunrise(loc: &Location, day: Day) -> Option<f64> {
    sun_at_zenith(loc, day, ZENITH_SUNRISE, true)
}

pub fn sunset(loc: &Location, day: Day) -> Option<f64> {
    sun_at_zenith(loc, day, ZENITH_SUNRISE, false)
}

/// Morgendämmerung (Alot HaSchachar), Sonne `degrees` unter dem Horizont.
pub fn dawn_degrees(loc: &Location, day: Day, degrees: f64) -> Option<f64> {
    sun_at_zenith(loc, day, 90.0 + degrees, true)
}

/// Einbruch der Nacht (Tzet HaKochawim), Sonne `degrees` unter dem Horizont.
pub fn nightfall_degrees(loc: &Location, day: Day, degrees: f64) -> Option<f64> {
    sun_at_zenith(loc, day, 90.0 + degrees, false)
}

/// Parameter, nach denen Zeiten für einen Ort berechnet werden. Die
/// Voreinstellung `frankfurt()` reproduziert die Zeiten im Gemeindekalender
/// Frankfurt 5784 (an 60 Tagen auf die Minute, Rest ±1 Minute).
#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    /// Kerzenzünden: Minuten vor Sonnenuntergang
    pub candle_minutes: f64,
    /// Schabbat- und Feiertagsende: Sonnenwinkel unter dem Horizont
    pub shabbat_end_degrees: f64,
    /// Fastenbeginn (Alot HaSchachar)
    pub fast_begin_degrees: f64,
    /// Fastenende, Chanukka-Kerzen an Wochentagen, Bdikat Chametz
    pub fast_end_degrees: f64,
    /// Winkel für den Tag der Chametz-Zeiten (Anfang und Ende gleich)
    pub chametz_degrees: f64,
}

impl Profile {
    /// Voreinstellung der Jüdischen Gemeinde Frankfurt.
    pub fn frankfurt() -> Self {
        Self {
            candle_minutes: 30.0,
            shabbat_end_degrees: 8.5,
            fast_begin_degrees: 16.1,
            fast_end_degrees: 7.083,
            chametz_degrees: 16.1,
        }
    }
}

fn whole(x: Option<f64>, f: fn(f64) -> f64) -> Option<i32> {
    x.map(|v| f(v) as i32)
}

/// Kerzenzünden vor Schabbat und Jom Tov (auf die nächste Minute gerundet).
pub fn candle_lighting(loc: &Location, p: &Profile, day: Day) -> Option<i32> {
    whole(sunset(loc, day).map(|s| s - p.candle_minutes), f64::round)
}

/// Schabbat- bzw. Feiertagsende und Kerzenzünden in der zweiten Nacht
/// (aufgerundet).
pub fn shabbat_end(loc: &Location, p: &Profile, day: Day) -> Option<i32> {
    whole(
        nightfall_degrees(loc, day, p.shabbat_end_degrees),
        f64::ceil,
    )
}

/// Fastenbeginn am Morgen (abgerundet).
pub fn fast_begin(loc: &Location, p: &Profile, day: Day) -> Option<i32> {
    whole(dawn_degrees(loc, day, p.fast_begin_degrees), f64::floor)
}

/// Fastenende am Abend (aufgerundet).
pub fn fast_end(loc: &Location, p: &Profile, day: Day) -> Option<i32> {
    whole(nightfall_degrees(loc, day, p.fast_end_degrees), f64::ceil)
}

/// Beginn des Fastens am Vorabend von Tischa beAw: Sonnenuntergang (abgerundet).
pub fn fast_begin_at_sunset(loc: &Location, _p: &Profile, day: Day) -> Option<i32> {
    whole(sunset(loc, day), f64::floor)
}

/// Chanukka-Lichter an Wochentagen: Einbruch der Nacht (aufgerundet).
pub fn chanukka_candles(loc: &Location, p: &Profile, day: Day) -> Option<i32> {
    fast_end(loc, p, day)
}

/// Bdikat Chametz am Abend vor Erew Pessach (Einbruch der Nacht).
pub fn bdikat_chametz(loc: &Location, p: &Profile, day: Day) -> Option<i32> {
    fast_end(loc, p, day)
}

fn chametz_hour(loc: &Location, p: &Profile, day: Day) -> Option<(f64, f64)> {
    let dawn = dawn_degrees(loc, day, p.chametz_degrees)?;
    let dusk = nightfall_degrees(loc, day, p.chametz_degrees)?;
    Some((dawn, (dusk - dawn) / 12.0))
}

/// Ende der Zeit, Chametz zu essen: 4 Stunden nach dem Morgen (abgerundet).
pub fn chametz_eating_ends(loc: &Location, p: &Profile, day: Day) -> Option<i32> {
    chametz_hour(loc, p, day).map(|(d, h)| (d + 4.0 * h).floor() as i32)
}

/// Ende der Zeit, Chametz zu verbrennen: 5 Stunden nach dem Morgen (abgerundet).
pub fn chametz_burning_ends(loc: &Location, p: &Profile, day: Day) -> Option<i32> {
    chametz_hour(loc, p, day).map(|(d, h)| (d + 5.0 * h).floor() as i32)
}

/// Mitternacht der Nacht, die am Abend des Tages beginnt: Mitte zwischen
/// Sonnenuntergang und Sonnenaufgang des Folgetags (aufgerundet). Das
/// Ergebnis sind Minuten seit Mitternacht des *Folgetags* (z. B. 84 = 1:24).
pub fn midnight_after(loc: &Location, day: Day) -> Option<i32> {
    let set = sunset(loc, day)?;
    let rise = sunrise(loc, day + 1)? + 1440.0;
    Some((((set + rise) / 2.0).ceil() as i32).rem_euclid(1440))
}

/// Minuten seit Mitternacht als "hh.mm" (gerundet auf die nächste Minute).
pub fn format_time(minutes: f64, sep: char) -> String {
    let total = minutes.round() as i64;
    let (h, m) = (total.div_euclid(60).rem_euclid(24), total.rem_euclid(60));
    format!("{h}{sep}{m:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use cc_calendar::day_from_gregorian;

    fn hhmm(m: f64) -> String {
        format_time(m, ':')
    }

    #[test]
    fn frankfurt_sunset_matches_known_values() {
        // Hebcal (Frankfurt, 30 Min. vor Sonnenuntergang) für 22.9.2023: Kerzenzünden 18:54
        // → Sonnenuntergang 19:24.
        let loc = Location::frankfurt();
        let d = day_from_gregorian(2023, 9, 22).unwrap();
        let set = sunset(&loc, d).unwrap();
        assert_eq!(hhmm(set), "19:24");
    }

    #[test]
    fn dst_switch_dates() {
        let tz = TimeZone::CET;
        let d = day_from_gregorian(2024, 3, 31).unwrap(); // Umstellung
        assert_eq!(tz.offset_minutes(d, 0.0), 60);
        assert_eq!(tz.offset_minutes(d, 70.0), 120);
        let d = day_from_gregorian(2023, 10, 29).unwrap();
        assert_eq!(tz.offset_minutes(d, 30.0), 120);
        assert_eq!(tz.offset_minutes(d, 70.0), 60);
    }

    #[test]
    fn polar_day_has_no_nightfall() {
        let loc = Location {
            name: "Tromsø".into(),
            latitude: 69.65,
            longitude: 18.96,
            time_zone: TimeZone::CET,
        };
        let d = day_from_gregorian(2024, 6, 21).unwrap();
        assert!(nightfall_degrees(&loc, d, 8.5).is_none());
    }
}
