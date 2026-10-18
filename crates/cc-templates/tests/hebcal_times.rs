//! Kerzenzünden vor Schabbat/Jom Tov gegen Hebcal (Frankfurt, 30 Minuten vor
//! Sonnenuntergang). Daten nur lokal unter testdata/private (von Git
//! ausgeschlossen); fehlen sie, wird der Test übersprungen.

use cc_astro::{Location, Profile};
use cc_holidays::year_days;
use cc_templates::times::{MainTime, day_times};

#[test]
fn candle_lighting_matches_hebcal_frankfurt() {
    let (loc, p) = (Location::frankfurt(), Profile::frankfurt());
    let mut compared = 0;
    for year in [5784, 5787] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../testdata/private/hebcal-{year}-ffm.json"));
        let Ok(text) = std::fs::read_to_string(&path) else {
            eprintln!("übersprungen: {} fehlt", path.display());
            continue;
        };
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        let hebcal: std::collections::BTreeMap<String, i32> = v["items"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|i| i["category"] == "candles")
            .map(|i| {
                let d = i["date"].as_str().unwrap();
                let (date, time) = d.split_once('T').unwrap();
                let h: i32 = time[0..2].parse().unwrap();
                let m: i32 = time[3..5].parse().unwrap();
                (date.to_string(), h * 60 + m)
            })
            .collect();
        let days = year_days(year);
        for i in 0..days.len() {
            let Some(MainTime::Candles(ours)) = day_times(&days, i, &loc, &p).main else {
                continue;
            };
            let (y, m, d) = days[i].gregorian;
            let key = format!("{y:04}-{m:02}-{d:02}");
            let theirs = hebcal
                .get(&key)
                .unwrap_or_else(|| panic!("{key}: bei Hebcal fehlt Kerzenzünden"));
            assert!(
                (ours - theirs).abs() <= 1,
                "{key}: ours {ours}, Hebcal {theirs}"
            );
            compared += 1;
        }
    }
    println!("{compared} Kerzenzündezeiten verglichen");
}
