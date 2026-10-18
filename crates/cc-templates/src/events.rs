//! Eigene Einträge aus einer Textdatei (eine Zeile je Eintrag, Felder mit
//! Semikolon getrennt, `#` leitet Kommentare ein):
//!
//! ```text
//! hebr;8 Tischrei;Jahrzeit Israel ben Zechariahu     jedes Jahr am 8. Tischri
//! jährlich;19.09.;Geburtstag Anna                      jedes Jahr am 19. September
//! einmal;2026-10-05;Elternabend                        nur an diesem Tag
//! ferien;2026-10-05;2026-10-17;Herbstferien            "Beginn der …" / "Ende der …"
//! ```
//!
//! Fehlt ein 30. Cheschwan oder Kislew in einem Jahr, steht der Eintrag am
//! 29. "Adar" meint im Schaltjahr Adar II; "Adar I" im Gemeinjahr Adar.

use std::collections::HashMap;

use cc_calendar::{Day, day_from_gregorian};
use cc_holidays::{DayInfo, month};

#[derive(Debug, Clone, PartialEq)]
enum Rule {
    Hebrew { code: &'static str, day: u8 },
    Yearly { month: u8, day: u8 },
    Once(Day),
    Range(Day, Day),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    rule: Rule,
    text: String,
}

fn hebrew_month_code(name: &str) -> Option<&'static str> {
    let n = name.trim().to_lowercase().replace(['.', '-'], " ");
    let n = n.split_whitespace().collect::<Vec<_>>().join(" ");
    Some(match n.as_str() {
        "tischri" | "tischrei" | "tischrej" | "tishrei" | "tishri" => month::TISHREI,
        "cheschwan" | "marcheschwan" | "cheshvan" | "heshvan" => month::CHESHVAN,
        "kislew" | "kislev" => month::KISLEV,
        "tewet" | "tevet" | "teves" => month::TEVET,
        "schwat" | "schewat" | "shevat" | "shvat" => month::SHEVAT,
        "adar i" | "adar 1" | "adar rischon" => month::ADAR_I,
        "adar" | "adar ii" | "adar 2" | "adar scheni" => month::ADAR,
        "nissan" | "nisan" => month::NISAN,
        "ijar" | "iyar" | "iyyar" => month::IYAR,
        "siwan" | "sivan" => month::SIVAN,
        "tammus" | "tamus" | "tammuz" | "tamuz" => month::TAMMUZ,
        "aw" | "av" => month::AV,
        "elul" => month::ELUL,
        _ => return None,
    })
}

fn parse_date(s: &str) -> Option<Day> {
    let s = s.trim();
    if let Some((y, rest)) = s.split_once('-') {
        let (m, d) = rest.split_once('-')?;
        return day_from_gregorian(y.parse().ok()?, m.parse().ok()?, d.parse().ok()?).ok();
    }
    let parts: Vec<&str> = s.split('.').filter(|p| !p.is_empty()).collect();
    if let [d, m, y] = parts.as_slice() {
        return day_from_gregorian(y.parse().ok()?, m.parse().ok()?, d.parse().ok()?).ok();
    }
    None
}

/// Liest Einträge; Fehler nennen die Zeilennummer.
pub fn parse(text: &str) -> Result<Vec<Event>, String> {
    let mut out = Vec::new();
    for (no, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split(';').map(str::trim).collect();
        let err = |m: &str| format!("Zeile {}: {m}: {line}", no + 1);
        let kind = f[0].to_lowercase();
        let rule = match kind.as_str() {
            "hebr" | "hebräisch" | "hebraeisch" => {
                let date = f.get(1).ok_or_else(|| err("Datum fehlt"))?;
                let (d, m) = date
                    .trim()
                    .split_once(' ')
                    .ok_or_else(|| err("Datum wie \"8 Tischrei\""))?;
                let day: u8 = d
                    .trim_end_matches('.')
                    .parse()
                    .map_err(|_| err("Tag nicht lesbar"))?;
                let code = hebrew_month_code(m).ok_or_else(|| err("Monat unbekannt"))?;
                if !(1..=30).contains(&day) {
                    return Err(err("Tag 1–30"));
                }
                Rule::Hebrew { code, day }
            }
            "jährlich" | "jaehrlich" | "greg" => {
                let date = f.get(1).ok_or_else(|| err("Datum fehlt"))?;
                let p: Vec<&str> = date.split('.').filter(|x| !x.is_empty()).collect();
                let [d, m] = p.as_slice() else {
                    return Err(err("Datum wie \"19.09.\""));
                };
                Rule::Yearly {
                    month: m.parse().map_err(|_| err("Monat nicht lesbar"))?,
                    day: d.parse().map_err(|_| err("Tag nicht lesbar"))?,
                }
            }
            "einmal" | "datum" => Rule::Once(
                parse_date(f.get(1).ok_or_else(|| err("Datum fehlt"))?)
                    .ok_or_else(|| err("Datum wie 2026-10-05 oder 05.10.2026"))?,
            ),
            "ferien" | "zeitraum" => {
                let a = parse_date(f.get(1).ok_or_else(|| err("Beginn fehlt"))?)
                    .ok_or_else(|| err("Beginn nicht lesbar"))?;
                let b = parse_date(f.get(2).ok_or_else(|| err("Ende fehlt"))?)
                    .ok_or_else(|| err("Ende nicht lesbar"))?;
                Rule::Range(a, b)
            }
            _ => return Err(err("Art unbekannt (hebr, jährlich, einmal, ferien)")),
        };
        let text_idx = if matches!(rule, Rule::Range(..)) {
            3
        } else {
            2
        };
        let text = f
            .get(text_idx..)
            .map(|t| t.join("; "))
            .filter(|t| !t.is_empty())
            .ok_or_else(|| err("Text fehlt"))?;
        out.push(Event { rule, text });
    }
    Ok(out)
}

/// Texte der Einträge je Tag für eine fortlaufende Tagesliste.
pub fn resolve(events: &[Event], days: &[DayInfo]) -> HashMap<Day, Vec<String>> {
    let mut map: HashMap<Day, Vec<String>> = HashMap::new();
    for (i, d) in days.iter().enumerate() {
        let month_has_29_days =
            d.hebrew.day == 29 && days.get(i + 1).is_some_and(|n| n.hebrew.day == 1);
        for e in events {
            let text = match &e.rule {
                Rule::Hebrew { code, day } => {
                    let code = if *code == month::ADAR_I && !d.hebrew.leap_year {
                        month::ADAR
                    } else {
                        code
                    };
                    let same_month = d.hebrew.month_code == code;
                    let hit = d.hebrew.day == *day || (*day == 30 && month_has_29_days);
                    (same_month && hit).then(|| e.text.clone())
                }
                Rule::Yearly { month, day } => {
                    (d.gregorian.1 == *month && d.gregorian.2 == *day).then(|| e.text.clone())
                }
                Rule::Once(x) => (d.day == *x).then(|| e.text.clone()),
                Rule::Range(a, b) => {
                    if d.day == *a {
                        Some(format!("Beginn der {}", e.text))
                    } else if d.day == *b {
                        Some(format!("Ende der {}", e.text))
                    } else {
                        None
                    }
                }
            };
            if let Some(t) = text {
                map.entry(d.day).or_default().push(t);
            }
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use cc_holidays::days_between;

    #[test]
    fn parses_and_resolves() {
        let ev = parse(
            "# Kommentar\nhebr;8 Tischrei;Jahrzeit\njährlich;19.09.;Geburtstag\neinmal;2026-10-05;Elternabend\nferien;2026-10-05;2026-10-17;Herbstferien\nhebr;30 Cheschwan;Test 30\n",
        )
        .unwrap();
        assert_eq!(ev.len(), 5);
        let from = day_from_gregorian(2026, 9, 1).unwrap();
        let to = day_from_gregorian(2027, 12, 31).unwrap();
        let days = days_between(from, to);
        let m = resolve(&ev, &days);
        let at = |y, mo, d| {
            m.get(&day_from_gregorian(y, mo, d).unwrap())
                .cloned()
                .unwrap_or_default()
        };
        // 8. Tischri 5787 = 19. September 2026 (wie im Beispielkalender)
        assert_eq!(at(2026, 9, 19), ["Jahrzeit", "Geburtstag"]);
        assert_eq!(at(2026, 10, 5), ["Elternabend", "Beginn der Herbstferien"]);
        assert_eq!(at(2026, 10, 17), ["Ende der Herbstferien"]);
        // 30. Cheschwan 5787 gibt es (11. November 2026 ist Rosch Chodesch Kislew, 1. Tag)
        assert_eq!(at(2026, 11, 10), ["Test 30"]);
    }

    #[test]
    fn reports_line_numbers() {
        let e = parse("hebr;8 Tischrei;ok\nfoo;1;x\n").unwrap_err();
        assert!(e.starts_with("Zeile 2"), "{e}");
    }
}
