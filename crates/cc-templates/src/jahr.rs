//! Vorlage "Jahreskalender" im Stil gängiger Druckkalender (z. B.
//! kalenderpedia.de): gregorianische Monate als Spalten, eine Zeile je Tag,
//! Kalenderwochen und gesetzliche Feiertage, dazu hebräisches Datum, jüdische
//! Feiertage, Parascha und Zeiten.

use std::fmt::Write as _;

use cc_astro::{Location, Profile};
use cc_calendar::{Day, MONDAY, SUNDAY, day_from_gregorian, iso_week};
use cc_holidays::{DayInfo, days_between, german};

use crate::common::{
    DAY_FILL, EREV_FILL, GREG_MONTHS, GREY_FILL, LEGEND, Paper, SHABBAT_FILL, Style, TYPST_TIME_FN,
    WD_DE, YOM_TOV_FILL, jewish_row, month_name_de, pessach_footnote, q,
};
use crate::gemeinde::footer;
use crate::times::{DayTimes, day_times};

#[derive(Debug, Clone)]
pub struct Options {
    pub paper: Paper,
    pub style: Style,
    pub location: Location,
    pub zmanim: Profile,
    /// Erster Monat (1–12) und Anzahl der Monate (Standard: Januar, 12)
    pub start_month: u8,
    pub months: u8,
    /// Seiten: 1 (alle Monate nebeneinander) oder 2 (je Hälfte)
    pub pages: u8,
    /// Überschrift links oben, z. B. "Jüdischer Kalender"
    pub title: String,
    /// Bundesland für die gesetzlichen Feiertage
    pub region: german::State,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            paper: Paper::A4,
            style: Style::default(),
            location: Location::frankfurt(),
            zmanim: Profile::frankfurt(),
            start_month: 1,
            months: 12,
            region: german::State::Hessen,
            pages: 2,
            title: "Jüdischer Kalender".into(),
        }
    }
}

const STATE_HOLIDAY_FILL: &str = "#f6d6d6";
const SUNDAY_FILL: &str = "#f1f1f1";

fn fill(d: &DayInfo, state_holiday: bool) -> Option<&'static str> {
    let has = |c: cc_holidays::Class| d.holidays.iter().any(|h| h.class == c);
    use cc_holidays::Class::*;
    if d.is_shabbat() {
        Some(SHABBAT_FILL)
    } else if d.yom_tov {
        Some(YOM_TOV_FILL)
    } else if state_holiday {
        Some(STATE_HOLIDAY_FILL)
    } else if has(CholHamoed) || has(Chanukka) || has(Purim) {
        Some(GREY_FILL)
    } else if has(Erev) {
        Some(EREV_FILL)
    } else if d.weekday == SUNDAY {
        Some(SUNDAY_FILL)
    } else {
        None
    }
}

fn hebrew_date(d: &DayInfo) -> String {
    format!(
        "{}. {}",
        d.hebrew.day,
        month_name_de(&d.hebrew.month_code, d.hebrew.leap_year)
    )
}

/// "Tewet – Schwat 5787" bzw. "Elul 5787 – Tischrei 5788"
fn hebrew_months(days: &[&DayInfo]) -> String {
    let mut parts: Vec<(i32, &'static str)> = Vec::new();
    for d in days {
        let m = month_name_de(&d.hebrew.month_code, d.hebrew.leap_year);
        if parts.last() != Some(&(d.hebrew.year, m)) {
            parts.push((d.hebrew.year, m));
        }
    }
    let (y0, m0) = parts[0];
    let (y1, m1) = *parts.last().unwrap();
    if parts.len() == 1 {
        format!("{m0} {y0}")
    } else if y0 == y1 {
        format!("{m0} – {m1} {y0}")
    } else {
        format!("{m0} {y0} – {m1} {y1}")
    }
}

fn row(d: &DayInfo, t: &DayTimes, state_holiday: Option<&str>) -> String {
    let j = jewish_row(d, t);
    let mut top = if j.rc.is_empty() {
        hebrew_date(d)
    } else {
        format!("{} {}", j.rc, hebrew_date(d))
    };
    if d.weekday == MONDAY {
        top.push_str(&format!("  ·  KW {}", iso_week(d.day).1));
    }
    let mut sub = j.sub.clone();
    if !j.omer.is_empty() {
        sub = if sub.is_empty() {
            j.omer.clone()
        } else {
            format!("{} · {sub}", j.omer)
        };
    }
    let fill = format!(
        "rgb({})",
        q(fill(d, state_holiday.is_some()).unwrap_or(DAY_FILL))
    );
    format!(
        "rj(fill: {fill}, gd: {}, wd: {}, top: {}, topred: {}, name: {}, red: {}, sub: {}, time: {}, tk: {}, candle: {}, ctr: {}, bold: {}, gdred: {})\n",
        q(&d.gregorian.2.to_string()),
        q(WD_DE[usize::from(d.weekday)]),
        q(&top),
        q(&j.top),
        q(&j.name),
        q(state_holiday.unwrap_or("")),
        q(&sub),
        q(&j.time),
        q(j.tk),
        j.candle,
        j.centered && state_holiday.is_none(),
        j.bold,
        state_holiday.is_some(),
    )
}

/// Typst-Quelltext des Jahreskalenders für das gregorianische Jahr `year`.
pub fn typst_source(year: i32, o: &Options) -> String {
    // Zeitraum: start_month/year bis zum Ende des letzten Monats
    let first = day_from_gregorian(year, o.start_month, 1).expect("gültiger Startmonat");
    let mut month_starts = Vec::new();
    let (mut y, mut m) = (year, o.start_month);
    for _ in 0..=o.months {
        month_starts.push(day_from_gregorian(y, m, 1).expect("gültiger Monat"));
        m += 1;
        if m > 12 {
            m = 1;
            y += 1;
        }
    }
    let last = *month_starts.last().unwrap() - 1;
    // Ein Tag Vorlauf für die Zeiten am letzten Tag
    let days = days_between(first, last + 1);
    let times: Vec<DayTimes> = (0..days.len())
        .map(|i| day_times(&days, i, &o.location, &o.zmanim))
        .collect();
    let state: std::collections::HashMap<Day, &str> =
        german::holidays_between(first, last, o.region)
            .into_iter()
            .collect();

    let s = o.paper.scale();
    let (header_h, colhead_h, footer_h, gap) = (17.0, 8.0, 10.0, 1.5);
    let rows_n = 31.0;
    let row_h =
        (o.paper.height_mm - (5.0 + 4.0 + header_h + colhead_h + footer_h + gap) * s) / rows_n;

    let (y_first, y_last) = (
        days.first().map(|d| d.hebrew.year).unwrap_or(0),
        days.iter().rev().nth(1).map(|d| d.hebrew.year).unwrap_or(0),
    );
    let year_label = if o.start_month == 1 {
        year.to_string()
    } else {
        format!("{}/{:02}", year, (year + 1) % 100)
    };

    let mut out = o.style.preamble(&o.paper);
    out.push_str(TYPST_TIME_FN);
    out.push_str(&format!(
        r##"#let ROWH = {row_h:.4}mm
#let LW = 5.6mm * S
#let rj(fill: none, gd: "", wd: "", top: "", topred: "", name: "", red: "", sub: "", time: "", tk: "", candle: false, ctr: false, bold: false, gdred: false) = block(
  width: 100%, height: ROWH, fill: fill, clip: true,
  stroke: (bottom: 0.3pt * S + luma(165), right: 0.3pt * S + luma(175)), inset: (x: 0.5mm * S, y: 0pt),
  {{
    set par(leading: 0.12em, spacing: 0pt)
    grid(
      columns: (LW * F, 1fr, auto), column-gutter: 0.6mm * S,
      align(horizon + left, if gd != "" {{
        stack(dir: ttb, spacing: 0pt,
          text(size: 6.2pt * S * F, weight: "bold", fill: if gdred {{ RED }} else {{ rgb("#1b1b1b") }})[#gd],
          text(size: 4.3pt * S * F, fill: luma(80))[#wd])
      }}),
      align(horizon + if ctr {{ center }} else {{ left }}, {{
        if top != "" or topred != "" {{
          text(size: 4.2pt * S * F, fill: luma(95))[#top]
          if topred != "" {{ h(1mm * S); text(size: 4.2pt * S * F, fill: RED)[#topred] }}
          linebreak()
        }}
        if name != "" {{ text(size: if name.len() > 22 {{ 4.8pt * S * F }} else {{ 5.4pt * S * F }}, weight: if bold {{ "bold" }} else {{ "regular" }}, fill: if ctr {{ rgb("#0b2a5b") }} else {{ rgb("#1b1b1b") }})[#name] }}
        if red != "" {{ if name != "" {{ linebreak() }}; text(size: 4.8pt * S * F, weight: "bold", fill: RED)[#red] }}
        if sub != "" {{ if name != "" or red != "" {{ linebreak() }}; text(size: 4.1pt * S * F, fill: luma(60))[#sub] }}
      }}),
      align(horizon + right, timebox(time, tk, candle, 5.6pt * S * F)),
    )
  }})
#let blank() = rj()
#let header() = block(width: 100%, height: {header_h}mm * S, grid(
  columns: (1fr, auto),
  align(horizon + left, {{
    text(size: 40pt * S, weight: "bold", fill: rgb("#3b5bc4"))[{year_label}]
    h(3mm * S)
    box(baseline: -2mm * S, stack(dir: ttb, spacing: 1.2mm * S,
      text(size: 13pt * S * F, weight: "bold", fill: BLUE)[#{title}],
      text(size: 10pt * S * F, weight: "bold", fill: rgb("#b8932f"))[{y_first}{y_dash}]))
  }}),
  align(right + top, text(size: 9pt * S * F, lang: "he", fill: BLUE)[בס״ד]),
))
"##,
        title = q(&o.title),
        y_dash = if y_last != y_first { format!(" – {y_last}") } else { String::new() },
    ));

    // Monatsspalten
    let cols: Vec<(String, String, Vec<usize>)> = (0..usize::from(o.months))
        .map(|k| {
            let (a, b) = (month_starts[k], month_starts[k + 1]);
            let idx: Vec<usize> = days
                .iter()
                .enumerate()
                .filter(|(_, d)| d.day >= a && d.day < b)
                .map(|(i, _)| i)
                .collect();
            let g = days[idx[0]].gregorian;
            let refs: Vec<&DayInfo> = idx.iter().map(|&i| &days[i]).collect();
            (
                format!("{} {}", GREG_MONTHS[usize::from(g.1) - 1], g.0),
                hebrew_months(&refs),
                idx,
            )
        })
        .collect();
    let per_page = if o.pages <= 1 {
        cols.len()
    } else {
        cols.len().div_ceil(usize::from(o.pages))
    };
    let location_line = format!(
        "Zeiten für {}: Kerzenzünden {} Min. vor Sonnenuntergang, Schabbat- und Feiertagsende bei {}° unter dem Horizont · Gesetzliche Feiertage: {} · KW = Kalenderwoche",
        o.location.name,
        o.zmanim.candle_minutes,
        o.zmanim.shabbat_end_degrees.to_string().replace('.', ","),
        o.region.name()
    );
    for (pi, page) in cols.chunks(per_page).enumerate() {
        if pi > 0 {
            out.push_str("#pagebreak()\n");
        }
        let _ = writeln!(out, "#header()\n#v({gap}mm * S)");
        let _ = writeln!(
            out,
            "#grid(columns: (1fr,) * {}, column-gutter: 0mm,",
            page.len()
        );
        let mut footnote = String::new();
        for (title, sub, idx) in page {
            let _ = writeln!(out, "[");
            let _ = writeln!(out, "#colhead({}, {}, {colhead_h}mm * S)", q(title), q(sub));
            for &i in idx {
                let d = &days[i];
                let _ = write!(out, "#{}", row(d, &times[i], state.get(&d.day).copied()));
                if d.holidays.iter().any(|h| h.name_de == "Taanit Bechorot") {
                    footnote.push_str(&pessach_footnote(&times[i]));
                }
            }
            for _ in idx.len()..31 {
                let _ = writeln!(out, "#blank()");
            }
            let _ = writeln!(out, "],");
        }
        let _ = writeln!(out, ")");
        footer(
            &mut out,
            footer_h,
            &[LEGEND.to_string(), location_line.clone(), footnote],
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_rows(region: german::State) -> Vec<String> {
        let o = Options {
            region,
            ..Options::default()
        };
        typst_source(2027, &o)
            .lines()
            .filter(|l| l.contains(STATE_HOLIDAY_FILL))
            .map(|l| {
                l.split("gd: ")
                    .nth(1)
                    .unwrap_or("")
                    .chars()
                    .take(4)
                    .collect()
            })
            .collect()
    }

    #[test]
    fn region_changes_state_holidays() {
        let he = state_rows(german::State::Hessen);
        let by = state_rows(german::State::Bayern);
        eprintln!("HE {he:?}\nBY {by:?}");
        assert!(by.len() > he.len());
    }
}
