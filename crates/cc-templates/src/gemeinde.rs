//! Vorlage "Gemeindekalender": Wandkalender mit einer Spalte je jüdischem
//! Monat und einer Zeile je Tag (siehe docs/07-vorlage-gemeindekalender.md).
//!
//! Erzeugt Typst-Quelltext; Setzen und Export übernimmt `cc-render`.

use std::fmt::Write as _;

use cc_astro::{Location, Profile};
use cc_calendar::{Day, gregorian_from_day, hebrew_year_days};
use cc_holidays::{Class, DayInfo, RoshChodesh, german, month, year_days};

use crate::times::{DayTimes, MainTime, day_times, fmt};

/// Seitenformat in Millimetern, quer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Paper {
    pub width_mm: f64,
    pub height_mm: f64,
}

impl Paper {
    pub const A4: Paper = Paper {
        width_mm: 297.0,
        height_mm: 210.0,
    };
    pub const A3: Paper = Paper {
        width_mm: 420.0,
        height_mm: 297.0,
    };
    pub const A2: Paper = Paper {
        width_mm: 594.0,
        height_mm: 420.0,
    };
    pub const LETTER: Paper = Paper {
        width_mm: 279.4,
        height_mm: 215.9,
    };

    pub fn by_name(name: &str) -> Option<Paper> {
        Some(match name.to_ascii_lowercase().as_str() {
            "a4" => Self::A4,
            "a3" => Self::A3,
            "a2" => Self::A2,
            "letter" => Self::LETTER,
            _ => return None,
        })
    }
}

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
    pub profile: CommunityProfile,
    pub location: Location,
    pub zmanim: Profile,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            paper: Paper::A4,
            profile: CommunityProfile::default(),
            location: Location::frankfurt(),
            zmanim: Profile::frankfurt(),
        }
    }
}

const GREG_MONTHS: [&str; 12] = [
    "Januar",
    "Februar",
    "März",
    "April",
    "Mai",
    "Juni",
    "Juli",
    "August",
    "September",
    "Oktober",
    "November",
    "Dezember",
];
const WD_DE: [&str; 7] = ["So", "Mo", "Di", "Mi", "Do", "Fr", "Sa"];
const WD_HE: [&str; 7] = ["ראשון", "שני", "שלישי", "רביעי", "חמישי", "שישי", "שבת"];

fn month_name_de(code: &str, leap: bool) -> &'static str {
    match code {
        month::TISHREI => "Tischrei",
        month::CHESHVAN => "Cheschwan",
        month::KISLEV => "Kislew",
        month::TEVET => "Tewet",
        month::SHEVAT => "Schwat",
        month::ADAR_I => "Adar I",
        month::ADAR if leap => "Adar II",
        month::ADAR => "Adar",
        month::NISAN => "Nissan",
        month::IYAR => "Ijar",
        month::SIVAN => "Siwan",
        month::TAMMUZ => "Tammus",
        month::AV => "Aw",
        month::ELUL => "Elul",
        _ => "?",
    }
}

/// Typst-Zeichenkette mit Anführungszeichen.
fn q(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
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

fn fill_for(d: &DayInfo) -> Option<&'static str> {
    if d.is_shabbat() {
        return Some("#9fc4ec");
    }
    let has = |c: Class| d.holidays.iter().any(|h| h.class == c);
    if d.yom_tov {
        Some("#c9d8f2")
    } else if has(Class::CholHamoed) || has(Class::Chanukka) || has(Class::Purim) {
        Some("#dedede")
    } else if has(Class::Erev) {
        Some("#eceaf7")
    } else {
        None
    }
}

fn row(d: &DayInfo, t: &DayTimes, has_taanit_bechorot: bool) -> String {
    let mut names: Vec<String> = Vec::new();
    let mut sub = String::new();
    for h in &d.holidays {
        if h.name_de == "Taanit Bechorot" {
            continue;
        }
        names.push(h.name_de.clone());
    }
    let centered = d.is_shabbat() && d.parasha.is_some();
    if let Some(p) = d.parasha {
        // Schabbat: Parascha in der Mitte; Feiertage und besondere Schabbatot
        // stehen als kleine Zeile darunter
        sub = names.join(" · ");
        names = vec![p.name_de()];
    }
    if let Some(s) = d.special_shabbat {
        sub = if sub.is_empty() {
            s.to_string()
        } else {
            format!("{s} · {sub}")
        };
    }
    if d.yizkor {
        sub = if sub.is_empty() {
            "Jiskor".into()
        } else {
            format!("{sub} · Jiskor")
        };
    }
    if d.first_selichot {
        sub = if sub.is_empty() {
            "Erste Slichot".into()
        } else {
            format!("{sub} · Erste Slichot")
        };
    }
    // Zeile oben (rot, klein): Fasten- und Chametz-Zeiten
    let mut top = String::new();
    let mut add = |s: String| {
        if !top.is_empty() {
            top.push_str("  ");
        }
        top.push_str(&s);
    };
    if !has_taanit_bechorot {
        match (t.fast_begin, t.fast_end) {
            (Some(b), Some(e)) => add(format!("FB {} FE {}", fmt(b), fmt(e))),
            (Some(b), None) => add(format!("FB {}", fmt(b))),
            (None, Some(e)) => add(format!("FE {}", fmt(e))),
            _ => {}
        }
    }
    if let Some(b) = t.bdikat_chametz {
        add(format!("Bdikat Chametz {}", fmt(b)));
    }
    if let Some(c) = t.chametz_eating_ends {
        add(format!("Chametz Verbot {} *", fmt(c)));
    }
    let omer = d.omer.map(|n| format!("{n} L’O")).unwrap_or_default();
    let (time, tk, candle) = match t.main {
        Some(MainTime::Candles(m)) | Some(MainTime::SecondNightCandles(m)) => {
            (fmt(m), "over", false)
        }
        Some(MainTime::End(m)) => (fmt(m), "under", false),
        Some(MainTime::ChanukkaCandles(m)) => (fmt(m), "", true),
        None => (String::new(), "", false),
    };
    let rc = match d.rosh_chodesh {
        Some(RoshChodesh::Single) => "●",
        Some(RoshChodesh::First) => "①",
        Some(RoshChodesh::Second) => "②",
        None => "",
    };
    let bold = d.yom_tov
        || d.holidays.iter().any(|h| {
            matches!(
                h.class,
                Class::Fast | Class::Purim | Class::Minor | Class::Modern | Class::YomTov
            )
        });
    let mut name = names.join(" · ");
    if name.is_empty() && !omer.is_empty() {
        name = String::new();
    }
    let fill = fill_for(d).map_or("none".to_string(), |c| format!("rgb({})", q(c)));
    format!(
        "r(fill: {fill}, gd: {}, wd: {}, top: {}, name: {}, sub: {}, time: {}, tk: {}, candle: {}, hnum: {}, hwd: {}, rc: {}, omer: {}, ctr: {}, bold: {})\n",
        q(&d.gregorian.2.to_string()),
        q(WD_DE[usize::from(d.weekday)]),
        q(&top),
        q(&name),
        q(&sub),
        q(&time),
        q(tk),
        candle,
        q(&d.hebrew.day.to_string()),
        q(WD_HE[usize::from(d.weekday)]),
        q(rc),
        q(&omer),
        centered,
        bold,
    )
}

/// "3. Oktober, 25.–26. Dezember, 1. Januar"
fn state_holidays_text(from: Day, to: Day) -> String {
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

    let s = o.paper.height_mm / 210.0; // Maßstab gegenüber A4
    let rows_n = cols
        .iter()
        .map(|c| c.days.len())
        .max()
        .unwrap_or(30)
        .max(30) as f64;
    let (header_h, colhead_h, footer_h, gap) = (29.0, 9.0, 11.0, 1.5);
    let row_h = (210.0 - 5.0 - 4.0 - header_h - colhead_h - footer_h - gap) / rows_n;

    let (g0, _, _) = gregorian_from_day(hebrew_year_days(year).start);
    let year_label = format!("{}/{:02}", g0, (g0 + 1) % 100);

    let mut out = String::new();
    let _ = writeln!(out, "#let S = {s:.5}");
    let _ = writeln!(
        out,
        "#set page(width: {}mm, height: {}mm, margin: (x: 5.5mm * S, top: 5mm * S, bottom: 4mm * S))",
        o.paper.width_mm, o.paper.height_mm
    );
    let _ = writeln!(
        out,
        "#set text(font: (\"Arial\", \"DejaVu Sans Mono\"), size: 6pt * S, lang: \"de\", fill: rgb(\"#1b1b1b\"))"
    );
    out.push_str(&format!(
        r##"#set block(spacing: 0pt)
#let ROWH = {row_h:.4}mm * S
#let LW = 6.4mm * S
#let RW = 7.4mm * S
#let BLUE = rgb("#2a78bd")
#let RED = rgb("#d4202a")
#let r(fill: none, gd: "", wd: "", top: "", name: "", sub: "", time: "", tk: "", candle: false, hnum: "", hwd: "", rc: "", omer: "", ctr: false, bold: false) = block(
  width: 100%, height: ROWH, fill: fill, clip: true,
  stroke: (bottom: 0.12pt * S + luma(190)), inset: (x: 0.5mm * S, y: 0pt),
  {{
    set par(leading: 0.1em * S, spacing: 0pt)
    grid(
      columns: (LW, 1fr, RW), column-gutter: 0.4mm * S,
      align(horizon + left, if gd != "" {{
        stack(dir: ttb, spacing: 0pt,
          text(size: 5.8pt * S, weight: "bold")[#gd],
          text(size: 4.1pt * S, fill: luma(80))[#wd])
      }}),
      align(horizon + left, grid(
        columns: (1fr, auto), column-gutter: 0.5mm * S,
        align(horizon + if ctr {{ center }} else {{ left }}, {{
          if top != "" {{ text(size: 4.3pt * S, fill: RED)[#top]; linebreak() }}
          if omer != "" and name == "" {{ text(size: 4.3pt * S, fill: luma(90))[#omer] }}
          else if omer != "" {{ text(size: 4.1pt * S, fill: luma(90))[#omer #h(0.8mm * S)] }}
          if name != "" {{ text(size: if name.len() > 20 {{ 4.9pt * S }} else {{ 5.6pt * S }}, weight: if bold {{ "bold" }} else {{ "regular" }}, fill: if ctr {{ rgb("#0b2a5b") }} else {{ rgb("#1b1b1b") }})[#name] }}
          if sub != "" {{ linebreak(); text(size: 4.2pt * S, fill: luma(60))[#sub] }}
        }}),
        align(horizon + right, if time != "" {{
          text(size: 5.6pt * S, fill: RED)[#if candle {{ [✦ ] }}#if tk == "over" {{ box(stroke: (top: 0.45pt * S + RED), inset: (top: 0.25mm * S), time) }} else if tk == "under" {{ box(stroke: (bottom: 0.45pt * S + RED), inset: (bottom: 0.25mm * S), time) }} else {{ time }}]
        }}),
      )),
      align(horizon + right, stack(dir: ttb, spacing: 0pt,
        text(size: 5.2pt * S)[#if rc != "" {{ text(size: 5.5pt * S)[#rc ] }}#hnum],
        text(size: 4.1pt * S, lang: "he", fill: luma(80))[#hwd])),
    )
  }})
#let colhead(title, sub) = block(width: 100%, height: {colhead_h}mm * S, fill: BLUE, inset: (x: 0.8mm * S, y: 0.6mm * S),
  align(center + horizon, stack(dir: ttb, spacing: 0.6mm * S,
    text(font: ("Times New Roman", "Libertinus Serif"), size: 8.6pt * S, weight: "bold", fill: white)[#title],
    text(size: 5.4pt * S, weight: "bold", fill: white)[#sub])))
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
                "text(size: 12pt * S, weight: \"bold\", fill: BLUE)[#{}]",
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
            format!("block(width: 100%, text(size: 4.6pt * S)[#({lines},).join(linebreak())])")
        })
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&format!(
        r##"#let header() = block(width: 100%, height: {header_h}mm * S, grid(
  columns: (62mm * S, 1fr, 66mm * S),
  align(horizon + left, stack(dir: ttb, spacing: 1mm * S, {name_lines}, text(size: 11pt * S, lang: "he", fill: BLUE)[#{hebrew}])),
  align(center + horizon, {{ text(size: 86pt * S, weight: "bold", fill: rgb("#3b5bc4"))[{year}]; h(3mm * S); text(size: 17pt * S, weight: "bold", fill: rgb("#b8932f"))[{year_label}] }}),
  stack(dir: ttb, spacing: 1.5mm * S,
    align(right, text(size: 9pt * S, lang: "he", fill: BLUE)[#{mark}]),
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
    for (pi, page) in pages.iter().enumerate() {
        if pi > 0 {
            out.push_str("#pagebreak()\n");
        }
        out.push_str("#header()\n#v(");
        let _ = writeln!(out, "{gap}mm * S)");
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
            let _ = writeln!(out, "#colhead({}, {})", q(&c.title), q(&c.greg_range));
            for &i in &c.days {
                let d = &days[i];
                page_first.get_or_insert(d.day);
                page_last = Some(d.day);
                let tb = d.holidays.iter().any(|h| h.name_de == "Taanit Bechorot");
                let _ = write!(out, "#{}", row(d, &times[i], tb));
                if tb {
                    let (b, e) = (times[i].fast_begin, times[i].fast_end);
                    if let (Some(b), Some(e)) = (b, e) {
                        footnote.push_str(&format!("*  Taanit Bechorot {} – {}", fmt(b), fmt(e)));
                    }
                    if let Some(m) = times[i].seder_midnight {
                        footnote.push_str(&format!(
                            "      Mitternacht (Sederabend) {:02}:{:02}",
                            m / 60,
                            m % 60
                        ));
                    }
                    if let Some(v) = times[i].chametz_burning_ends {
                        footnote.push_str(&format!("      Chametz verbrennen {}", fmt(v)));
                    }
                }
            }
            for _ in c.days.len()..30 {
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
        let legend = "FB = Fastenbeginn, FE = Fastenende, Rosch Chodesch = ●, Rosch Chodesch 1. Tag = ①, Rosch Chodesch 2. Tag = ②, Linie über Uhrzeit = Schabbat-Beginn, Linie unter Uhrzeit = Ende, ✦ = Kerzenzünden (Chanukka)";
        let line = |s: &str| format!("text(fill: luma(40))[#{}]", q(s));
        let _ = writeln!(
            out,
            "#place(bottom + left, block(width: 100%, height: {footer_h}mm * S, {{
  set text(size: 5pt * S)
  align(bottom, stack(dir: ttb, spacing: 0.8mm * S, {}, text(weight: \"bold\", fill: luma(40))[#{}], {}))
}}))",
            line(legend),
            q(&format!("Staatliche Feiertage {label}: {}", state_holidays_text(from, to))),
            line(&footnote),
        );
    }
    out
}
