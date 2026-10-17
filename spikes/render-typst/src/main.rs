//! Spike ADR-002, Option A: Typst als eingebettete Render-Engine.

mod shared;

use std::path::Path;
use std::time::Instant;

use shared::{SpikeWorld, markup};
use typst::foundations::Bytes;
use typst::text::Font;
use typst_layout::PagedDocument;

fn load_fonts() -> Vec<Font> {
    let mut fonts: Vec<Font> = typst_assets::fonts()
        .flat_map(|data| Font::iter(Bytes::new(data)))
        .collect();
    for file in ["times.ttf", "timesbd.ttf", "arial.ttf", "arialbd.ttf"] {
        let path = Path::new("C:/Windows/Fonts").join(file);
        match std::fs::read(&path) {
            Ok(data) => fonts.extend(Font::iter(Bytes::new(data))),
            Err(e) => eprintln!("Schrift fehlt: {} ({e})", path.display()),
        }
    }
    fonts
}

/// Synthetisches "Foto" (Farbverlauf) in 6000 × 4000 Pixeln, als JPEG.
fn make_photo() -> Bytes {
    let img = image::RgbImage::from_fn(6000, 4000, |x, y| {
        image::Rgb([(x * 255 / 6000) as u8, (y * 255 / 4000) as u8, 160])
    });
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Jpeg).unwrap();
    Bytes::new(out.into_inner())
}

fn main() {
    let out = Path::new("target/spikes/typst");
    std::fs::create_dir_all(out).unwrap();

    let t = Instant::now();
    let fonts = load_fonts();
    let photo = make_photo();
    println!(
        "Schriften + Foto vorbereitet: {:?} ({} Schriften)",
        t.elapsed(),
        fonts.len()
    );

    for (name, paper, rtl, with_photo) in [
        ("a4-ltr", "a4", false, false),
        ("a4-rtl", "a4", true, false),
        ("a0-foto", "a0", false, true),
    ] {
        let world = SpikeWorld::new(markup(paper, rtl, with_photo), fonts.clone(), photo.clone());

        let t = Instant::now();
        let result = typst::compile::<PagedDocument>(&world);
        let doc = match result.output {
            Ok(doc) => doc,
            Err(errors) => {
                for e in errors {
                    eprintln!("Typst-Fehler: {}", e.message);
                }
                std::process::exit(1);
            }
        };
        let t_layout = t.elapsed();

        let t = Instant::now();
        let pdf = typst_pdf::pdf(&doc, &typst_pdf::PdfOptions::default()).expect("PDF-Export");
        let t_pdf = t.elapsed();

        let t = Instant::now();
        let svg = typst_svg::svg(&doc.pages()[0], &typst_svg::SvgOptions::default());
        let t_svg = t.elapsed();

        let t = Instant::now();
        let png = typst_render::render(&doc.pages()[0], &typst_render::RenderOptions::default())
            .encode_png()
            .expect("PNG");
        let t_png = t.elapsed();

        std::fs::write(out.join(format!("{name}.pdf")), &pdf).unwrap();
        std::fs::write(out.join(format!("{name}.svg")), &svg).unwrap();
        std::fs::write(out.join(format!("{name}.png")), &png).unwrap();
        println!(
            "{name:8} Layout {t_layout:>10.2?} | PDF {t_pdf:>10.2?} ({:>7} KB) | SVG {t_svg:>10.2?} ({:>7} KB) | PNG {t_png:>10.2?}",
            pdf.len() / 1024,
            svg.len() / 1024
        );
    }
}
