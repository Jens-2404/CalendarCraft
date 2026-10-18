//! Feiertage, Wochenabschnitte und Tagesinformationen des jüdischen Jahres.
//!
//! Stand Prototyp: Diaspora (Israel-Abweichungen folgen), deutsche Namen
//! in der Schreibweise des Gemeindekalenders Frankfurt. Die
//! Wochenabschnitte kommen aus einer generierten Tabelle
//! (`parasha_table`, siehe tools/gen_parasha_table.py).

pub mod german;
pub mod names;
mod parasha_table;

use cc_calendar::{
    Day, FRIDAY, HebrewDate, MONDAY, SATURDAY, SUNDAY, TUESDAY, Weekday, gregorian_from_day,
    hebrew_from_day, hebrew_year_length, is_hebrew_leap_year, rosh_hashana, weekday,
};

/// Art eines Eintrags; bestimmt u. a. die Darstellung der Zeile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// Jom Tov mit Arbeitsverbot
    YomTov,
    CholHamoed,
    Erev,
    Fast,
    Chanukka,
    Purim,
    /// Kleinere Tage (Tu biSchwat, Lag baOmer …)
    Minor,
    /// Moderne Gedenk- und Feiertage
    Modern,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holiday {
    pub class: Class,
    pub name_de: String,
}

impl Holiday {
    fn new(class: Class, name: impl Into<String>) -> Self {
        Self {
            class,
            name_de: name.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoshChodesh {
    /// Ein Tag
    Single,
    /// Erster von zwei Tagen (30. des Vormonats)
    First,
    /// Zweiter von zwei Tagen (1. des Monats)
    Second,
}

/// Wochenabschnitt, ggf. Doppelabschnitt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parasha {
    /// Index des (ersten) Abschnitts, 0 = Bereschit
    pub first: u8,
    pub pair: bool,
}

impl Parasha {
    pub fn name_de(&self) -> String {
        names::parasha_name_de(self.first, self.pair)
    }
}

/// Alles, was an einem Tag im Kalender steht (ohne Zeiten).
#[derive(Debug, Clone)]
pub struct DayInfo {
    pub day: Day,
    pub gregorian: (i32, u8, u8),
    pub weekday: Weekday,
    pub hebrew: HebrewDate,
    pub parasha: Option<Parasha>,
    pub special_shabbat: Option<&'static str>,
    pub holidays: Vec<Holiday>,
    pub rosh_chodesh: Option<RoshChodesh>,
    /// Omerzählung (1–49), wie im Gemeindekalender am Tag nach dem Zählabend
    pub omer: Option<u8>,
    /// Jom Tov (Arbeitsverbot): Rosch HaSchana, Jom Kippur, Sukkot … Schawuot
    pub yom_tov: bool,
    /// Totengedenken (Jiskor)
    pub yizkor: bool,
    /// Erste Selichot (Samstagnacht), Zeit wird im Zmanim-Teil berechnet
    pub first_selichot: bool,
    /// Schabbat Mewarchim: Monatscode des angekündigten Monats
    pub mevarchim: Option<String>,
}

impl DayInfo {
    pub fn is_shabbat(&self) -> bool {
        self.weekday == SATURDAY
    }
}

/// Monatscodes des hebräischen Kalenders (ICU/CLDR).
pub mod month {
    pub const TISHREI: &str = "M01";
    pub const CHESHVAN: &str = "M02";
    pub const KISLEV: &str = "M03";
    pub const TEVET: &str = "M04";
    pub const SHEVAT: &str = "M05";
    pub const ADAR_I: &str = "M05L";
    /// Adar im Gemeinjahr, Adar II im Schaltjahr
    pub const ADAR: &str = "M06";
    pub const NISAN: &str = "M07";
    pub const IYAR: &str = "M08";
    pub const SIVAN: &str = "M09";
    pub const TAMMUZ: &str = "M10";
    pub const AV: &str = "M11";
    pub const ELUL: &str = "M12";
}

/// Die Tage von Erew Rosch HaSchana (29. Elul des Vorjahres) bis
/// Erew Rosch HaSchana des nächsten Jahres, wie im Gemeindekalender.
pub fn year_days(year: i32) -> Vec<DayInfo> {
    let start = rosh_hashana(year) - 1;
    let end = rosh_hashana(year + 1) - 1; // inklusive
    let mut days: Vec<DayInfo> = (start..=end)
        .map(|d| DayInfo {
            day: d,
            gregorian: gregorian_from_day(d),
            weekday: weekday(d),
            hebrew: hebrew_from_day(d),
            parasha: None,
            special_shabbat: None,
            holidays: Vec::new(),
            rosh_chodesh: None,
            omer: None,
            yom_tov: false,
            yizkor: false,
            first_selichot: false,
            mevarchim: None,
        })
        .collect();

    let ctx = Ctx { year, start };
    ctx.add_parashot(&mut days);
    ctx.add_holidays(&mut days);
    ctx.add_rosh_chodesh(&mut days);
    ctx.add_special_shabbatot(&mut days);
    ctx.add_selichot(&mut days);
    ctx.add_mevarchim(&mut days);
    days
}

/// Die Tage `from..=to` (Tageszahlen), auch über Jahresgrenzen des
/// jüdischen Kalenders hinweg, z. B. für einen gregorianischen Jahreskalender.
pub fn days_between(from: Day, to: Day) -> Vec<DayInfo> {
    let first_year = cc_calendar::hebrew_from_day(from).year;
    let last_year = cc_calendar::hebrew_from_day(to).year;
    let mut out: Vec<DayInfo> = Vec::new();
    for y in first_year..=last_year + 1 {
        for d in year_days(y) {
            // Erew Rosch HaSchana steht am Ende des einen und am Anfang des
            // nächsten Jahres; die erste Fassung (mit Parascha) bleibt.
            if out.last().is_some_and(|l| l.day >= d.day) {
                continue;
            }
            out.push(d);
        }
    }
    out.retain(|d| (from..=to).contains(&d.day));
    out
}

struct Ctx {
    year: i32,
    start: Day,
}

impl Ctx {
    fn idx(&self, day: Day) -> usize {
        (day - self.start) as usize
    }

    /// Tageszahl des hebräischen Datums im Jahr (Monatscode, Tag).
    fn day_of(&self, days: &[DayInfo], code: &str, d: u8) -> Option<Day> {
        days.iter()
            .find(|i| i.hebrew.month_code == code && i.hebrew.day == d)
            .map(|i| i.day)
    }

    fn add_parashot(&self, days: &mut [DayInfo]) {
        let rh = rosh_hashana(self.year);
        let rh_wd = weekday(rh);
        let len = hebrew_year_length(self.year) as u16;
        let Some(t) = parasha_table::YEAR_TYPES
            .iter()
            .find(|t| t.rh_weekday == rh_wd && t.length == len)
        else {
            return;
        };
        let first_sat = rh + i64::from((6 + 7 - rh_wd) % 7);
        for &(week, first, pair) in t.readings {
            let day = first_sat + 7 * i64::from(week);
            if day >= self.start && day < self.start + days.len() as i64 {
                days[self.idx(day)].parasha = Some(Parasha { first, pair });
            }
        }
    }

    fn put(&self, days: &mut [DayInfo], code: &str, d: u8, h: Holiday) -> Option<Day> {
        let day = self.day_of(days, code, d)?;
        let i = self.idx(day);
        if h.class == Class::YomTov {
            days[i].yom_tov = true;
        }
        days[i].holidays.push(h);
        Some(day)
    }

    /// Fasttag mit Verschiebung: fällt er auf Schabbat, auf den Sonntag.
    fn put_fast_after_shabbat(&self, days: &mut [DayInfo], code: &str, d: u8, name: &str) {
        if let Some(day) = self.day_of(days, code, d) {
            let target = if weekday(day) == SATURDAY {
                day + 1
            } else {
                day
            };
            days[self.idx(target)]
                .holidays
                .push(Holiday::new(Class::Fast, name));
        }
    }

    fn add_holidays(&self, days: &mut [DayInfo]) {
        use month::*;
        let leap = is_hebrew_leap_year(self.year);
        let h = |c: Class, n: &str| Holiday::new(c, n);

        // Tischri
        // Erew Rosch HaSchana: der 29. Elul am Anfang und am Ende des Zeitraums
        for d in days.iter_mut() {
            if d.hebrew.month_code == ELUL && d.hebrew.day == 29 {
                d.holidays.push(h(Class::Erev, "Erew Rosch Haschana"));
            }
        }
        self.put(days, TISHREI, 1, h(Class::YomTov, "1. Tag Rosch Haschana"));
        self.put(days, TISHREI, 2, h(Class::YomTov, "2. Tag Rosch Haschana"));
        self.put_fast_after_shabbat(days, TISHREI, 3, "Fasten Gedalja");
        self.put(days, TISHREI, 9, h(Class::Erev, "Erew Jom Kippur"));
        self.put(days, TISHREI, 10, h(Class::YomTov, "Jom Kippur"));
        self.put(days, TISHREI, 14, h(Class::Erev, "Erew Sukkot"));
        self.put(days, TISHREI, 15, h(Class::YomTov, "1. Tag Sukkot"));
        self.put(days, TISHREI, 16, h(Class::YomTov, "2. Tag Sukkot"));
        for d in 17..=20 {
            self.put(days, TISHREI, d, h(Class::CholHamoed, "Chol Hamoed"));
        }
        self.put(days, TISHREI, 21, h(Class::CholHamoed, "Hoschana Raba"));
        self.put(days, TISHREI, 22, h(Class::YomTov, "Schemini Azeret"));
        self.put(days, TISHREI, 23, h(Class::YomTov, "Simchat Torah"));

        // Chanukka
        self.put(days, KISLEV, 24, h(Class::Erev, "Erew Chanukka"));
        if let Some(c1) = self.day_of(days, KISLEV, 25) {
            for n in 0..8 {
                days[self.idx(c1 + n)]
                    .holidays
                    .push(h(Class::Chanukka, &format!("{}. Tag Chanukka", n + 1)));
            }
        }
        // 10. Tewet (nie verschoben)
        self.put(days, TEVET, 10, h(Class::Fast, "Fasten 10. Tewet"));

        // Schwat / Adar
        self.put(days, SHEVAT, 15, h(Class::Minor, "Tu biSchwat"));
        if leap {
            self.put(days, ADAR_I, 14, h(Class::Minor, "Purim Katan"));
            self.put(days, ADAR_I, 15, h(Class::Minor, "Schuschan Purim Katan"));
        }
        // Ta'anit Esther: 13. Adar, auf Schabbat → Donnerstag davor
        if let Some(d13) = self.day_of(days, ADAR, 13) {
            let t = if weekday(d13) == SATURDAY {
                d13 - 2
            } else {
                d13
            };
            days[self.idx(t)]
                .holidays
                .push(h(Class::Fast, "Fasten Esther"));
        }
        self.put(days, ADAR, 14, h(Class::Purim, "Purim"));
        self.put(days, ADAR, 15, h(Class::Purim, "Schuschan Purim"));

        // Nisan
        // Ta'anit Bechorot: 14. Nisan, auf Schabbat → Donnerstag davor
        if let Some(d14) = self.day_of(days, NISAN, 14) {
            let t = if weekday(d14) == SATURDAY {
                d14 - 2
            } else {
                d14
            };
            days[self.idx(t)]
                .holidays
                .push(h(Class::Fast, "Taanit Bechorot"));
        }
        self.put(days, NISAN, 14, h(Class::Erev, "Erew Pessach"));
        self.put(days, NISAN, 15, h(Class::YomTov, "1. Tag Pessach"));
        self.put(days, NISAN, 16, h(Class::YomTov, "2. Tag Pessach"));
        for d in 17..=20 {
            self.put(days, NISAN, d, h(Class::CholHamoed, "Chol Hamoed"));
        }
        self.put(days, NISAN, 21, h(Class::YomTov, "7. Tag Pessach"));
        self.put(days, NISAN, 22, h(Class::YomTov, "8. Tag Pessach"));

        // Omerzählung: 1 am 16. Nisan … 49 am 5. Siwan
        if let Some(d16) = self.day_of(days, NISAN, 16) {
            for n in 0..49 {
                days[self.idx(d16 + n)].omer = Some(n as u8 + 1);
            }
        }

        // Jom HaSchoa: 27. Nisan; Freitag → Donnerstag, Sonntag → Montag
        if let Some(d) = self.day_of(days, NISAN, 27) {
            let t = match weekday(d) {
                FRIDAY => d - 1,
                SUNDAY => d + 1,
                _ => d,
            };
            days[self.idx(t)]
                .holidays
                .push(h(Class::Modern, "Jom HaSchoa"));
        }
        // Jom HaSikaron / Jom HaAzmaut (4./5. Ijar mit Verschiebung)
        if let Some(d5) = self.day_of(days, IYAR, 5) {
            let t = match weekday(d5) {
                FRIDAY => d5 - 1,
                SATURDAY => d5 - 2,
                MONDAY => d5 + 1,
                _ => d5,
            };
            days[self.idx(t - 1)]
                .holidays
                .push(h(Class::Modern, "Jom HaSikaron"));
            days[self.idx(t)]
                .holidays
                .push(h(Class::Modern, "Jom HaAzmaut"));
        }
        self.put(days, IYAR, 14, h(Class::Minor, "Pessach Scheni"));
        self.put(days, IYAR, 18, h(Class::Minor, "Lag Baomer"));
        self.put(days, IYAR, 28, h(Class::Modern, "Jom Jeruschalajim"));
        // Gedenktag an das Massaker vom 7. Oktober (gesetzlich ab 5786): 24. Tischri,
        // fällt er auf Schabbat, am Sonntag
        if self.year >= 5786
            && let Some(d) = self.day_of(days, TISHREI, 24)
        {
            let t = if weekday(d) == SATURDAY { d + 1 } else { d };
            days[self.idx(t)]
                .holidays
                .push(h(Class::Modern, "Gedenktag an das Massaker vom 7. Oktober"));
        }

        // Siwan
        self.put(days, SIVAN, 5, h(Class::Erev, "Erew Schawuot"));
        self.put(days, SIVAN, 6, h(Class::YomTov, "1. Tag Schawuot"));
        self.put(days, SIVAN, 7, h(Class::YomTov, "2. Tag Schawuot"));

        // Tammus / Aw
        self.put_fast_after_shabbat(days, TAMMUZ, 17, "Fasten 17. Tammus");
        // 9. Aw: auf Schabbat → Sonntag, 10. Aw
        self.put_fast_after_shabbat(days, AV, 9, "Fasten 9. Aw");
        self.put(days, AV, 15, h(Class::Minor, "Tu beAw"));

        // Jiskor
        for (code, d) in [(TISHREI, 22), (NISAN, 22), (SIVAN, 7)] {
            if let Some(day) = self.day_of(days, code, d) {
                days[self.idx(day)].yizkor = true;
            }
        }
    }

    fn add_rosh_chodesh(&self, days: &mut [DayInfo]) {
        for i in 0..days.len() {
            let h = &days[i].hebrew;
            if h.day == 1 && h.month_code != month::TISHREI {
                let prev_30 = i > 0 && days[i - 1].hebrew.day == 30;
                if prev_30 {
                    days[i - 1].rosh_chodesh = Some(RoshChodesh::First);
                    days[i].rosh_chodesh = Some(RoshChodesh::Second);
                } else {
                    days[i].rosh_chodesh = Some(RoshChodesh::Single);
                }
            }
        }
    }

    fn add_special_shabbatot(&self, days: &mut [DayInfo]) {
        use month::*;
        // Letzter Schabbat auf oder vor einem Tag
        let sat_on_or_before = |d: Day| d - i64::from((weekday(d) + 1) % 7);
        // Schabbat Schuwa: Schabbat zwischen Rosch HaSchana und Jom Kippur
        if let (Some(a), Some(b)) = (self.day_of(days, TISHREI, 3), self.day_of(days, TISHREI, 9))
            && let Some(s) = (a..=b).find(|&d| weekday(d) == SATURDAY)
        {
            days[self.idx(s)].special_shabbat = Some("Schabbat Schuwa");
        }
        // Schekalim: Schabbat auf oder vor Rosch Chodesch Adar (Adar II im Schaltjahr)
        if let Some(rc) = self.day_of(days, ADAR, 1) {
            days[self.idx(sat_on_or_before(rc))].special_shabbat = Some("Schabbat Schkalim");
        }
        // Sachor: Schabbat vor Purim
        if let Some(p) = self.day_of(days, ADAR, 14) {
            days[self.idx(sat_on_or_before(p - 1))].special_shabbat = Some("Schabbat Sachor");
        }
        // HaChodesch: Schabbat auf oder vor Rosch Chodesch Nisan; Para eine Woche davor
        if let Some(rc) = self.day_of(days, NISAN, 1) {
            let hc = sat_on_or_before(rc);
            days[self.idx(hc)].special_shabbat = Some("Schabbat haChodesch");
            days[self.idx(hc - 7)].special_shabbat = Some("Schabbat Para");
        }
        // HaGadol: Schabbat vor Pessach (8.–14. Nisan)
        if let Some(e) = self.day_of(days, NISAN, 14) {
            days[self.idx(sat_on_or_before(e))].special_shabbat = Some("Schabbat haGadol");
        }
    }

    /// Schabbat Mewarchim: letzter Schabbat vor dem (ersten) Tag Rosch
    /// Chodesch, für alle Monate außer Tischri.
    fn add_mevarchim(&self, days: &mut [DayInfo]) {
        for i in 0..days.len() {
            let h = &days[i].hebrew;
            if h.day != 1 || h.month_code == month::TISHREI {
                continue;
            }
            let code = h.month_code.clone();
            let first_rc = if i > 0 && days[i - 1].hebrew.day == 30 {
                days[i].day - 1
            } else {
                days[i].day
            };
            // Schabbat strikt vor dem ersten Tag Rosch Chodesch
            let back = i64::from((weekday(first_rc) + 1) % 7);
            let sat = first_rc - if back == 0 { 7 } else { back };
            if sat >= self.start && sat < self.start + days.len() as i64 {
                days[self.idx(sat)].mevarchim = Some(code);
            }
        }
    }

    fn add_selichot(&self, days: &mut [DayInfo]) {
        // Erste Selichot: Samstagnacht vor Rosch HaSchana; fällt dieser auf
        // Montag oder Dienstag, eine Woche früher (mindestens vier Tage davor).
        let next_rh = rosh_hashana(self.year + 1);
        let wd = weekday(next_rh);
        let back = if wd == SATURDAY { 7 } else { i64::from(wd) + 1 };
        let mut sat = next_rh - back;
        if matches!(wd, MONDAY | TUESDAY) {
            sat -= 7;
        }
        if sat >= self.start && sat < self.start + days.len() as i64 {
            days[self.idx(sat)].first_selichot = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find(days: &[DayInfo], g: (i32, u8, u8)) -> &DayInfo {
        days.iter().find(|d| d.gregorian == g).unwrap()
    }

    #[test]
    fn range_matches_community_calendar_5784() {
        let days = year_days(5784);
        assert_eq!(days.first().unwrap().gregorian, (2023, 9, 15));
        assert_eq!(days.last().unwrap().gregorian, (2024, 10, 2));
        assert_eq!(days.len(), 384);
    }

    #[test]
    fn parashot_5784_match_community_calendar() {
        let days = year_days(5784);
        // Samstage und Abschnitte laut Gemeindekalender Frankfurt 5784
        for (g, name) in [
            ((2023, 9, 23), "Haasinu"),
            ((2023, 10, 14), "Bereschit"),
            ((2023, 11, 4), "Wajera"),
            ((2024, 3, 9), "Wajakhel"),
            ((2024, 3, 16), "Pekudei"),
            ((2024, 3, 23), "Wajikra"),
            ((2024, 4, 13), "Tasria"),
            ((2024, 6, 15), "Nasso"),
            ((2024, 9, 28), "Nizawim – Wajelech"),
        ] {
            let d = find(&days, g);
            assert!(d.is_shabbat(), "{g:?} ist kein Schabbat");
            if let Some(p) = d.parasha {
                assert_eq!(p.name_de(), name, "{g:?}");
            }
        }
        assert!(find(&days, (2024, 9, 28)).first_selichot);
    }

    #[test]
    fn gregorian_year_2027_spans_two_jewish_years() {
        let from = cc_calendar::day_from_gregorian(2027, 1, 1).unwrap();
        let to = cc_calendar::day_from_gregorian(2027, 12, 31).unwrap();
        let days = days_between(from, to);
        assert_eq!(days.len(), 365);
        assert!(days.windows(2).all(|w| w[1].day == w[0].day + 1));
        let names = |g| {
            find(&days, g)
                .holidays
                .iter()
                .map(|h| h.name_de.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(names((2027, 3, 23)), ["Purim"]); // 5787
        assert_eq!(names((2027, 10, 1)), ["Erew Rosch Haschana"]);
        assert_eq!(names((2027, 10, 2)), ["1. Tag Rosch Haschana"]); // 5788
        assert_eq!(names((2027, 10, 11)), ["Jom Kippur"]);
    }

    #[test]
    fn holidays_5784() {
        let days = year_days(5784);
        let names = |g| {
            find(&days, g)
                .holidays
                .iter()
                .map(|h| h.name_de.as_str())
                .collect::<Vec<_>>()
        };
        assert_eq!(names((2023, 9, 15)), ["Erew Rosch Haschana"]);
        assert_eq!(names((2023, 9, 16)), ["1. Tag Rosch Haschana"]);
        assert_eq!(names((2023, 9, 18)), ["Fasten Gedalja"]);
        assert_eq!(names((2024, 3, 21)), ["Fasten Esther"]);
        assert_eq!(names((2024, 3, 24)), ["Purim"]);
        assert_eq!(names((2024, 4, 23)), ["1. Tag Pessach"]);
        assert_eq!(names((2024, 5, 6)), ["Jom HaSchoa"]);
        assert_eq!(names((2024, 5, 13)), ["Jom HaSikaron"]);
        assert_eq!(names((2024, 5, 14)), ["Jom HaAzmaut"]);
        assert_eq!(names((2024, 8, 13)), ["Fasten 9. Aw"]);
        assert_eq!(find(&days, (2024, 4, 24)).omer, Some(1));
        assert_eq!(find(&days, (2024, 6, 11)).omer, Some(49));
        assert!(find(&days, (2023, 10, 7)).yizkor);
    }
}
