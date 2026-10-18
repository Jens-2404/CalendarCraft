//! Gemeinsames für alle Vorlagen: Papierformat, Schrift, Namen, Typst-Helfer.

use cc_holidays::{Class, DayInfo, RoshChodesh, month};

use crate::times::{DayTimes, MainTime, fmt};

/// Seitenformat in Millimetern (Breite × Höhe, wie gedruckt).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Paper {
    pub width_mm: f64,
    pub height_mm: f64,
}

impl Paper {
    /// Quer (Standard)
    pub const A4: Paper = Paper::new(297.0, 210.0);
    pub const A3: Paper = Paper::new(420.0, 297.0);
    pub const A2: Paper = Paper::new(594.0, 420.0);
    pub const A1: Paper = Paper::new(841.0, 594.0);
    pub const A0: Paper = Paper::new(1189.0, 841.0);
    pub const A5: Paper = Paper::new(210.0, 148.0);
    pub const LETTER: Paper = Paper::new(279.4, 215.9);
    pub const LEGAL: Paper = Paper::new(355.6, 215.9);
    pub const TABLOID: Paper = Paper::new(431.8, 279.4);

    pub const fn new(width_mm: f64, height_mm: f64) -> Self {
        Self {
            width_mm,
            height_mm,
        }
    }

    /// Format nach Name, quer: a0–a5, letter, legal, tabloid, oder "BxH" in mm.
    pub fn by_name(name: &str) -> Option<Paper> {
        let n = name.to_ascii_lowercase();
        Some(match n.as_str() {
            "a0" => Self::A0,
            "a1" => Self::A1,
            "a2" => Self::A2,
            "a3" => Self::A3,
            "a4" => Self::A4,
            "a5" => Self::A5,
            "letter" => Self::LETTER,
            "legal" => Self::LEGAL,
            "tabloid" => Self::TABLOID,
            _ => {
                let (w, h) = n.split_once('x')?;
                Paper::new(w.trim().parse().ok()?, h.trim().parse().ok()?)
            }
        })
    }

    /// Hochformat: kurze Seite als Breite.
    pub fn portrait(self) -> Self {
        Paper::new(
            self.width_mm.min(self.height_mm),
            self.width_mm.max(self.height_mm),
        )
    }

    pub fn is_portrait(&self) -> bool {
        self.height_mm > self.width_mm
    }

    /// Maßstab gegenüber A4 quer, bezogen auf die kürzere Seite.
    pub fn scale(&self) -> f64 {
        self.width_mm.min(self.height_mm) / 210.0
    }
}

/// Schrift und Schriftgröße.
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    /// Schriftfamilie für den Kalenderinhalt
    pub font: String,
    /// Schriftfamilie für Monatsköpfe
    pub heading_font: String,
    /// Faktor für alle Schriftgrößen (1.0 = Standard)
    pub font_scale: f64,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            font: "Arial".into(),
            heading_font: "Times New Roman".into(),
            font_scale: 1.0,
        }
    }
}

impl Style {
    /// Typst-Präambel mit Schriftangaben: setzt `S` (Maßstab) und `F`
    /// (Schriftfaktor) sowie die Grundschrift.
    pub fn preamble(&self, paper: &Paper) -> String {
        format!(
            "#let S = {s:.5}\n#let F = {f:.3}\n#let FONT = ({font}, \"DejaVu Sans Mono\")\n#let HFONT = ({hfont}, \"Libertinus Serif\")\n\
             #set page(width: {w}mm, height: {h}mm, margin: (x: 5.5mm * S, top: 5mm * S, bottom: 4mm * S))\n\
             #set text(font: FONT, size: 6pt * S * F, lang: \"de\", fill: rgb(\"#1b1b1b\"))\n\
             #set block(spacing: 0pt)\n",
            s = paper.scale(),
            f = self.font_scale,
            font = q(&self.font),
            hfont = q(&self.heading_font),
            w = paper.width_mm,
            h = paper.height_mm,
        )
    }
}

pub const GREG_MONTHS: [&str; 12] = [
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
pub const WD_DE: [&str; 7] = ["So", "Mo", "Di", "Mi", "Do", "Fr", "Sa"];
pub const WD_HE: [&str; 7] = ["ראשון", "שני", "שלישי", "רביעי", "חמישי", "שישי", "שבת"];

pub fn month_name_de(code: &str, leap: bool) -> &'static str {
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
pub fn q(s: &str) -> String {
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

pub const SHABBAT_FILL: &str = "#9fc4ec";
pub const YOM_TOV_FILL: &str = "#c9d8f2";
pub const GREY_FILL: &str = "#dedede";
pub const EREV_FILL: &str = "#eceaf7";

/// Hintergrund einer Zeile nach jüdischem Tagestyp.
pub fn jewish_fill(d: &DayInfo) -> Option<&'static str> {
    if d.is_shabbat() {
        return Some(SHABBAT_FILL);
    }
    let has = |c: Class| d.holidays.iter().any(|h| h.class == c);
    if d.yom_tov {
        Some(YOM_TOV_FILL)
    } else if has(Class::CholHamoed) || has(Class::Chanukka) || has(Class::Purim) {
        Some(GREY_FILL)
    } else if has(Class::Erev) {
        Some(EREV_FILL)
    } else {
        None
    }
}

/// Die jüdischen Texte einer Zeile, für alle Vorlagen gleich.
#[derive(Debug, Clone, Default)]
pub struct JewishRow {
    /// Hauptname: Parascha am Schabbat, sonst Feiertage
    pub name: String,
    /// Kleine Zeile darunter: besondere Schabbatot, Feiertage am Schabbat, Jiskor
    pub sub: String,
    /// Kleine rote Zeile darüber: Fasten- und Chametz-Zeiten
    pub top: String,
    /// Omerzählung, z. B. "12 L’O"
    pub omer: String,
    pub time: String,
    /// "over" (Beginn), "under" (Ende) oder ""
    pub tk: &'static str,
    pub candle: bool,
    /// Rosch-Chodesch-Zeichen
    pub rc: &'static str,
    /// Name zentriert (Schabbat mit Parascha)
    pub centered: bool,
    pub bold: bool,
}

pub fn jewish_row(d: &DayInfo, t: &DayTimes) -> JewishRow {
    let taanit_bechorot = d.holidays.iter().any(|h| h.name_de == "Taanit Bechorot");
    let mut names: Vec<String> = d
        .holidays
        .iter()
        .filter(|h| h.name_de != "Taanit Bechorot")
        .map(|h| h.name_de.clone())
        .collect();
    let mut subs: Vec<String> = Vec::new();
    let centered = d.is_shabbat() && d.parasha.is_some();
    if let Some(p) = d.parasha {
        // Schabbat: Parascha in der Mitte, Feiertage als kleine Zeile darunter
        if !names.is_empty() {
            subs.push(names.join(" · "));
        }
        names = vec![p.name_de()];
    }
    if let Some(s) = d.special_shabbat {
        subs.insert(0, s.to_string());
    }
    if d.yizkor {
        subs.push("Jiskor".into());
    }
    if d.first_selichot {
        subs.push("Erste Slichot".into());
    }
    let mut top: Vec<String> = Vec::new();
    if !taanit_bechorot {
        match (t.fast_begin, t.fast_end) {
            (Some(b), Some(e)) => top.push(format!("FB {} FE {}", fmt(b), fmt(e))),
            (Some(b), None) => top.push(format!("FB {}", fmt(b))),
            (None, Some(e)) => top.push(format!("FE {}", fmt(e))),
            _ => {}
        }
    }
    if let Some(b) = t.bdikat_chametz {
        top.push(format!("Bdikat Chametz {}", fmt(b)));
    }
    if let Some(c) = t.chametz_eating_ends {
        top.push(format!("Chametz Verbot {} *", fmt(c)));
    }
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
    JewishRow {
        name: names.join(" · "),
        sub: subs.join(" · "),
        top: top.join("  "),
        omer: d.omer.map(|n| format!("{n} L’O")).unwrap_or_default(),
        time,
        tk,
        candle,
        rc,
        centered,
        bold,
    }
}

/// Fußnote für Erew Pessach (Taanit Bechorot, Mitternacht, Chametz).
pub fn pessach_footnote(t: &DayTimes) -> String {
    let mut s = String::new();
    if let (Some(b), Some(e)) = (t.fast_begin, t.fast_end) {
        s.push_str(&format!("*  Taanit Bechorot {} – {}", fmt(b), fmt(e)));
    }
    if let Some(m) = t.seder_midnight {
        s.push_str(&format!(
            "      Mitternacht (Sederabend) {:02}:{:02}",
            m / 60,
            m % 60
        ));
    }
    if let Some(v) = t.chametz_burning_ends {
        s.push_str(&format!("      Chametz verbrennen {}", fmt(v)));
    }
    s
}

pub const LEGEND: &str = "FB = Fastenbeginn, FE = Fastenende, Rosch Chodesch = ●, Rosch Chodesch 1. Tag = ①, Rosch Chodesch 2. Tag = ②, Linie über Uhrzeit = Schabbat-Beginn, Linie unter Uhrzeit = Ende, ✦ = Kerzenzünden (Chanukka)";

/// Gemeinsame Typst-Funktion für die Zeitangabe rechts (rot, mit Linie).
pub const TYPST_TIME_FN: &str = r##"#let RED = rgb("#d4202a")
#let BLUE = rgb("#2a78bd")
#let timebox(time, tk, candle, size) = if time != "" {
  text(size: size, fill: RED)[#if candle [✦ ]#if tk == "over" { box(stroke: (top: 0.45pt * S + RED), inset: (top: 0.25mm * S), time) } else if tk == "under" { box(stroke: (bottom: 0.45pt * S + RED), inset: (bottom: 0.25mm * S), time) } else { time }]
}
#let colhead(title, sub, h) = block(width: 100%, height: h, fill: BLUE, inset: (x: 0.8mm * S, y: 0.5mm * S),
  align(center + horizon, stack(dir: ttb, spacing: 0.5mm * S,
    text(font: HFONT, size: 8.6pt * S * F, weight: "bold", fill: white)[#title],
    text(size: 5.2pt * S * F, weight: "bold", fill: white)[#sub])))
"##;
