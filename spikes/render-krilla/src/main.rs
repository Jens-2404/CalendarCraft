//! Spike ADR-002, Option B: eigenes Layout, PDF direkt mit krilla.
//!
//! Absichtlich schlicht: festes Raster, keine Zeilenumbrüche. Genau das
//! müsste Option B in der App selbst bauen (Umbruch, Bidi-Absätze,
//! Silbentrennung, SVG-Ausgabe für die Vorschau).

use std::path::Path;
use std::time::Instant;

use krilla::Document;
use krilla::color::rgb;
use krilla::geom::{PathBuilder, Point, Rect};
use krilla::page::PageSettings;
use krilla::paint::{Fill, Stroke};
use krilla::text::{Font, TextDirection};

const A4_W: f32 = 841.89; // quer, in pt
const A4_H: f32 = 595.28;

fn font(file: &str) -> Font {
    let data = std::fs::read(Path::new("C:/Windows/Fonts").join(file)).expect("Schrift fehlt");
    Font::new(data.into(), 0).expect("Schrift nicht lesbar")
}

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

fn main() {
    let out = Path::new("target/spikes/krilla");
    std::fs::create_dir_all(out).unwrap();
    let regular = font("times.ttf");
    let bold = font("timesbd.ttf");

    let t = Instant::now();
    let mut document = Document::new();
    let mut page = document.start_page_with(PageSettings::from_wh(A4_W, A4_H).unwrap());
    let mut surface = page.surface();

    let black = Fill {
        paint: rgb::Color::new(0, 0, 0).into(),
        ..Default::default()
    };
    let blue = Fill {
        paint: rgb::Color::new(31, 78, 140).into(),
        ..Default::default()
    };

    surface.set_fill(Some(black.clone()));
    surface.draw_text(
        Point::from_xy(250.0, 40.0),
        bold.clone(),
        20.0,
        "September 2026 · אֱלוּל – תִּשְׁרֵי תשפ״ו–תשפ״ז",
        false,
        TextDirection::Auto,
    );

    let (x0, y0, cw, ch) = (24.0, 70.0, (A4_W - 48.0) / 7.0, (A4_H - 100.0) / 5.0);
    surface.set_stroke(Some(Stroke {
        paint: rgb::Color::new(128, 128, 128).into(),
        width: 0.5,
        ..Default::default()
    }));
    surface.set_fill(None);
    for row in 0..5 {
        for col in 0..7 {
            let mut pb = PathBuilder::new();
            pb.push_rect(
                Rect::from_xywh(x0 + col as f32 * cw, y0 + row as f32 * ch, cw, ch).unwrap(),
            );
            surface.draw_path(&pb.finish().unwrap());
        }
    }
    surface.set_stroke(None);

    for day in 1..=30u8 {
        let idx = day as usize + 1; // 1.9.2026 = Dienstag
        let (col, row) = ((idx % 7) as f32, (idx / 7) as f32);
        let (x, y) = (x0 + col * cw + 4.0, y0 + row * ch + 16.0);
        let h = cc_calendar::gregorian_to_hebrew(2026, 9, day).unwrap();

        surface.set_fill(Some(black.clone()));
        surface.draw_text(
            Point::from_xy(x, y),
            bold.clone(),
            16.0,
            &day.to_string(),
            false,
            TextDirection::Auto,
        );
        surface.draw_text(
            Point::from_xy(x + cw - 70.0, y),
            regular.clone(),
            10.0,
            &format!("{} {}", h.day, hebrew_month(&h.month_code)),
            false,
            TextDirection::RightToLeft,
        );
        if let Some((he, de)) = events(day) {
            surface.set_fill(Some(blue.clone()));
            surface.draw_text(
                Point::from_xy(x, y + 18.0),
                regular.clone(),
                10.0,
                he,
                false,
                TextDirection::RightToLeft,
            );
            surface.set_fill(Some(black.clone()));
            surface.draw_text(
                Point::from_xy(x, y + 32.0),
                regular.clone(),
                8.0,
                de,
                false,
                TextDirection::Auto,
            );
        }
    }

    surface.finish();
    page.finish();
    let pdf = document.finish().expect("PDF");
    let elapsed = t.elapsed();
    std::fs::write(out.join("a4-ltr.pdf"), &pdf).unwrap();
    println!(
        "a4-ltr   Layout+PDF {elapsed:>10.2?} ({:>7} KB)",
        pdf.len() / 1024
    );
}
