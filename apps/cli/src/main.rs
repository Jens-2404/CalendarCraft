//! Kommandozeile.
//!
//! ```text
//! cc-cli convert JJJJ-MM-TT
//! cc-cli gemeindekalender --year 5787 --out kalender.pdf [--paper a4|a3|a2|letter]
//!                         [--profile gemeinde.json] [--source kalender.typ] [--png]
//! ```

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use cc_render::Fonts;
use cc_templates::gemeinde::{CommunityProfile, Options, Paper, typst_source};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("convert") if args.len() == 2 => convert(&args[1]),
        Some("gemeindekalender") => match gemeindekalender(&args[1..]) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Fehler: {e}");
                ExitCode::FAILURE
            }
        },
        _ => {
            eprintln!(
                "Aufruf:\n  cc-cli convert JJJJ-MM-TT\n  cc-cli gemeindekalender --year 5787 --out datei.pdf [--paper a4|a3|a2|letter] [--profile gemeinde.json] [--source datei.typ] [--png]"
            );
            ExitCode::from(2)
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

fn gemeindekalender(args: &[String]) -> Result<(), String> {
    let mut year: Option<i32> = None;
    let mut out: Option<PathBuf> = None;
    let mut paper = Paper::A4;
    let mut profile = CommunityProfile::default();
    let mut source: Option<PathBuf> = None;
    let mut png = false;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = |name: &str| {
            it.next()
                .cloned()
                .ok_or(format!("{name} braucht einen Wert"))
        };
        match a.as_str() {
            "--year" => year = Some(value("--year")?.parse().map_err(|_| "Jahr nicht lesbar")?),
            "--out" => out = Some(value("--out")?.into()),
            "--paper" => {
                let n = value("--paper")?;
                paper = Paper::by_name(&n).ok_or(format!("unbekanntes Format {n}"))?;
            }
            "--profile" => profile = read_profile(&PathBuf::from(value("--profile")?))?,
            "--source" => source = Some(value("--source")?.into()),
            "--png" => png = true,
            other => return Err(format!("unbekannte Option {other}")),
        }
    }
    let year = year.ok_or("--year fehlt (hebräisches Jahr, z. B. 5787)")?;
    let out = out.ok_or("--out fehlt")?;

    let t = Instant::now();
    let options = Options {
        paper,
        profile,
        ..Options::default()
    };
    let src = typst_source(year, &options);
    let t_data = t.elapsed();
    if let Some(p) = &source {
        std::fs::write(p, &src).map_err(|e| e.to_string())?;
    }

    let t = Instant::now();
    let fonts = Fonts::bundled().with_system(&["arial", "times"]);
    let t_fonts = t.elapsed();
    let t = Instant::now();
    let doc =
        cc_render::compile(&src, &fonts, &[]).map_err(|e| format!("Typst: {}", e.join("; ")))?;
    let t_layout = t.elapsed();
    let t = Instant::now();
    let pdf = cc_render::to_pdf(&doc)?;
    std::fs::write(&out, &pdf).map_err(|e| e.to_string())?;
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
fn read_profile(path: &PathBuf) -> Result<CommunityProfile, String> {
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
    let mut p = CommunityProfile::default();
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
