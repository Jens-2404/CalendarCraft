//! Zustand des Worker und Bearbeitung der Aufträge, unabhängig von
//! JavaScript (daher auch nativ testbar).

use cc_render::{Document, Fonts};
use cc_templates::settings::typst_source;
use cc_worker_protocol::{Reply, Request};

pub struct Satz {
    fonts: Fonts,
    photos: Vec<(String, Vec<u8>)>,
    doc: Option<Document>,
}

impl Default for Satz {
    fn default() -> Self {
        Self {
            fonts: Fonts::bundled(),
            photos: Vec::new(),
            doc: None,
        }
    }
}

impl Satz {
    /// Bearbeitet einen Auftrag; `data` sind die mitgeschickten Binärdaten.
    /// Gibt die Antwort und ihre Binärdaten zurück.
    pub fn handle(&mut self, req: Request, data: Vec<Vec<u8>>) -> (Reply, Vec<Vec<u8>>) {
        match req {
            Request::State => (
                Reply::State {
                    families: self.fonts.families(),
                    photos: self.photos.len(),
                },
                Vec::new(),
            ),
            Request::AddFonts => {
                let added = data.into_iter().map(|d| self.fonts.add_data(d)).collect();
                (
                    Reply::Fonts {
                        families: self.fonts.families(),
                        added,
                    },
                    Vec::new(),
                )
            }
            Request::SetPhotos { names } => {
                self.photos = names.into_iter().zip(data).collect();
                (
                    Reply::Photos {
                        count: self.photos.len(),
                    },
                    Vec::new(),
                )
            }
            Request::Render { settings, page } => {
                let names: Vec<String> = self.photos.iter().map(|(n, _)| n.clone()).collect();
                let result = typst_source(&settings, &names).and_then(|src| {
                    let mut files: Vec<(&str, Vec<u8>)> = cc_templates::monat::icon_files();
                    for (n, d) in &self.photos {
                        files.push((n.as_str(), d.clone()));
                    }
                    cc_render::compile(&src, &self.fonts, &files).map_err(|e| e.join("; "))
                });
                match result {
                    Ok(doc) => {
                        self.doc = Some(doc);
                        (self.page(page), Vec::new())
                    }
                    // Die letzte gelungene Vorschau bleibt erhalten
                    Err(message) => (Reply::Error { message }, Vec::new()),
                }
            }
            Request::Page { page } => (self.page(page), Vec::new()),
            Request::Pdf => match self.doc.as_ref().map(cc_render::to_pdf) {
                Some(Ok(bytes)) => (Reply::Pdf, vec![bytes]),
                Some(Err(message)) => (Reply::Error { message }, Vec::new()),
                None => (
                    Reply::Error {
                        message: "Noch keine Vorschau vorhanden".into(),
                    },
                    Vec::new(),
                ),
            },
        }
    }

    fn page(&self, page: usize) -> Reply {
        let Some(doc) = &self.doc else {
            return Reply::Error {
                message: "Noch keine Vorschau vorhanden".into(),
            };
        };
        let pages = doc.page_count();
        let page = page.min(pages.saturating_sub(1));
        Reply::Page {
            pages,
            page,
            svg: cc_render::page_svg(doc, page).unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cc_templates::settings::{CalendarKind, CalendarSettings};

    #[test]
    fn renders_pages_and_pdf() {
        let mut s = Satz::default();
        let (r, _) = s.handle(Request::Pdf, Vec::new());
        assert!(matches!(r, Reply::Error { .. }));

        let settings = CalendarSettings {
            kind: CalendarKind::Jahr,
            year: 2027,
            ..Default::default()
        };
        let (r, _) = s.handle(
            Request::Render {
                settings: Box::new(settings),
                page: 99,
            },
            Vec::new(),
        );
        let Reply::Page { pages, page, svg } = r else {
            panic!("{r:?}")
        };
        assert_eq!(pages, 2);
        assert_eq!(page, 1);
        assert!(svg.starts_with("<svg"));

        let (r, _) = s.handle(Request::Page { page: 0 }, Vec::new());
        assert!(matches!(r, Reply::Page { page: 0, .. }));

        let (r, data) = s.handle(Request::Pdf, Vec::new());
        assert_eq!(r, Reply::Pdf);
        assert!(data[0].starts_with(b"%PDF"));
    }

    #[test]
    fn fonts_and_photos() {
        let mut s = Satz::default();
        let (r, _) = s.handle(Request::AddFonts, vec![b"keine Schrift".to_vec()]);
        let Reply::Fonts { families, added } = r else {
            panic!("{r:?}")
        };
        assert!(families.iter().any(|f| f == "Libertinus Serif"));
        assert_eq!(added, vec![Vec::<String>::new()]);

        let (r, _) = s.handle(
            Request::SetPhotos {
                names: vec!["foto-000.png".into()],
            },
            vec![vec![1, 2, 3]],
        );
        assert_eq!(r, Reply::Photos { count: 1 });
        let (r, _) = s.handle(Request::State, Vec::new());
        assert!(matches!(r, Reply::State { photos: 1, .. }));
    }
}
