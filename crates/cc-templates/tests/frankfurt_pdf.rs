//! Vergleich der berechneten Zeiten mit den gedruckten Zeiten des
//! Gemeindekalenders Frankfurt 5784. Datei fehlt → Test wird übersprungen.

use cc_astro::{Location, Profile};
use cc_holidays::year_days;
use cc_templates::times::{MainTime, day_times};

fn parse(t: &str) -> i32 {
    let (h, m) = t.split_once(['.', ':']).unwrap();
    h.parse::<i32>().unwrap() * 60 + m.parse::<i32>().unwrap()
}

#[test]
fn times_match_community_calendar_5784() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../testdata/private/frankfurt-5784-pdf.json");
    let Ok(text) = std::fs::read_to_string(&path) else {
        eprintln!("übersprungen: {} fehlt", path.display());
        return;
    };
    let pdf: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
    let days = year_days(5784);
    let (loc, p) = (Location::frankfurt(), Profile::frankfurt());
    assert_eq!(pdf.len(), days.len());

    let (mut compared, mut exact) = (0, 0);
    let mut problems = Vec::new();
    let mut unexplained = Vec::new();
    for (i, e) in pdf.iter().enumerate() {
        let t = day_times(&days, i, &loc, &p);
        let printed: Vec<i32> = e["times"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| parse(x.as_str().unwrap()))
            .collect();
        let date = e["date"].as_str().unwrap();
        let text = e["text"].as_str().unwrap();
        let mut check = |what: &str, ours: i32, candidates: &[i32]| {
            compared += 1;
            let best = candidates.iter().copied().min_by_key(|c| (c - ours).abs());
            match best {
                Some(b) if b == ours => exact += 1,
                Some(b) if (b - ours).abs() <= 1 => {}
                other => problems.push(format!(
                    "{date} {what}: berechnet {ours}, gedruckt {other:?} ({printed:?})"
                )),
            }
        };
        if let Some(m) = t.main {
            let what = match m {
                MainTime::Candles(_) => "Kerzen",
                MainTime::SecondNightCandles(_) => "Kerzen 2. Nacht",
                MainTime::End(_) => "Ende",
                MainTime::ChanukkaCandles(_) => "Chanukka",
            };
            check(what, m.minutes(), &printed);
        } else if !printed.is_empty() {
            let extras = [
                t.bdikat_chametz,
                t.chametz_eating_ends,
                t.fast_begin,
                t.fast_end,
            ];
            let explained = printed
                .iter()
                .all(|p| extras.iter().flatten().any(|x| (x - p).abs() <= 1));
            if !explained {
                unexplained.push(format!(
                    "{date}: gedruckte Zeit ohne Rechnung {printed:?} ({text})"
                ));
            }
        }
        if let (Some(fb), Some(fe)) = (e.get("fb"), e.get("fe")) {
            let (fb, fe) = (parse(fb.as_str().unwrap()), parse(fe.as_str().unwrap()));
            check("FB", t.fast_begin.unwrap_or(-999), &[fb]);
            check("FE", t.fast_end.unwrap_or(-999), &[fe]);
        }
        for (what, ours) in [
            ("Bdikat", t.bdikat_chametz),
            ("Verbot", t.chametz_eating_ends),
        ] {
            if let Some(o) = ours {
                check(what, o, &printed);
            }
        }
    }
    problems.extend(unexplained);
    // Zusatzzeiten aus der Fußnote von Seite 2 (Erew Pessach, 22.4.2024)
    let seder = days
        .iter()
        .position(|d| d.gregorian == (2024, 4, 22))
        .unwrap();
    let t = day_times(&days, seder, &loc, &p);
    for (what, ours, printed) in [
        ("Chametz verbrennen", t.chametz_burning_ends, 11 * 60 + 54),
        ("Mitternacht", t.seder_midnight, 60 + 24),
        (
            "Taanit Bechorot FB",
            cc_astro::fast_begin(&loc, &p, days[seder].day),
            4 * 60 + 24,
        ),
        (
            "Taanit Bechorot FE",
            cc_astro::fast_end(&loc, &p, days[seder].day),
            21 * 60 + 15,
        ),
    ] {
        compared += 1;
        let o = ours.unwrap();
        if o == printed {
            exact += 1;
        } else if (o - printed).abs() > 1 {
            problems.push(format!("{what}: berechnet {o}, gedruckt {printed}"));
        }
    }
    println!(
        "{compared} Zeiten verglichen, {exact} exakt, {} Abweichungen über 1 Minute",
        problems.len()
    );
    for p in problems.iter().take(30) {
        println!("  {p}");
    }
    assert!(problems.is_empty());
    assert!(
        exact as f64 / compared as f64 > 0.9,
        "weniger als 90 % exakt"
    );
}
