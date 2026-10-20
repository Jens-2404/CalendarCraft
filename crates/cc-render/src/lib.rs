//! Ausgabe als PDF, SVG und PNG über Typst (ADR-002).
//!
//! Alles Typst-Spezifische bleibt in diesem Crate. Die Vorlagen erzeugen
//! Typst-Quelltext; hier wird er gesetzt und exportiert.

use std::collections::HashMap;
use std::path::Path;

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_layout::PagedDocument;

/// Gesetzte Seiten.
pub struct Document(PagedDocument);

impl Document {
    pub fn page_count(&self) -> usize {
        self.0.pages().len()
    }
}

/// Geladene Schriften.
#[derive(Clone)]
pub struct Fonts(Vec<Font>);

impl Fonts {
    /// Mitgelieferte Schriften: Libertinus Serif (mit Hebräisch und Nikud)
    /// und DejaVu Sans Mono (Symbole). Lizenzen: assets/fonts/LICENSES.md.
    pub fn bundled() -> Self {
        const FILES: [&[u8]; 8] = [
            include_bytes!("../../../assets/fonts/LibertinusSerif-Regular.otf"),
            include_bytes!("../../../assets/fonts/LibertinusSerif-Bold.otf"),
            include_bytes!("../../../assets/fonts/LibertinusSerif-Italic.otf"),
            include_bytes!("../../../assets/fonts/LibertinusSerif-BoldItalic.otf"),
            include_bytes!("../../../assets/fonts/LibertinusSerif-Semibold.otf"),
            include_bytes!("../../../assets/fonts/LibertinusSerif-SemiboldItalic.otf"),
            include_bytes!("../../../assets/fonts/DejaVuSansMono.ttf"),
            include_bytes!("../../../assets/fonts/DejaVuSansMono-Bold.ttf"),
        ];
        Fonts(
            FILES
                .iter()
                .flat_map(|data| Font::iter(Bytes::new(data)))
                .collect(),
        )
    }

    /// Zusätzlich Schriften aus Systemordnern, deren Dateiname mit einem der
    /// Präfixe beginnt (ohne Beachtung der Groß-/Kleinschreibung). Für den
    /// Prototyp; die App liefert später eigene freie Schriften mit.
    pub fn with_system(mut self, prefixes: &[&str]) -> Self {
        for dir in system_font_dirs() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().to_lowercase();
                let is_font =
                    name.ends_with(".ttf") || name.ends_with(".otf") || name.ends_with(".ttc");
                if is_font
                    && prefixes.iter().any(|p| name.starts_with(p))
                    && let Ok(data) = std::fs::read(e.path())
                {
                    self.0.extend(Font::iter(Bytes::new(data)));
                }
            }
        }
        self
    }

    /// Schriften aus einer Datei (TTF, OTF, TTC) hinzufügen. Gibt die
    /// Familiennamen der hinzugefügten Schriften zurück.
    pub fn add_data(&mut self, data: Vec<u8>) -> Vec<String> {
        let new: Vec<Font> = Font::iter(Bytes::new(data)).collect();
        let names = new.iter().map(|f| f.info().family.clone()).collect();
        self.0.extend(new);
        names
    }

    /// Alle Schriften aus den Systemordnern (inkl. vom Benutzer installierter
    /// Schriften unter Windows). Dauert je nach Anzahl etwas länger.
    pub fn with_all_system(self) -> Self {
        self.with_system(&[""])
    }

    /// Ist eine Schriftfamilie geladen (ohne Beachtung der Groß-/Kleinschreibung)?
    pub fn has_family(&self, family: &str) -> bool {
        self.0
            .iter()
            .any(|f| f.info().family.eq_ignore_ascii_case(family))
    }

    /// Schriftfamilien (für Fehlersuche).
    pub fn families(&self) -> Vec<String> {
        let mut f: Vec<String> = self.0.iter().map(|f| f.info().family.clone()).collect();
        f.sort();
        f.dedup();
        f
    }
}

/// Schriftfamilien der Systemordner mit Dateipfad (eine Datei je Familie
/// und Schnitt), sortiert nach Familie. Für die Auswahl in der Desktop-App.
pub fn system_font_catalog() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for dir in system_font_dirs() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let path = e.path();
            let name = path.to_string_lossy().to_lowercase();
            if !(name.ends_with(".ttf") || name.ends_with(".otf") || name.ends_with(".ttc")) {
                continue;
            }
            if let Ok(data) = std::fs::read(&path) {
                for f in Font::iter(Bytes::new(data)) {
                    out.push((f.info().family.clone(), path.to_string_lossy().into_owned()));
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn system_font_dirs() -> Vec<std::path::PathBuf> {
    let mut v = Vec::new();
    if cfg!(windows) {
        v.push("C:/Windows/Fonts".into());
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            v.push(std::path::PathBuf::from(local).join("Microsoft/Windows/Fonts"));
        }
    } else if cfg!(target_os = "macos") {
        v.push("/System/Library/Fonts".into());
        v.push("/Library/Fonts".into());
    } else {
        v.push("/usr/share/fonts/truetype".into());
        v.push("/usr/share/fonts".into());
    }
    v
}

struct World_ {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main: Source,
    files: HashMap<String, Bytes>,
}

impl World for World_ {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }
    fn main(&self) -> FileId {
        self.main.id()
    }
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            Ok(self.main.clone())
        } else {
            Err(FileError::NotFound(id.vpath().get_without_slash().into()))
        }
    }
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.files
            .get(id.vpath().get_without_slash())
            .cloned()
            .ok_or_else(|| FileError::NotFound(id.vpath().get_without_slash().into()))
    }
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }
    fn today(&self, _offset: Option<Duration>) -> Option<Datetime> {
        None
    }
}

/// Setzt Typst-Quelltext zu Seiten. `files` sind eingebettete Dateien
/// (Bilder) unter ihrem Pfad im Quelltext.
pub fn compile(
    source: &str,
    fonts: &Fonts,
    files: &[(&str, Vec<u8>)],
) -> Result<Document, Vec<String>> {
    let id = FileId::new(RootedPath::new(
        VirtualRoot::Project,
        VirtualPath::new("main.typ").expect("gültiger Pfad"),
    ));
    let world = World_ {
        library: LazyHash::new(Library::default()),
        book: LazyHash::new(FontBook::from_fonts(&fonts.0)),
        fonts: fonts.0.clone(),
        main: Source::new(id, source.to_string()),
        files: files
            .iter()
            .map(|(n, d)| ((*n).to_string(), Bytes::new(d.clone())))
            .collect(),
    };
    let result = typst::compile::<PagedDocument>(&world);
    match result.output {
        Ok(doc) => Ok(Document(doc)),
        Err(errors) => Err(errors.iter().map(|e| e.message.to_string()).collect()),
    }
}

/// PDF mit eingebetteten Schriften.
pub fn to_pdf(doc: &Document) -> Result<Vec<u8>, String> {
    typst_pdf::pdf(&doc.0, &typst_pdf::PdfOptions::default()).map_err(|e| {
        e.iter()
            .map(|d| d.message.to_string())
            .collect::<Vec<_>>()
            .join("; ")
    })
}

/// SVG einer Seite (0-basiert), für die Vorschau.
pub fn page_svg(doc: &Document, page: usize) -> Option<String> {
    doc.0
        .pages()
        .get(page)
        .map(|p| typst_svg::svg(p, &typst_svg::SvgOptions::default()))
}

/// PNG einer Seite (0-basiert); `pixel_per_pt` = 1.0 entspricht 72 dpi.
#[cfg(feature = "png")]
pub fn page_png(doc: &Document, page: usize, pixel_per_pt: f32) -> Option<Vec<u8>> {
    let opts = typst_render::RenderOptions {
        pixel_per_pt: typst::utils::Scalar::new(f64::from(pixel_per_pt)),
        ..Default::default()
    };
    doc.0
        .pages()
        .get(page)
        .and_then(|p| typst_render::render(p, &opts).encode_png().ok())
}

/// Hilfsfunktion: Datei lesen, falls vorhanden.
pub fn read_file(path: &Path) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_fonts_have_hebrew() {
        let f = Fonts::bundled();
        assert!(f.has_family("Libertinus Serif"));
        assert!(f.has_family("DejaVu Sans Mono"));
    }

    #[test]
    fn system_catalog_lists_installed_fonts() {
        // Auf Rechnern ohne Schriftordner (z. B. manche CI-Container) leer
        let cat = system_font_catalog();
        if cfg!(windows) {
            assert!(
                cat.iter().any(|(f, _)| f == "Arial"),
                "{} Einträge",
                cat.len()
            );
        }
    }

    #[test]
    fn compiles_hebrew_text() {
        let doc = compile(
            "#set text(font: \"Libertinus Serif\")\nשָׁלוֹם 5787",
            &Fonts::bundled(),
            &[],
        )
        .unwrap();
        assert_eq!(doc.page_count(), 1);
        assert!(to_pdf(&doc).unwrap().starts_with(b"%PDF"));
    }
}
