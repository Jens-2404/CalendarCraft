//! Spike ADR-002: Typst im Browser. Misst Bundle-Größe und Renderzeiten.
//!
//! Start: `cd spikes/typst-wasm && trunk serve --release --port 8091`

#[path = "../../render-typst/src/shared.rs"]
mod shared;

use shared::{SpikeWorld, markup};
use typst::foundations::Bytes;
use typst::text::Font;
use typst_layout::PagedDocument;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

fn now() -> f64 {
    web_sys::window().unwrap().performance().unwrap().now()
}

async fn fetch_bytes(url: &str) -> Vec<u8> {
    let window = web_sys::window().unwrap();
    let resp: web_sys::Response = JsFuture::from(window.fetch_with_str(url))
        .await
        .unwrap()
        .dyn_into()
        .unwrap();
    let buf = JsFuture::from(resp.array_buffer().unwrap()).await.unwrap();
    js_sys::Uint8Array::new(&buf).to_vec()
}

fn main() {
    console_error_panic_hook::set_once();
    wasm_bindgen_futures::spawn_local(run());
}

async fn run() {
    let doc = web_sys::window().unwrap().document().unwrap();
    let log_el = doc.get_element_by_id("log").unwrap();
    let mut log = String::new();

    let t = now();
    let font_data = fetch_bytes("times.ttf").await;
    let fonts: Vec<Font> = Font::iter(Bytes::new(font_data)).collect();
    log.push_str(&format!("Schrift laden: {:.0} ms\n", now() - t));

    let mut svg_first = String::new();
    for (name, paper, rtl) in [
        ("A4 LTR (kalt)", "a4", false),
        ("A4 LTR (warm)", "a4", false),
        ("A4 RTL", "a4", true),
        ("A0 LTR", "a0", false),
    ] {
        let world = SpikeWorld::new(
            markup(paper, rtl, false),
            fonts.clone(),
            Bytes::new(Vec::new()),
        );
        let t = now();
        let document = match typst::compile::<PagedDocument>(&world).output {
            Ok(d) => d,
            Err(errs) => {
                log.push_str(&format!(
                    "Fehler: {:?}\n",
                    errs.iter()
                        .map(|e| e.message.to_string())
                        .collect::<Vec<_>>()
                ));
                break;
            }
        };
        let t_layout = now() - t;
        let t = now();
        let pdf = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default()).unwrap();
        let t_pdf = now() - t;
        let t = now();
        let svg = typst_svg::svg(&document.pages()[0], &typst_svg::SvgOptions::default());
        let t_svg = now() - t;
        log.push_str(&format!(
            "{name:14} Layout {t_layout:6.0} ms | PDF {t_pdf:5.0} ms ({} KB) | SVG {t_svg:5.0} ms\n",
            pdf.len() / 1024
        ));
        if svg_first.is_empty() {
            svg_first = svg;
        }
        log_el.set_text_content(Some(&log));
    }
    log.push_str("fertig\n");
    log_el.set_text_content(Some(&log));
    doc.get_element_by_id("page")
        .unwrap()
        .set_inner_html(&svg_first);
}
