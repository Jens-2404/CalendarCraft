//! Gemeinsamer Teil der Typst-Spikes (nativ und WASM): World und Testblatt.

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};

pub struct SpikeWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main: Source,
    photo: Bytes,
}

impl SpikeWorld {
    pub fn new(markup: String, fonts: Vec<Font>, photo: Bytes) -> Self {
        let main_id = FileId::new(RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new("main.typ").unwrap(),
        ));
        Self {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(FontBook::from_fonts(&fonts)),
            fonts,
            main: Source::new(main_id, markup),
            photo,
        }
    }
}

impl World for SpikeWorld {
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
        if id.vpath().get_without_slash() == "photo.jpg" {
            Ok(self.photo.clone())
        } else {
            Err(FileError::NotFound(id.vpath().get_without_slash().into()))
        }
    }
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }
    fn today(&self, _offset: Option<typst::foundations::Duration>) -> Option<Datetime> {
        Datetime::from_ymd(2026, 10, 17)
    }
}

/// Testdaten September 2026: (Tag, Eintrag hebräisch mit Nikud, Eintrag deutsch)
fn events(day: u8) -> Option<(&'static str, &'static str)> {
    match day {
        5 => Some(("כִּי תָבוֹא", "Ki Tawo · Kerzen 19:46")),
        11 => Some(("עֶרֶב רֹאשׁ הַשָּׁנָה", "Erew Rosch HaSchana")),
        12 => Some(("רֹאשׁ הַשָּׁנָה א׳", "Rosch HaSchana I")),
        13 => Some(("רֹאשׁ הַשָּׁנָה ב׳", "Rosch HaSchana II")),
        14 => Some(("צוֹם גְּדַלְיָה", "Zom Gedalja")),
        21 => Some(("יוֹם כִּפּוּר", "Jom Kippur")),
        26 => Some(("סֻכּוֹת א׳", "Sukkot I")),
        _ => None,
    }
}

fn hebrew_month(code: &str) -> &'static str {
    match code {
        "M12" => "אֱלוּל",
        "M01" => "תִּשְׁרֵי",
        _ => "?",
    }
}

pub fn markup(paper: &str, rtl: bool, photo: bool) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "#set page(paper: \"{paper}\", flipped: true, margin: 4%)\n\
         #set text(font: (\"Times New Roman\", \"Libertinus Serif\"), size: {}pt, lang: \"{}\", dir: {})\n",
        if paper == "a0" { 40 } else { 11 },
        if rtl { "he" } else { "de" },
        if rtl { "rtl" } else { "ltr" },
    ));
    if photo {
        s.push_str("#box(height: 30%, width: 100%, clip: true, image(\"photo.jpg\", width: 100%, height: 100%, fit: \"cover\"))\n");
    }
    s.push_str(
        "#align(center, text(size: 2em, weight: \"bold\")[September 2026 · אֱלוּל – תִּשְׁרֵי תשפ״ו–תשפ״ז])\n",
    );
    let header = if rtl {
        ["א׳", "ב׳", "ג׳", "ד׳", "ה׳", "ו׳", "שבת"]
    } else {
        ["So", "Mo", "Di", "Mi", "Do", "Fr", "Schabbat"]
    };
    s.push_str("#table(columns: (1fr,) * 7, rows: (auto,) + (1fr,) * 5, stroke: 0.5pt + gray, inset: 4pt,\n");
    for h in header {
        s.push_str(&format!("  table.cell(fill: luma(230))[*{h}*],\n"));
    }
    // 1. September 2026 ist ein Dienstag (Index 2 bei Wochenbeginn Sonntag).
    for _ in 0..2 {
        s.push_str("  [],\n");
    }
    for day in 1..=30u8 {
        let h = cc_calendar::gregorian_to_hebrew(2026, 9, day).unwrap();
        let mut cell = format!(
            "#text(size: 1.6em, weight: \"bold\")[{day}] #h(1fr) #text(dir: rtl, lang: \"he\")[{} {}]",
            h.day,
            hebrew_month(&h.month_code)
        );
        if let Some((he, de)) = events(day) {
            cell.push_str(&format!(
                " \\ #text(dir: rtl, lang: \"he\", fill: rgb(\"#1f4e8c\"))[{he}] \\ #text(size: 0.8em)[{de}]"
            ));
        }
        s.push_str(&format!("  [{cell}],\n"));
    }
    s.push_str(")\n");
    s
}
