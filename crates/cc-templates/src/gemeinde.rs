//! Vorlage "Gemeindekalender": Wandkalender mit einer Spalte je jüdischem
//! Monat und einer Zeile je Tag (siehe docs/07-vorlage-gemeindekalender.md).
//!
//! Erzeugt Typst-Quelltext; Setzen und Export übernimmt `cc-render`.

use std::fmt::Write as _;

use cc_astro::{Location, Profile};
use cc_calendar::{Day, gregorian_from_day, hebrew_year_days};
use cc_holidays::{DayInfo, german, year_days};

use crate::common::{
    DAY_FILL, GREG_MONTHS, LEGEND, TYPST_TIME_FN, WD_DE, WD_HE, jewish_fill, jewish_row,
    month_name_de, pessach_footnote, q,
};
pub use crate::common::{Paper, Style};
use crate::times::{DayTimes, day_times};

/// Texte des Kopfbereichs (Gemeindeprofil).
#[derive(Debug, Clone, PartialEq)]
pub struct CommunityProfile {
    /// Name der Gemeinde, links oben (eine Zeile je Eintrag)
    pub name_lines: Vec<String>,
    /// Hebräischer Name, links oben unter dem Namen
    pub name_hebrew: String,
    /// Textblöcke rechts oben (Adresse, Kontakt)
    pub info_blocks: Vec<Vec<String>>,
    /// Kopfzeichen, Standard "בס״ד"
    pub header_mark: String,
}

impl Default for CommunityProfile {
    fn default() -> Self {
        Self {
            name_lines: vec!["Jüdischer Kalender".into()],
            name_hebrew: "לוח שנה".into(),
            info_blocks: vec![vec!["Frankfurt am Main".into()]],
            header_mark: "בס״ד".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Options {
    pub paper: Paper,
    pub style: Style,
    pub profile: CommunityProfile,
    pub location: Location,
    pub zmanim: Profile,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            paper: Paper::A4,
            style: Style::default(),
            profile: CommunityProfile::default(),
            location: Location::frankfurt(),
            zmanim: Profile::frankfurt(),
        }
    }
}

/// Eine Monatsspalte: Kopf und die Indizes der Tage.
struct Column {
    title: String,
    greg_range: String,
    days: Vec<usize>,
}

fn greg_range(days: &[DayInfo], idx: &[usize]) -> String {
    let first = days[idx[0]].gregorian;
    let last = days[*idx.last().unwrap()].gregorian;
    let m = |g: (i32, u8, u8)| GREG_MONTHS[usize::from(g.1) - 1];
    if first.0 == last.0 && first.1 == last.1 {
        format!("{} {}", m(first), first.0)
    } else if first.0 == last.0 {
        format!("{} - {} {}", m(first), m(last), first.0)
    } else {
        format!("{} {} - {} {}", m(first), first.0, m(last), last.0)
    }
}

fn columns(days: &[DayInfo]) -> Vec<Column> {
    // Gruppen nach (Jahr, Monat); der einzelne Tag am Anfang (29. Elul des
    // Vorjahres) gehört zur ersten Spalte.
    let mut groups: Vec<(i32, String, bool, Vec<usize>)> = Vec::new();
    for (i, d) in days.iter().enumerate() {
        let key = (d.hebrew.year, d.hebrew.month_code.clone());
        match groups.last_mut() {
            Some(g) if (g.0, g.1.clone()) == key => g.3.push(i),
            _ => groups.push((key.0, key.1, d.hebrew.leap_year, vec![i])),
        }
    }
    let mut cols = Vec::new();
    let mut carry: Option<(i32, String, bool, Vec<usize>)> = None;
    for g in groups {
        if g.3.len() == 1 && cols.is_empty() && carry.is_none() {
            carry = Some(g);
            continue;
        }
        let (mut idx, mut title) = (g.3.clone(), format!("{} {}", month_name_de(&g.1, g.2), g.0));
        if let Some(c) = carry.take() {
            let mut merged = c.3.clone();
            merged.extend(idx);
            idx = merged;
            title = format!("{} {} - {}", month_name_de(&c.1, c.2), c.0, title);
        }
        cols.push(Column {
            title,
            greg_range: greg_range(days, &idx),
            days: idx,
        });
    }
    cols
}

fn row(d: &DayInfo, t: &DayTimes) -> String {
    let j = jewish_row(d, t);
    let fill = format!("rgb({})", q(jewish_fill(d).unwrap_or(DAY_FILL)));
    format!(
        "r(fill: {fill}, gd: {}, wd: {}, top: {}, name: {}, sub: {}, time: {}, tk: {}, candle: {}, hnum: {}, hwd: {}, rc: {}, omer: {}, ctr: {}, bold: {})\n",
        q(&d.gregorian.2.to_string()),
        q(WD_DE[usize::from(d.weekday)]),
        q(&j.top),
        q(&j.name),
        q(&j.sub),
        q(&j.time),
        q(j.tk),
        j.candle,
        q(&d.hebrew.day.to_string()),
        q(WD_HE[usize::from(d.weekday)]),
        q(j.rc),
        q(&j.omer),
        j.centered,
        j.bold,
    )
}

/// "3. Oktober, 25.–26. Dezember, 1. Januar"
pub(crate) fn state_holidays_text(from: Day, to: Day) -> String {
    let hs = german::holidays_hesse_between(from, to);
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < hs.len() {
        let (d, _) = hs[i];
        let g = gregorian_from_day(d);
        // aufeinanderfolgende Tage im selben Monat zusammenfassen
        let mut j = i;
        while j + 1 < hs.len()
            && hs[j + 1].0 == hs[j].0 + 1
            && gregorian_from_day(hs[j + 1].0).1 == g.1
        {
            j += 1;
        }
        let last = gregorian_from_day(hs[j].0);
        let month = GREG_MONTHS[usize::from(g.1) - 1];
        parts.push(if j > i {
            format!("{}.–{}. {month}", g.2, last.2)
        } else {
            format!("{}. {month}", g.2)
        });
        i = j + 1;
    }
    parts.join(", ")
}

/// Typst-Quelltext des Gemeindekalenders für das hebräische Jahr `year`.
pub fn typst_source(year: i32, o: &Options) -> String {
    let days = year_days(year);
    let cols = columns(&days);
    let times: Vec<DayTimes> = (0..days.len())
        .map(|i| day_times(&days, i, &o.location, &o.zmanim))
        .collect();

    let s = o.paper.scale();
    let rows_n = cols
        .iter()
        .map(|c| c.days.len())
        .max()
        .unwrap_or(30)
        .max(30) as f64;
    let (header_h, colhead_h, footer_h, gap) = (29.0, 9.0, 11.0, 1.5);
    // Zeilenhöhe aus der tatsächlichen Seitenhöhe (auch im Hochformat)
    let row_h =
        (o.paper.height_mm - (5.0 + 4.0 + header_h + colhead_h + footer_h + gap) * s) / rows_n;

    let (g0, _, _) = gregorian_from_day(hebrew_year_days(year).start);
    let year_label = format!("{}/{:02}", g0, (g0 + 1) % 100);

    let mut out = o.style.preamble(&o.paper);
    out.push_str(TYPST_TIME_FN);
    out.push_str(&format!(
        r##"#let ROWH = {row_h:.4}mm
#let LW = 6.4mm * S
#let RW = 7.4mm * S
#let r(fill: none, gd: "", wd: "", top: "", name: "", sub: "", time: "", tk: "", candle: false, hnum: "", hwd: "", rc: "", omer: "", ctr: false, bold: false) = block(
  width: 100%, height: ROWH, fill: fill, clip: true,
  stroke: (bottom: 0.3pt * S + luma(165)), inset: (x: 0.5mm * S, y: 0pt),
  {{
    set par(leading: 0.1em, spacing: 0pt)
    grid(
      columns: (LW * F, 1fr, RW * F), column-gutter: 0.4mm * S,
      align(horizon + left, if gd != "" {{
        stack(dir: ttb, spacing: 0pt,
          text(size: 5.8pt * S * F, weight: "bold")[#gd],
          text(size: 4.1pt * S * F, fill: luma(80))[#wd])
      }}),
      align(horizon + left, grid(
        columns: (1fr, auto), column-gutter: 0.5mm * S,
        align(horizon + if ctr {{ center }} else {{ left }}, {{
          if top != "" {{ text(size: 4.3pt * S * F, fill: RED)[#top]; linebreak() }}
          if omer != "" and name == "" {{ text(size: 4.3pt * S * F, fill: luma(90))[#omer] }}
          else if omer != "" {{ text(size: 4.1pt * S * F, fill: luma(90))[#omer #h(0.8mm * S)] }}
          if name != "" {{ text(size: if name.len() > 20 {{ 4.9pt * S * F }} else {{ 5.6pt * S * F }}, weight: if bold {{ "bold" }} else {{ "regular" }}, fill: if ctr {{ rgb("#0b2a5b") }} else {{ rgb("#1b1b1b") }})[#name] }}
          if sub != "" {{ linebreak(); text(size: 4.2pt * S * F, fill: luma(60))[#sub] }}
        }}),
        align(horizon + right, timebox(time, tk, candle, 5.6pt * S * F)),
      )),
      align(horizon + right, stack(dir: ttb, spacing: 0pt,
        text(size: 5.2pt * S * F)[#if rc != "" {{ text(size: 5.5pt * S * F)[#rc ] }}#hnum],
        text(size: 4.1pt * S * F, lang: "he", fill: luma(80))[#hwd])),
    )
  }})
#let blank() = r()
"##
    ));

    // Kopfbereich
    let name_lines: String = o
        .profile
        .name_lines
        .iter()
        .map(|l| {
            format!(
                "text(size: 12pt * S * F, weight: \"bold\", fill: BLUE)[#{}]",
                q(l)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let info: String = o
        .profile
        .info_blocks
        .iter()
        .map(|b| {
            let lines = b.iter().map(|l| q(l)).collect::<Vec<_>>().join(", ");
            format!("block(width: 100%, text(size: 4.6pt * S * F)[#({lines},).join(linebreak())])")
        })
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&format!(
        r##"#let header() = block(width: 100%, height: {header_h}mm * S, grid(
  columns: (62mm * S, 1fr, 66mm * S),
  align(horizon + left, stack(dir: ttb, spacing: 1mm * S, {name_lines}, text(size: 11pt * S * F, lang: "he", fill: BLUE)[#{hebrew}])),
  align(center + horizon, {{ text(size: 86pt * S, weight: "bold", fill: rgb("#3b5bc4"))[{year}]; h(3mm * S); text(size: 17pt * S, weight: "bold", fill: rgb("#b8932f"))[{year_label}] }}),
  stack(dir: ttb, spacing: 1.5mm * S,
    align(right, text(size: 9pt * S * F, lang: "he", fill: BLUE)[#{mark}]),
    grid(columns: (1fr, 1fr), column-gutter: 2mm * S, {info})),
))
"##,
        hebrew = q(&o.profile.name_hebrew),
        mark = q(&o.profile.header_mark),
    ));

    // Seiten: erste Seite floor(n/2) Spalten, zweite Seite der Rest
    let first_n = cols.len() / 2;
    let pages: Vec<&[Column]> = if cols.len() > 1 {
        vec![&cols[..first_n], &cols[first_n..]]
    } else {
        vec![&cols[..]]
    };
    let rows_total = rows_n as usize;
    for (pi, page) in pages.iter().enumerate() {
        if pi > 0 {
            out.push_str("#pagebreak()\n");
        }
        let _ = writeln!(out, "#header()\n#v({gap}mm * S)");
        let _ = writeln!(
            out,
            "#grid(columns: (1fr,) * {}, column-gutter: 0mm,",
            page.len()
        );
        let mut page_first: Option<Day> = None;
        let mut page_last: Option<Day> = None;
        let mut footnote = String::new();
        for c in page.iter() {
            let _ = writeln!(out, "[");
            let _ = writeln!(
                out,
                "#colhead({}, {}, {colhead_h}mm * S)",
                q(&c.title),
                q(&c.greg_range)
            );
            for &i in &c.days {
                let d = &days[i];
                page_first.get_or_insert(d.day);
                page_last = Some(d.day);
                let _ = write!(out, "#{}", row(d, &times[i]));
                if d.holidays.iter().any(|h| h.name_de == "Taanit Bechorot") {
                    footnote.push_str(&pessach_footnote(&times[i]));
                }
            }
            for _ in c.days.len()..rows_total {
                let _ = writeln!(out, "#blank()");
            }
            let _ = writeln!(out, "],");
        }
        let _ = writeln!(out, ")");
        let (from, to) = (page_first.unwrap(), page_last.unwrap());
        let (y0, _, _) = gregorian_from_day(from);
        let (y1, _, _) = gregorian_from_day(to);
        let label = if y0 == y1 {
            y0.to_string()
        } else {
            format!("{y0}/{y1}")
        };
        footer(
            &mut out,
            footer_h,
            &[
                LEGEND.to_string(),
                format!(
                    "Staatliche Feiertage {label}: {}",
                    state_holidays_text(from, to)
                ),
                footnote,
            ],
        );
    }
    out
}

/// Fußzeile unten auf der Seite; die zweite Zeile wird fett gesetzt.
pub(crate) fn footer(out: &mut String, footer_h: f64, lines: &[String]) {
    let items: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let weight = if i == 1 { "bold" } else { "regular" };
            format!("text(weight: \"{weight}\", fill: luma(40))[#{}]", q(l))
        })
        .collect();
    let _ = writeln!(
        out,
        "#place(bottom + left, block(width: 100%, height: {footer_h}mm * S, {{\n  set text(size: 5pt * S * F)\n  align(bottom, stack(dir: ttb, spacing: 0.8mm * S, {}))\n}}))",
        items.join(", ")
    );
}
