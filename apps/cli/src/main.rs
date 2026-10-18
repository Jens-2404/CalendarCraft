//! Kommandozeile.
//!
//! ```text
//! cc-cli convert JJJJ-MM-TT
//! cc-cli jahreskalender  --year 2027 --out kalender.pdf [Gestaltung] [--seiten 1|2] [--start-monat 1-12] [--titel TEXT]
//! cc-cli gemeindekalender --year 5787 --out kalender.pdf [Gestaltung] [--profile gemeinde.json]
//!
//! Gestaltung:
//!   --format a4|a3|a2|a1|a0|a5|letter|legal|tabloid|BxH   (Standard a4, quer)
//!   --hoch                         Hochformat
//!   --schrift NAME                 Schriftfamilie, z. B. "Segoe UI", "Calibri"
//!   --titelschrift NAME            Schrift der Monatsköpfe
//!   --schriftgroesse FAKTOR        z. B. 1.2 für 20 % größer
//!   --png                          zusätzlich PNG je Seite
//!   --source DATEI.typ             Typst-Quelltext speichern
//! ```

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use cc_render::Fonts;
use cc_templates::common::{Paper, Style};
use cc_templates::{gemeinde, jahr};

const USAGE: &str = "Aufruf:
  cc-cli convert JJJJ-MM-TT
  cc-cli jahreskalender  --year 2027 --out datei.pdf [Gestaltung] [--seiten 1|2] [--start-monat 1-12] [--titel TEXT]
  cc-cli gemeindekalender --year 5787 --out datei.pdf [Gestaltung] [--profile gemeinde.json]

Gestaltung:
  --format a4|a3|a2|a1|a0|a5|letter|legal|tabloid|BxH (mm)   Standard: a4 quer
  --hoch                    Hochformat
  --schrift NAME            Schriftfamilie, z. B. \"Segoe UI\"
  --titelschrift NAME       Schrift der Monatsköpfe
  --schriftgroesse FAKTOR   z. B. 1.2 = 20 % größer
  --png                     zusätzlich PNG je Seite
  --source DATEI.typ        Typst-Quelltext speichern";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("convert") if args.len() == 2 => return convert(&args[1]),
        Some("gemeindekalender") => run(Kind::Gemeinde, &args[1..]),
        Some("jahreskalender") => run(Kind::Jahr, &args[1..]),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Fehler: {e}");
            ExitCode::FAILURE
        }
    }
}

fn convert(input: &str) -> ExitCode {
    let parts: Vec<&str> = input.split('-').collect();
    let parsed = match parts.as_slice() {
        [y, m, d] => y.parse().ok().zip(m.parse().ok()).zip(d.parse().ok()),
        _ => None,
    };
    let Some(((y, m), d)) = parsed else {
        eprintln!("Datum nicht lesbar: {input} (erwartet JJJJ-MM-TT)");
        return ExitCode::from(2);
    };
    match cc_calendar::gregorian_to_hebrew(y, m, d) {
        Ok(h) => {
            println!("{input} = {}. {} {}", h.day, h.month_code, h.year);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Gemeinde,
    Jahr,
}

fn run(kind: Kind, args: &[String]) -> Result<(), String> {
    let mut year: Option<i32> = None;
    let mut out: Option<PathBuf> = None;
    let mut paper = Paper::A4;
    let mut portrait = false;
    let mut style = Style::default();
    let mut custom_font = false;
    let mut profile = gemeinde::CommunityProfile::default();
    let mut source: Option<PathBuf> = None;
    let mut png = false;
    let mut pages = 2u8;
    let mut start_month = 1u8;
    let mut title: Option<String> = None;

    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = |name: &str| {
            it.next()
                .cloned()
                .ok_or(format!("{name} braucht einen Wert"))
        };
        match a.as_str() {
            "--year" | "--jahr" => year = Some(value(a)?.parse().map_err(|_| "Jahr nicht lesbar")?),
            "--out" => out = Some(value(a)?.into()),
            "--format" | "--paper" => {
                let n = value(a)?;
                paper = Paper::by_name(&n).ok_or(format!("unbekanntes Format {n}"))?;
            }
            "--hoch" => portrait = true,
            "--schrift" => {
                style.font = value(a)?;
                custom_font = true;
            }
            "--titelschrift" => {
                style.heading_font = value(a)?;
                custom_font = true;
            }
            "--schriftgroesse" => {
                let f: f64 = value(a)?
                    .replace(',', ".")
                    .parse()
                    .map_err(|_| "Schriftgröße: Faktor wie 1.2 angeben")?;
                if !(0.5..=3.0).contains(&f) {
                    return Err("Schriftgröße: Faktor zwischen 0.5 und 3.0".into());
                }
                style.font_scale = f;
            }
            "--seiten" => {
                pages = value(a)?.parse().map_err(|_| "--seiten: 1 oder 2")?;
            }
            "--start-monat" => {
                start_month = value(a)?.parse().map_err(|_| "--start-monat: 1–12")?;
                if !(1..=12).contains(&start_month) {
                    return Err("--start-monat: 1–12".into());
                }
            }
            "--titel" => title = Some(value(a)?),
            "--profile" => profile = read_profile(&PathBuf::from(value(a)?))?,
            "--source" => source = Some(value(a)?.into()),
            "--png" => png = true,
            other => return Err(format!("unbekannte Option {other}\n\n{USAGE}")),
        }
    }
    let year = year.ok_or("--year fehlt")?;
    let out = out.ok_or("--out fehlt")?;
    if portrait {
        paper = paper.portrait();
    }

    // Schriften: Standard nur Arial/Times; bei eigener Schrift alle Systemschriften
    let t = Instant::now();
    let fonts = if custom_font {
        Fonts::bundled().with_all_system()
    } else {
        Fonts::bundled().with_system(&["arial", "times"])
    };
    for f in [&style.font, &style.heading_font] {
        if !fonts.has_family(f) {
            let similar: Vec<String> = fonts
                .families()
                .into_iter()
                .filter(|x| {
                    x.to_lowercase()
                        .contains(&f.to_lowercase().chars().take(4).collect::<String>())
                })
                .take(10)
                .collect();
            return Err(format!(
                "Schrift \"{f}\" nicht gefunden. Ähnliche installierte Schriften: {}",
                if similar.is_empty() {
                    "keine".into()
                } else {
                    similar.join(", ")
                }
            ));
        }
    }
    let t_fonts = t.elapsed();

    let t = Instant::now();
    let src = match kind {
        Kind::Gemeinde => gemeinde::typst_source(
            year,
            &gemeinde::Options {
                paper,
                style,
                profile,
                ..gemeinde::Options::default()
            },
        ),
        Kind::Jahr => {
            let mut o = jahr::Options {
                paper,
                style,
                pages,
                start_month,
                ..jahr::Options::default()
            };
            if let Some(t) = title {
                o.title = t;
            }
            jahr::typst_source(year, &o)
        }
    };
    let t_data = t.elapsed();
    if let Some(p) = &source {
        std::fs::write(p, &src).map_err(|e| e.to_string())?;
    }

    let t = Instant::now();
    let doc =
        cc_render::compile(&src, &fonts, &[]).map_err(|e| format!("Typst: {}", e.join("; ")))?;
    let t_layout = t.elapsed();
    let t = Instant::now();
    let pdf = cc_render::to_pdf(&doc)?;
    std::fs::write(&out, &pdf).map_err(|e| format!("{}: {e}", out.display()))?;
    let t_pdf = t.elapsed();
    if png {
        for i in 0..doc.page_count() {
            let data = cc_render::page_png(&doc, i, 2.5).ok_or("PNG nicht möglich")?;
            let p = out.with_file_name(format!(
                "{}-{}.png",
                out.file_stem().unwrap_or_default().to_string_lossy(),
                i + 1
            ));
            std::fs::write(p, data).map_err(|e| e.to_string())?;
        }
    }
    println!(
        "{}: {} Seiten, {} KB | Daten {:.0?}, Schriften {:.0?}, Satz {:.0?}, PDF {:.0?}",
        out.display(),
        doc.page_count(),
        pdf.len() / 1024,
        t_data,
        t_fonts,
        t_layout,
        t_pdf
    );
    Ok(())
}

/// Gemeindeprofil aus JSON: name_lines, name_hebrew, info_blocks, header_mark.
fn read_profile(path: &PathBuf) -> Result<gemeinde::CommunityProfile, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let strings = |x: &serde_json::Value| -> Vec<String> {
        x.as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|s| s.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    let mut p = gemeinde::CommunityProfile::default();
    if v.get("name_lines").is_some() {
        p.name_lines = strings(&v["name_lines"]);
    }
    if let Some(s) = v["name_hebrew"].as_str() {
        p.name_hebrew = s.to_string();
    }
    if let Some(blocks) = v["info_blocks"].as_array() {
        p.info_blocks = blocks.iter().map(strings).collect();
    }
    if let Some(s) = v["header_mark"].as_str() {
        p.header_mark = s.to_string();
    }
    Ok(p)
}
