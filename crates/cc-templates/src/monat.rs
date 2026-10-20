//! Vorlage "Monatskalender": ein Monat je Seite im Wochenraster
//! Sonntag–Schabbat. Zwei Stile nach Beispielen des Auftraggebers:
//!
//! - `Klassisch` (quer): großer gregorianischer Tag, darunter der hebräische
//!   Tag in Buchstaben, hebräisches Datum hochkant am Rand, Einträge rechts
//!   oben, Zeiten mit Symbolen unten; Schabbat und Feiertage grau.
//! - `Foto` (hoch): Fotofläche oben, darunter das Raster mit grauem Balken
//!   für das hebräische Datum.

use std::collections::HashMap;
use std::fmt::Write as _;

use cc_astro::{Location, Profile};
use cc_calendar::{Day, SATURDAY, day_from_gregorian, weekday};
use cc_holidays::{Class, DayInfo, days_between, german};
use cc_i18n::hebrew_numeral;

use crate::common::{GREG_MONTHS, Paper, Style, month_name_de, q};
use crate::times::{DayTimes, MainTime, day_times};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonthStyle {
    Klassisch,
    Foto,
}

#[derive(Debug, Clone)]
pub struct Options {
    pub paper: Paper,
    pub style: Style,
    pub month_style: MonthStyle,
    pub location: Location,
    pub zmanim: Profile,
    /// Dateinamen der Fotos (wie an `cc_render::compile` übergeben), der
    /// Reihe nach auf die Monate verteilt
    pub photos: Vec<String>,
    pub photos_per_month: usize,
    /// Eigene Einträge je Tag (siehe `events`)
    pub events: HashMap<Day, Vec<String>>,
    /// Bundesland für die gesetzlichen Feiertage
    pub region: german::State,
}

impl Options {
    pub fn new(month_style: MonthStyle) -> Self {
        Self {
            paper: match month_style {
                MonthStyle::Klassisch => Paper::A4,
                MonthStyle::Foto => Paper::A4.portrait(),
            },
            style: Style::default(),
            month_style,
            location: Location::frankfurt(),
            zmanim: Profile::frankfurt(),
            photos: Vec::new(),
            photos_per_month: 1,
            events: HashMap::new(),
            region: german::State::Hessen,
        }
    }
}

/// Symbole als SVG; der Aufrufer übergibt sie beim Setzen als Dateien.
pub fn icon_files() -> Vec<(&'static str, Vec<u8>)> {
    let candle = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><g fill="#3a3a3a"><rect x="4.5" y="10" width="4.5" height="13" rx="0.8"/><rect x="15" y="10" width="4.5" height="13" rx="0.8"/></g><g fill="#e39a2d"><path d="M6.75 1.5c1.9 2.3 2.4 3.9 2.4 5a2.4 2.4 0 0 1-4.8 0c0-1.1.6-2.7 2.4-5z"/><path d="M17.25 1.5c1.9 2.3 2.4 3.9 2.4 5a2.4 2.4 0 0 1-4.8 0c0-1.1.6-2.7 2.4-5z"/></g></svg>"##;
    let havdalah = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><g fill="#3a3a3a"><path d="M8 11h3l-1.5 12H8z"/><path d="M13 11h3v12h-1.5z"/><path d="M10.5 11h3l1 12h-5z" fill="#6b6b6b"/></g><g fill="#e39a2d"><path d="M9.5 2c1.6 2 2 3.3 2 4.4a2 2 0 0 1-4 0c0-1 .5-2.4 2-4.4z"/><path d="M14.5 2c1.6 2 2 3.3 2 4.4a2 2 0 0 1-4 0c0-1 .5-2.4 2-4.4z"/></g></svg>"##;
    let moon = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="#3a3a3a" d="M15.5 2.5A9.5 9.5 0 1 0 21.5 17 7.6 7.6 0 0 1 15.5 2.5z"/><circle cx="19" cy="6" r="1.2" fill="#3a3a3a"/></svg>"##;
    let fast = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><circle cx="12" cy="12" r="9.5" fill="none" stroke="#3a3a3a" stroke-width="2"/><path d="M8 6v6m0 0v6M6.5 6v4a1.5 1.5 0 0 0 3 0V6M16 6c-1.5 1-2 3-2 5h2v7" fill="none" stroke="#3a3a3a" stroke-width="1.4"/><path d="M4 20 20 4" stroke="#3a3a3a" stroke-width="2"/></svg>"##;
    vec![
        ("icon-candle.svg", candle.as_bytes().to_vec()),
        ("icon-end.svg", havdalah.as_bytes().to_vec()),
        ("icon-moon.svg", moon.as_bytes().to_vec()),
        ("icon-fast.svg", fast.as_bytes().to_vec()),
    ]
}

/// Ein Eintrag in einer Tageszelle: Text und Art ("b" fett, "n" normal,
/// "i" kursiv/klein, "c" eigener Eintrag).
type Ev = (String, &'static str);

fn fmt_time(m: i32, uhr: bool) -> String {
    let s = format!("{}:{:02}", m.div_euclid(60), m.rem_euclid(60));
    if uhr { format!("{s} Uhr") } else { s }
}

fn cell_events(
    d: &DayInfo,
    t: &DayTimes,
    o: &Options,
    state: Option<&str>,
    dst_change: bool,
) -> Vec<Ev> {
    let mut ev: Vec<Ev> = Vec::new();
    if let Some(p) = d.parasha {
        ev.push((p.name_de(), "n"));
    }
    if d.mevarchim.is_some() {
        ev.push(("Mewarchim".into(), "n"));
    }
    if let Some(s) = d.special_shabbat {
        ev.push((s.into(), "i"));
    }
    for h in &d.holidays {
        if h.name_de == "Taanit Bechorot" && o.month_style == MonthStyle::Foto {
            continue;
        }
        let kind = match h.class {
            Class::YomTov | Class::Fast | Class::Purim | Class::Chanukka => "b",
            _ => "n",
        };
        ev.push((h.name_de.clone(), kind));
    }
    if d.rosh_chodesh.is_some() {
        ev.push(("Rosch Chodesch".into(), "n"));
    }
    if d.yizkor {
        ev.push(("Jiskor".into(), "n"));
    }
    if d.first_selichot {
        ev.push(("Erste Slichot".into(), "i"));
    }
    if let Some(n) = d.omer {
        ev.push((format!("{n}. Tag Omer"), "i"));
    }
    if let Some(c) = t.chametz_eating_ends {
        ev.push((format!("Chametz-Verbot {}", fmt_time(c, false)), "i"));
    }
    if let Some(c) = t.chametz_burning_ends {
        ev.push((format!("Chametz verbrennen {}", fmt_time(c, false)), "i"));
    }
    if let Some(b) = t.bdikat_chametz {
        ev.push((format!("Bdikat Chametz {}", fmt_time(b, false)), "i"));
    }
    if let Some(name) = state {
        ev.push((format!("{name} (gesetzl. Feiertag)"), "i"));
    }
    if dst_change {
        ev.push(("Zeitumstellung".into(), "i"));
    }
    if let Some(list) = o.events.get(&d.day) {
        for e in list {
            ev.push((e.clone(), "c"));
        }
    }
    ev
}

fn cell_times(d: &DayInfo, t: &DayTimes, uhr: bool, foto: bool) -> Vec<(&'static str, String)> {
    let end_icon = if foto { "moon" } else { "end" };
    let mut v: Vec<(&'static str, String)> = Vec::new();
    match t.main {
        Some(MainTime::Candles(m)) => v.push(("candle", fmt_time(m, uhr))),
        Some(MainTime::SecondNightCandles(m)) => v.push((
            "candle",
            format!("{}{}", if uhr { "Ab " } else { "" }, fmt_time(m, uhr)),
        )),
        Some(MainTime::End(m)) => v.push((end_icon, fmt_time(m, uhr))),
        Some(MainTime::ChanukkaCandles(m)) => v.push(("candle", fmt_time(m, uhr))),
        None => {}
    }
    let taanit_bechorot = d.holidays.iter().any(|h| h.name_de == "Taanit Bechorot");
    if !(foto && taanit_bechorot) {
        if let Some(b) = t.fast_begin {
            v.push(("fast", format!("Beginn {}", fmt_time(b, uhr))));
        }
        if let Some(e) = t.fast_end {
            v.push(("fast", format!("Ende {}", fmt_time(e, uhr))));
        }
    }
    v
}

fn typst_array_ev(ev: &[Ev]) -> String {
    let items: Vec<String> = ev
        .iter()
        .map(|(t, k)| format!("({}, {})", q(t), q(k)))
        .collect();
    format!(
        "({}{})",
        items.join(", "),
        if items.len() == 1 { "," } else { "" }
    )
}

fn typst_array_times(v: &[(&str, String)]) -> String {
    let items: Vec<String> = v
        .iter()
        .map(|(i, t)| format!("({}, {})", q(i), q(t)))
        .collect();
    format!(
        "({}{})",
        items.join(", "),
        if items.len() == 1 { "," } else { "" }
    )
}

/// Hebräische Monate eines gregorianischen Monats, z. B. "Tischrej / Cheschwan 5787".
fn hebrew_months(days: &[&DayInfo], sep: &str) -> String {
    let mut parts: Vec<(i32, &'static str)> = Vec::new();
    for d in days {
        let m = month_name_de(&d.hebrew.month_code, d.hebrew.leap_year);
        if parts.last() != Some(&(d.hebrew.year, m)) {
            parts.push((d.hebrew.year, m));
        }
    }
    let (y0, _) = parts[0];
    let (y1, _) = *parts.last().unwrap();
    if y0 == y1 {
        let names: Vec<&str> = parts.iter().map(|p| p.1).collect();
        format!("{} {y0}", names.join(sep))
    } else {
        parts
            .iter()
            .map(|(y, m)| format!("{m} {y}"))
            .collect::<Vec<_>>()
            .join(sep)
    }
}

const WEEKDAYS: [&str; 7] = [
    "Sonntag",
    "Montag",
    "Dienstag",
    "Mittwoch",
    "Donnerstag",
    "Freitag",
    "Schabbat",
];

/// Typst-Quelltext für `months` Monate ab (Jahr, Monat).
pub fn typst_source(start: (i32, u8), months: u8, o: &Options) -> String {
    let first = day_from_gregorian(start.0, start.1, 1).expect("gültiger Startmonat");
    let mut starts = vec![first];
    let (mut y, mut m) = start;
    for _ in 0..months {
        m += 1;
        if m > 12 {
            m = 1;
            y += 1;
        }
        starts.push(day_from_gregorian(y, m, 1).expect("gültiger Monat"));
    }
    let last = *starts.last().unwrap() - 1;
    let days = days_between(first - 1, last + 1);
    let idx_of = |day: Day| (day - (first - 1)) as usize;
    let times: Vec<DayTimes> = (0..days.len())
        .map(|i| day_times(&days, i, &o.location, &o.zmanim))
        .collect();
    let state: HashMap<Day, &str> = german::holidays_between(first, last, o.region)
        .into_iter()
        .collect();
    let tz = o.location.time_zone;
    let foto = o.month_style == MonthStyle::Foto;
    let uhr = !foto;

    let s = o.paper.scale();
    let mut out = o.style.preamble(&o.paper);
    if !foto {
        // Platz für die senkrechte Legende rechts
        out.push_str(
            "#set page(margin: (left: 6mm * S, right: 11mm * S, top: 6mm * S, bottom: 6mm * S))\n",
        );
    }
    out.push_str(r##"#let OLIVE = rgb("#5b6b2e")
#let GREEN = rgb("#1f6a3a")
#let YELLOW = rgb("#e3c84a")
#let ev-item(e) = {
  let (t, k) = e
  if k == "b" { text(size: 5.6pt * S * F, weight: "bold")[#t] }
  else if k == "i" { text(size: 4.8pt * S * F, style: "italic", fill: luma(60))[#t] }
  else if k == "c" { text(size: 4.8pt * S * F, weight: "bold", fill: OLIVE)[#t] }
  else { text(size: 5.4pt * S * F)[#t] }
}
#let time-item(e, size, bold) = {
  let (ic, t) = e
  box(baseline: 25%, image("icon-" + ic + ".svg", height: size * 1.25))
  h(0.6mm * S)
  text(size: size, weight: if bold { "bold" } else { "regular" }, style: if bold { "italic" } else { "normal" })[#t]
}
"##);

    out.push_str(CELL_FNS);

    // Höhen (Millimeter, bezogen auf die tatsächliche Seite)
    let page_h = o.paper.height_mm;
    let usable = page_h - 12.0 * s;
    let (photo_h, title_h, wd_h) = if foto {
        (usable * 0.40, 13.0 * s, 6.0 * s)
    } else {
        (0.0, 15.0 * s, 8.0 * s)
    };

    for (mi, w) in starts.windows(2).enumerate() {
        let (a, b) = (w[0], w[1]);
        if mi > 0 {
            out.push_str("#pagebreak()\n");
        }
        let (gy, gm, _) = cc_calendar::gregorian_from_day(a);
        let month_title = format!("{} {}", GREG_MONTHS[usize::from(gm) - 1], gy);
        let month_days: Vec<&DayInfo> = (a..b).map(|d| &days[idx_of(d)]).collect();
        let hebr = hebrew_months(&month_days, if foto { " - " } else { " / " });

        // Wochen: erste Zeile beginnt am Sonntag vor/auf dem 1.
        let grid_start = a - i64::from(weekday(a));
        let weeks = ((b - grid_start) as f64 / 7.0).ceil() as i64;
        let grid_h = usable - photo_h - title_h - wd_h - if foto { 3.0 * s } else { 1.0 * s };
        let gap = if foto { 1.6 * s } else { 0.0 };
        let row_h = (grid_h - gap * (weeks as f64 - 1.0)) / weeks as f64;

        if foto {
            let n = o.photos_per_month.max(1);
            let photos: Vec<&String> = o.photos.iter().skip(mi * n).take(n).collect();
            let cols = match n {
                1 => 1,
                2 | 4 => 2,
                _ => 3,
            };
            let rows = n.div_ceil(cols);
            let mut cells = Vec::new();
            for k in 0..n {
                cells.push(match photos.get(k) {
                    Some(p) => format!(
                        "block(width: 100%, height: 100%, stroke: 1.2pt * S + YELLOW, clip: true, image({}, width: 100%, height: 100%, fit: \"cover\"))",
                        q(p)
                    ),
                    None => "block(width: 100%, height: 100%, stroke: 1.2pt * S + YELLOW, fill: luma(225), align(center + horizon, text(size: 9pt * S, fill: luma(120))[Foto]))".into(),
                });
            }
            let _ = writeln!(
                out,
                "#block(width: 100%, height: {photo_h:.2}mm, fill: GREEN, inset: 4mm * S, grid(columns: (1fr,) * {cols}, rows: (1fr,) * {rows}, gutter: 3mm * S, {}))",
                cells.join(", ")
            );
            let _ = writeln!(
                out,
                "#block(width: 100%, height: {title_h:.2}mm, inset: (x: 1mm * S), grid(columns: (1fr, auto), align(horizon + left, text(size: 24pt * S * F, fill: luma(40))[#{}]), align(horizon + right, text(size: 12pt * S * F, fill: luma(40))[#{}])))",
                q(&month_title),
                q(&hebr)
            );
        } else {
            let _ = writeln!(
                out,
                "#block(width: 100%, height: {title_h:.2}mm, align(horizon + left, {{ text(font: HFONT, size: 22pt * S * F, weight: \"bold\", style: \"italic\")[#{}]; h(2mm * S); text(font: HFONT, size: 12pt * S * F, style: \"italic\")[#{}] }}))",
                q(&month_title),
                q(&hebr)
            );
        }
        // Wochentage
        let wd: Vec<String> = WEEKDAYS
            .iter()
            .map(|w| {
                if foto {
                    format!("align(center + horizon, text(size: 8pt * S * F, fill: luma(90))[{w}])")
                } else {
                    format!("align(center + bottom, text(font: HFONT, size: 11pt * S * F, weight: \"bold\", style: \"italic\", fill: OLIVE)[{w}])")
                }
            })
            .collect();
        let _ = writeln!(
            out,
            "#block(width: 100%, height: {wd_h:.2}mm, grid(columns: (1fr,) * 7, {}))",
            wd.join(", ")
        );
        if foto {
            out.push_str("#v(1mm * S)\n");
        }

        // Zellen
        let mut cells: Vec<String> = Vec::new();
        for k in 0..weeks * 7 {
            let day = grid_start + k;
            if day < a || day >= b {
                cells.push(if foto {
                    "[]".into()
                } else {
                    format!("block(width: 100%, height: {row_h:.3}mm, stroke: 0.4pt + luma(170))")
                });
                continue;
            }
            let i = idx_of(day);
            let d = &days[i];
            let t = &times[i];
            let dst = tz.offset_minutes(day, 0.0) != tz.offset_minutes(day + 1, 0.0);
            let ev = cell_events(d, t, o, state.get(&day).copied(), dst);
            let tm = cell_times(d, t, uhr, foto);
            let hd = format!(
                "{}{} {}",
                d.hebrew.day,
                if foto { "" } else { "." },
                month_name_de(&d.hebrew.month_code, d.hebrew.leap_year)
            );
            if foto {
                cells.push(format!(
                    "cellf(h: {row_h:.3}mm, gd: {}, hd: {}, evs: {}, times: {})",
                    q(&d.gregorian.2.to_string()),
                    q(&hd),
                    typst_array_ev(&ev),
                    typst_array_times(&tm)
                ));
            } else {
                let grey = d.weekday == SATURDAY || d.yom_tov;
                cells.push(format!(
                    "cellk(h: {row_h:.3}mm, grey: {grey}, gd: {}, hd: {}, hn: {}, evs: {}, times: {})",
                    q(&d.gregorian.2.to_string()),
                    q(&hd),
                    q(&hebrew_numeral(u32::from(d.hebrew.day), false)),
                    typst_array_ev(&ev),
                    typst_array_times(&tm)
                ));
            }
        }
        let _ = writeln!(
            out,
            "#grid(columns: (1fr,) * 7, column-gutter: {}, row-gutter: {gap:.2}mm, {})",
            if foto { "1.6mm * S" } else { "0mm" },
            cells.join(",\n")
        );
        if !foto {
            out.push_str(LEGEND_KLASSISCH);
        }
    }

    out
}

const CELL_FNS: &str = r##"#let cellk(h: 10mm, grey: false, gd: "", hd: "", hn: "", evs: (), times: ()) = block(
  width: 100%, height: h, fill: if grey { luma(232) } else { luma(249) }, stroke: 0.4pt + luma(170),
  inset: (left: 0.8mm * S, right: 1.2mm * S, top: 1.2mm * S, bottom: 1.2mm * S), clip: true, {
  place(top + left, rotate(-90deg, reflow: true, text(size: 4.6pt * S * F, fill: luma(110))[#hd]))
  place(top + left, dx: 3.4mm * S, stack(dir: ttb, spacing: 0.8mm * S,
    text(size: 15pt * S * F, weight: "bold")[#gd],
    text(size: 9pt * S * F, lang: "he", fill: luma(30))[#hn]))
  place(top + right, block(width: 64%, align(right, {
    set par(leading: 0.25em)
    evs.map(ev-item).join(linebreak())
  })))
  place(bottom + left, stack(dir: ttb, spacing: 0.6mm * S, ..times.map(e => time-item(e, 5.4pt * S * F, false))))
})
#let cellf(h: 10mm, gd: "", hd: "", evs: (), times: ()) = block(width: 100%, height: h, clip: true, fill: luma(248), stroke: 0.4pt + luma(175), {
  block(width: 100%, height: 3.6mm * S, fill: luma(208), inset: (x: 1mm * S), align(horizon + left, text(size: 5.6pt * S * F, fill: luma(45))[#hd]))
  pad(x: 1mm * S, {
    v(0.8mm * S)
    text(size: 17pt * S * F, fill: luma(80))[#gd]
    linebreak()
    set par(leading: 0.25em)
    evs.map(ev-item).join(linebreak())
  })
  place(bottom + right, dx: -1mm * S, dy: -0.6mm * S, stack(dir: ttb, spacing: 0.6mm * S, ..times.map(e => align(right, time-item(e, 6pt * S * F, true)))))
})
"##;

const LEGEND_KLASSISCH: &str = r##"#place(top + right, dx: 9.5mm * S, dy: 24mm * S, rotate(-90deg, reflow: true, {
  set text(size: 5.4pt * S * F, fill: luma(40))
  box(baseline: 25%, image("icon-candle.svg", height: 3mm * S)); h(1mm * S); [Kerzenzünden]; h(5mm * S)
  box(baseline: 25%, image("icon-end.svg", height: 3mm * S)); h(1mm * S); [Schabbat & Feiertag endet]; h(5mm * S)
  box(baseline: 25%, image("icon-fast.svg", height: 3mm * S)); h(1mm * S); [Fastenbeginn / Fastenende]
}))
"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oktober_2026_like_example() {
        let o = Options::new(MonthStyle::Klassisch);
        let src = typst_source((2026, 10), 1, &o);
        // Schemini Azeret am Schabbat, Kerzen der zweiten Nacht ab 19:48
        assert!(src.contains("(\"Schemini Azeret\", \"b\")"));
        assert!(src.contains("(\"candle\", \"Ab 19:48 Uhr\")"));
        assert!(src.contains("(\"Mewarchim\", \"n\")"));
        assert!(src.contains("Gedenktag an das Massaker vom 7. Oktober"));
        assert!(src.contains("Zeitumstellung"));
        // Hebräischer Tag in Buchstaben: 3.10.2026 = 22. Tischri = כב
        assert!(src.contains("hn: \"כב\""));
        // 5 Wochenzeilen à 7 Zellen
        assert_eq!(
            src.matches("cellk(").count() + src.matches("stroke: 0.4pt + luma(170))").count() - 1,
            35
        );
    }
}
