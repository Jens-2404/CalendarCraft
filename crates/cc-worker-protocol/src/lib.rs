//! Nachrichten zwischen Oberfläche und Satz-Worker (ADR-002).
//!
//! Die Oberfläche (cc-ui) enthält kein Typst. Sie schickt Aufträge an den
//! Web Worker (apps/worker), der den Kalender setzt. Jede Nachricht ist ein
//! JavaScript-Objekt `{ msg, data }`: `msg` ist ein [`Envelope`] als JSON,
//! `data` eine Liste von `Uint8Array` für Binärdaten (Schriften, Fotos, PDF).
//! So werden große Dateien übertragen statt in JSON kodiert.
//!
//! Der Worker beantwortet jeden Auftrag mit derselben `id`. Er arbeitet die
//! Aufträge nacheinander ab.

use cc_templates::settings::CalendarSettings;
use serde::{Deserialize, Serialize};

/// Dateiname des Worker-Skripts (von Trunk ohne Prüfsumme erzeugt, siehe
/// apps/web/index.html).
pub const WORKER_SCRIPT: &str = "./cc-worker_loader.js";

/// Auftrag der Oberfläche.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {
    /// Stand abfragen: geladene Schriftfamilien und Fotos.
    State,
    /// Schriftdateien (TTF, OTF, TTC) hinzufügen, eine Datei je Eintrag in
    /// `data`.
    AddFonts,
    /// Fotos ersetzen. Die Inhalte stehen in `data` in derselben Reihenfolge
    /// wie die Namen.
    SetPhotos { names: Vec<String> },
    /// Kalender neu setzen und die Seite `page` als SVG liefern (wird auf
    /// die letzte Seite begrenzt).
    Render {
        settings: Box<CalendarSettings>,
        page: usize,
    },
    /// Eine Seite des zuletzt gesetzten Kalenders als SVG.
    Page { page: usize },
    /// PDF des zuletzt gesetzten Kalenders (Bytes in `data[0]`).
    Pdf,
}

/// Antwort des Worker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Reply {
    /// Worker ist geladen und nimmt Aufträge an (ohne Auftrag, `id` 0).
    Ready,
    /// Geladene Schriftfamilien (sortiert) und Anzahl der Fotos.
    State {
        families: Vec<String>,
        photos: usize,
    },
    /// Alle Schriftfamilien und je Datei die neu hinzugefügten Familien.
    Fonts {
        families: Vec<String>,
        added: Vec<Vec<String>>,
    },
    /// Fotos übernommen.
    Photos { count: usize },
    /// Gesetzte Seite.
    Page {
        pages: usize,
        page: usize,
        svg: String,
    },
    /// PDF-Bytes in `data[0]`.
    Pdf,
    /// Fehler (z. B. Typst-Meldungen oder noch keine Vorschau).
    Error { message: String },
}

/// Auftrag oder Antwort mit Kennung.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub id: u64,
    pub body: T,
}

impl<T: Serialize> Envelope<T> {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Nachricht ist serialisierbar")
    }
}

impl<T: for<'de> Deserialize<'de>> Envelope<T> {
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("ungültige Nachricht: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let req = Envelope {
            id: 7,
            body: Request::Render {
                settings: Box::default(),
                page: 2,
            },
        };
        assert_eq!(Envelope::from_json(&req.to_json()), Ok(req));
        let rep = Envelope {
            id: 7,
            body: Reply::Page {
                pages: 13,
                page: 2,
                svg: "<svg/>".into(),
            },
        };
        assert_eq!(Envelope::from_json(&rep.to_json()), Ok(rep));
    }
}
