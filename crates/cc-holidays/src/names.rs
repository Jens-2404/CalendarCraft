//! Deutsche Namen. Schreibweise nach dem Gemeindekalender Frankfurt 5784
//! (Kalender_5784.pdf); die Umschrift wird später über cc-i18n wählbar.

/// Deutsche Namen der 54 Abschnitte, Index 0 = Bereschit … 51 = Wajelech,
/// 52 = Haasinu, 53 = Wesot Habracha.
pub const PARASHA_DE: [&str; 54] = [
    "Bereschit",
    "Noach",
    "Lech Lecha",
    "Wajera",
    "Chaje Sarah",
    "Toldot",
    "Wajeze",
    "Wajischlach",
    "Wajeschew",
    "Mikez",
    "Wajigasch",
    "Wajechi",
    "Schemot",
    "Waera",
    "Bo",
    "Beschalach",
    "Jitro",
    "Mischpatim",
    "Teruma",
    "Tezawe",
    "Ki Tissa",
    "Wajakhel",
    "Pekudei",
    "Wajikra",
    "Zaw",
    "Schmini",
    "Tasria",
    "Mezora",
    "Acharei Mot",
    "Kedoschim",
    "Emor",
    "Behar",
    "Bechukotaj",
    "Bamidbar",
    "Nasso",
    "Beha’alot’cha",
    "Schlach Lecha",
    "Korach",
    "Chukat",
    "Balak",
    "Pinchas",
    "Mattot",
    "Mass’e",
    "Dewarim",
    "Waetchanan",
    "Ekew",
    "Reeh",
    "Schoftim",
    "Ki Teze",
    "Ki Tawo",
    "Nizawim",
    "Wajelech",
    "Haasinu",
    "Wesot Habracha",
];

/// Name eines Abschnitts oder Doppelabschnitts, z. B. "Nizawim – Wajelech".
pub fn parasha_name_de(first: u8, pair: bool) -> String {
    let a = PARASHA_DE[usize::from(first)];
    if pair {
        format!("{a} – {}", PARASHA_DE[usize::from(first) + 1])
    } else {
        a.to_string()
    }
}
