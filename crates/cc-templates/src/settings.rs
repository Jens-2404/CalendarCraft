//! Einstellungen eines Kalenders als ein Datenobjekt. Die Oberfläche
//! bearbeitet nur dieses Objekt; `typst_source` erzeugt daraus den
//! Typst-Quelltext der gewählten Vorlage.

use serde::{Deserialize, Serialize};

use crate::common::{Paper, Style};
use crate::{events, gemeinde, jahr, monat};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalendarKind {
    /// Monatskalender, Stil klassisch (quer)
    MonatKlassisch,
    /// Monatskalender mit Fotos (hoch)
    MonatFoto,
    /// Jahreskalender, gregorianische Monate als Spalten
    Jahr,
    /// Gemeindekalender, jüdische Monate als Spalten
    Gemeinde,
}

impl CalendarKind {
    pub const ALL: [CalendarKind; 4] = [
        CalendarKind::MonatKlassisch,
        CalendarKind::MonatFoto,
        CalendarKind::Jahr,
        CalendarKind::Gemeinde,
    ];

    pub fn label(self) -> &'static str {
        match self {
            CalendarKind::MonatKlassisch => "Monatskalender (klassisch, quer)",
            CalendarKind::MonatFoto => "Monatskalender mit Fotos (hoch)",
            CalendarKind::Jahr => "Jahreskalender (Monate in Spalten)",
            CalendarKind::Gemeinde => "Gemeindekalender (jüdische Monate)",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            CalendarKind::MonatKlassisch => "monat-klassisch",
            CalendarKind::MonatFoto => "monat-foto",
            CalendarKind::Jahr => "jahr",
            CalendarKind::Gemeinde => "gemeinde",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.id() == id)
    }

    /// Standard-Ausrichtung der Vorlage
    pub fn default_portrait(self) -> bool {
        self == CalendarKind::MonatFoto
    }
}

/// Papierformate zur Auswahl (Name, Anzeige).
pub const PAPERS: [(&str, &str); 9] = [
    ("a5", "DIN A5"),
    ("a4", "DIN A4"),
    ("a3", "DIN A3"),
    ("a2", "DIN A2"),
    ("a1", "DIN A1"),
    ("a0", "DIN A0"),
    ("letter", "US Letter"),
    ("legal", "US Legal"),
    ("tabloid", "US Tabloid"),
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CalendarSettings {
    pub kind: CalendarKind,
    /// Jahr: ab 3000 jüdisch (z. B. 5787), sonst gregorianisch (z. B. 2027)
    pub year: i32,
    /// Erster Monat bei gregorianischem Jahr (1–12)
    pub start_month: u8,
    /// Anzahl der Monate (Monatskalender)
    pub months: u8,
    /// Seiten des Jahreskalenders (1 oder 2)
    pub pages: u8,
    pub paper: String,
    pub portrait: bool,
    pub font: String,
    pub heading_font: String,
    pub font_scale: f64,
    /// Überschrift (Jahreskalender) bzw. Name der Gemeinde (Gemeindekalender)
    pub title: String,
    pub name_hebrew: String,
    /// Adresse/Kontakt, Blöcke durch Leerzeile getrennt
    pub info: String,
    /// Eigene Einträge (Format siehe `events`)
    pub events_text: String,
    pub photos_per_month: u8,
}

impl Default for CalendarSettings {
    fn default() -> Self {
        Self {
            kind: CalendarKind::MonatKlassisch,
            year: 5787,
            start_month: 1,
            months: 13,
            pages: 2,
            paper: "a4".into(),
            portrait: false,
            font: "Libertinus Serif".into(),
            heading_font: "Libertinus Serif".into(),
            font_scale: 1.0,
            title: "Jüdischer Kalender".into(),
            name_hebrew: "לוח שנה".into(),
            info: "Frankfurt am Main".into(),
            events_text: String::new(),
            photos_per_month: 6,
        }
    }
}

impl CalendarSettings {
    pub fn paper(&self) -> Paper {
        let p = Paper::by_name(&self.paper).unwrap_or(Paper::A4);
        let landscape = Paper::new(p.width_mm.max(p.height_mm), p.width_mm.min(p.height_mm));
        if self.portrait {
            landscape.portrait()
        } else {
            landscape
        }
    }

    pub fn style(&self) -> Style {
        Style {
            font: self.font.clone(),
            heading_font: self.heading_font.clone(),
            font_scale: self.font_scale.clamp(0.5, 3.0),
        }
    }

    /// (Jahr, Monat) des ersten Monats eines Monatskalenders.
    pub fn first_month(&self) -> (i32, u8) {
        if self.year >= 3000 {
            let rh = cc_calendar::rosh_hashana(self.year);
            let (y, m, _) = cc_calendar::gregorian_from_day(rh);
            (y, m)
        } else {
            (self.year, self.start_month.clamp(1, 12))
        }
    }
}

/// Typst-Quelltext für die Einstellungen. `photos` sind die Dateinamen der
/// Fotos in der Reihenfolge, in der sie auf die Monate verteilt werden.
pub fn typst_source(s: &CalendarSettings, photos: &[String]) -> Result<String, String> {
    let paper = s.paper();
    let style = s.style();
    match s.kind {
        CalendarKind::MonatKlassisch | CalendarKind::MonatFoto => {
            let ms = if s.kind == CalendarKind::MonatFoto {
                monat::MonthStyle::Foto
            } else {
                monat::MonthStyle::Klassisch
            };
            let mut o = monat::Options::new(ms);
            o.paper = paper;
            o.style = style;
            o.photos = photos.to_vec();
            o.photos_per_month = usize::from(s.photos_per_month.max(1));
            let start = s.first_month();
            let months = s.months.clamp(1, 24);
            if !s.events_text.trim().is_empty() {
                let ev = events::parse(&s.events_text)?;
                let first = cc_calendar::day_from_gregorian(start.0, start.1, 1)
                    .map_err(|e| e.to_string())?;
                let days = cc_holidays::days_between(first - 1, first + 31 * i64::from(months) + 1);
                o.events = events::resolve(&ev, &days);
            }
            Ok(monat::typst_source(start, months, &o))
        }
        CalendarKind::Jahr => {
            if s.year >= 3000 {
                return Err(
                    "Der Jahreskalender braucht ein gregorianisches Jahr, z. B. 2027.".into(),
                );
            }
            let o = jahr::Options {
                paper,
                style,
                pages: s.pages.clamp(1, 2),
                start_month: s.start_month.clamp(1, 12),
                title: s.title.clone(),
                ..jahr::Options::default()
            };
            Ok(jahr::typst_source(s.year, &o))
        }
        CalendarKind::Gemeinde => {
            if s.year < 3000 {
                return Err("Der Gemeindekalender braucht ein jüdisches Jahr, z. B. 5787.".into());
            }
            let info_blocks: Vec<Vec<String>> = s
                .info
                .split("\n\n")
                .map(|b| {
                    b.lines()
                        .map(str::trim)
                        .filter(|l| !l.is_empty())
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                })
                .filter(|b| !b.is_empty())
                .collect();
            let o = gemeinde::Options {
                paper,
                style,
                profile: gemeinde::CommunityProfile {
                    name_lines: s
                        .title
                        .lines()
                        .map(str::trim)
                        .filter(|l| !l.is_empty())
                        .map(str::to_string)
                        .collect(),
                    name_hebrew: s.name_hebrew.clone(),
                    info_blocks,
                    header_mark: "בס״ד".into(),
                },
                ..gemeinde::Options::default()
            };
            Ok(gemeinde::typst_source(s.year, &o))
        }
    }
}

/// Vorgeschlagener Dateiname für das PDF.
pub fn file_name(s: &CalendarSettings) -> String {
    let kind = match s.kind {
        CalendarKind::MonatKlassisch => "Monatskalender",
        CalendarKind::MonatFoto => "Fotokalender",
        CalendarKind::Jahr => "Jahreskalender",
        CalendarKind::Gemeinde => "Gemeindekalender",
    };
    format!("{kind}_{}.pdf", s.year)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_kinds_build() {
        for kind in CalendarKind::ALL {
            let mut s = CalendarSettings {
                kind,
                ..Default::default()
            };
            if kind == CalendarKind::Jahr {
                s.year = 2027;
            }
            let src = typst_source(&s, &[]).unwrap();
            assert!(src.contains("#set page"), "{kind:?}");
        }
    }

    #[test]
    fn settings_roundtrip_json() {
        let s = CalendarSettings::default();
        let json = serde_json::to_string(&s).unwrap();
        let back: CalendarSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
        // Fehlende Felder bekommen Standardwerte
        let partial: CalendarSettings = serde_json::from_str(r#"{"year": 2027}"#).unwrap();
        assert_eq!(partial.year, 2027);
        assert_eq!(partial.paper, "a4");
    }

    #[test]
    fn wrong_year_type_is_explained() {
        let s = CalendarSettings {
            kind: CalendarKind::Jahr,
            year: 5787,
            ..Default::default()
        };
        assert!(
            typst_source(&s, &[])
                .unwrap_err()
                .contains("gregorianisches Jahr")
        );
    }
}
