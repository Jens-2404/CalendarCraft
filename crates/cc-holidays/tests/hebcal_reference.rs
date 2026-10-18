//! Vergleich mit Hebcal-Jahresdaten (Diaspora). Die Daten liegen nur lokal
//! unter testdata/private/years (von Git ausgeschlossen, siehe
//! docs/06-teststrategie.md); fehlen sie, wird der Test übersprungen.

use std::collections::BTreeSet;
use std::path::PathBuf;

use cc_holidays::{RoshChodesh, year_days};

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata/private/years")
}

/// Hebcal-Titel → unser Name (nur Einträge, die wir berechnen).
fn map_title(title: &str) -> Option<&'static str> {
    Some(match title {
        "Erev Rosh Hashana" => "Erew Rosch Haschana",
        "Rosh Hashana II" => "2. Tag Rosch Haschana",
        "Tzom Gedaliah" => "Fasten Gedalja",
        "Erev Yom Kippur" => "Erew Jom Kippur",
        "Yom Kippur" => "Jom Kippur",
        "Erev Sukkos" => "Erew Sukkot",
        "Sukkos I" => "1. Tag Sukkot",
        "Sukkos II" => "2. Tag Sukkot",
        "Sukkos III (CH’’M)" | "Sukkos IV (CH’’M)" | "Sukkos V (CH’’M)" | "Sukkos VI (CH’’M)" => {
            "Chol Hamoed"
        }
        "Sukkos VII (Hoshana Raba)" => "Hoschana Raba",
        "Shmini Atzeres" => "Schemini Azeret",
        "Simchas Torah" => "Simchat Torah",
        "Asara B’Teves" => "Fasten 10. Tewet",
        "Tu BiShvat" => "Tu biSchwat",
        "Purim Koton" => "Purim Katan",
        "Shushan Purim Koton" => "Schuschan Purim Katan",
        "Ta’anis Esther" => "Fasten Esther",
        "Purim" => "Purim",
        "Shushan Purim" => "Schuschan Purim",
        "Ta’anis Bechoros" => "Taanit Bechorot",
        "Erev Pesach" => "Erew Pessach",
        "Pesach I" => "1. Tag Pessach",
        "Pesach II" => "2. Tag Pessach",
        "Pesach III (CH’’M)" | "Pesach IV (CH’’M)" | "Pesach V (CH’’M)" | "Pesach VI (CH’’M)" => {
            "Chol Hamoed"
        }
        "Pesach VII" => "7. Tag Pessach",
        "Pesach VIII" => "8. Tag Pessach",
        "Pesach Sheni" => "Pessach Scheni",
        "Lag BaOmer" => "Lag Baomer",
        "Erev Shavuos" => "Erew Schawuot",
        "Shavuos I" => "1. Tag Schawuot",
        "Shavuos II" => "2. Tag Schawuot",
        "Tzom Tammuz" => "Fasten 17. Tammus",
        "Tish’a B’Av" | "Tish’a B’Av (observed)" => "Fasten 9. Aw",
        "Tzom Tammuz (observed)" => "Fasten 17. Tammus",
        "Tzom Gedaliah (observed)" => "Fasten Gedalja",
        "Tu B’Av" => "Tu beAw",
        _ => return None,
    })
}

/// Moderne Tage nur in Jahren, für die Hebcal sie kennt und die Regeln gleich sind.
fn map_modern(title: &str) -> Option<&'static str> {
    Some(match title {
        "Yom HaShoah" => "Jom HaSchoa",
        "Yom HaZikaron" => "Jom HaSikaron",
        "Yom HaAtzma’ut" => "Jom HaAzmaut",
        "Yom Yerushalayim" => "Jom Jeruschalajim",
        _ => return None,
    })
}

#[test]
fn matches_hebcal_diaspora() {
    let dir = data_dir();
    if !dir.is_dir() {
        eprintln!("übersprungen: {} fehlt", dir.display());
        return;
    }
    let mut years_checked = 0;
    let mut problems = Vec::new();
    for year in 5650..=5900 {
        let path = dir.join(format!("h{year}.json"));
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        let items = v["items"].as_array().unwrap();
        let days = year_days(year);

        // Erwartete (Datum, Name) aus Hebcal
        let mut expected = BTreeSet::new();
        let mut expected_parasha = BTreeSet::new();
        for i in items {
            let title = i["title"].as_str().unwrap();
            let date = i["date"].as_str().unwrap();
            let date = &date[..10];
            let cat = i["category"].as_str().unwrap();
            if cat == "holiday" {
                if title.starts_with("Rosh Hashana ")
                    && !title.contains("II")
                    && !title.contains("LaBeheimos")
                {
                    expected.insert((date.to_string(), "1. Tag Rosch Haschana".to_string()));
                } else if let Some(n) = map_title(title) {
                    expected.insert((date.to_string(), n.to_string()));
                } else if year >= 5760
                    && let Some(n) = map_modern(title)
                {
                    expected.insert((date.to_string(), n.to_string()));
                }
            } else if cat == "parashat" {
                expected_parasha.insert(date.to_string());
            }
        }

        let mut got = BTreeSet::new();
        let mut got_parasha = BTreeSet::new();
        for d in &days {
            let (y, m, dd) = d.gregorian;
            let date = format!("{y:04}-{m:02}-{dd:02}");
            for h in &d.holidays {
                let n = h.name_de.as_str();
                let in_domain = n != "Erew Chanukka"
                    && !n.contains("Chanukka")
                    && (year >= 5760
                        || !matches!(
                            n,
                            "Jom HaSchoa" | "Jom HaSikaron" | "Jom HaAzmaut" | "Jom Jeruschalajim"
                        ));
                if in_domain {
                    got.insert((date.clone(), n.to_string()));
                }
            }
            if d.parasha.is_some() {
                got_parasha.insert(date);
            }
        }
        // Hebcal-Bereich: nur Tage im Bereich unserer Liste vergleichen
        let first = {
            let (y, m, d) = days.first().unwrap().gregorian;
            format!("{y:04}-{m:02}-{d:02}")
        };
        let last = {
            let (y, m, d) = days.last().unwrap().gregorian;
            format!("{y:04}-{m:02}-{d:02}")
        };
        let inside = |s: &String| *s >= first && *s <= last;
        let expected: BTreeSet<_> = expected.into_iter().filter(|(d, _)| inside(d)).collect();
        for e in expected.difference(&got) {
            problems.push(format!("{year}: fehlt bei uns {e:?}"));
        }
        for g in got.difference(&expected) {
            problems.push(format!("{year}: zu viel bei uns {g:?}"));
        }
        let expected_parasha: BTreeSet<_> = expected_parasha.into_iter().filter(inside).collect();
        for e in expected_parasha.difference(&got_parasha) {
            problems.push(format!("{year}: Parascha fehlt am {e}"));
        }
        for g in got_parasha.difference(&expected_parasha) {
            problems.push(format!("{year}: Parascha zu viel am {g}"));
        }
        // Rosch Chodesch: Anzahl der Tage
        let rc_expected = items
            .iter()
            .filter(|i| i["category"] == "roshchodesh")
            .count();
        let rc_got = days.iter().filter(|d| d.rosh_chodesh.is_some()).count();
        let _ = RoshChodesh::Single;
        if rc_expected > 0 && rc_got < rc_expected.saturating_sub(2) {
            problems.push(format!(
                "{year}: Rosch Chodesch Monate {rc_expected} vs Tage {rc_got}"
            ));
        }
        years_checked += 1;
    }
    println!(
        "{years_checked} Jahre geprüft, {} Abweichungen",
        problems.len()
    );
    for p in problems.iter().take(12) {
        println!("  {p}");
    }
    assert!(problems.is_empty(), "{} Abweichungen", problems.len());
}
