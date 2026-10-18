//! Übersetzungen und Zahlformate. Stand: hebräische Zahlbuchstaben.

const ONES: [&str; 10] = ["", "א", "ב", "ג", "ד", "ה", "ו", "ז", "ח", "ט"];
const TENS: [&str; 10] = ["", "י", "כ", "ל", "מ", "נ", "ס", "ע", "פ", "צ"];
const HUNDREDS: [&str; 5] = ["", "ק", "ר", "ש", "ת"];

/// Hebräische Zahlbuchstaben für 1–999 (Tausender werden weggelassen, wie
/// bei Jahreszahlen üblich: 5787 → תשפ״ז). 15 und 16 werden als ט״ו und
/// ט״ז geschrieben. Mit `marks` stehen Geresch (׳) bzw. Gerschajim (״).
pub fn hebrew_numeral(n: u32, marks: bool) -> String {
    let mut n = n % 1000;
    if n == 0 {
        return String::new();
    }
    let mut letters: Vec<&str> = Vec::new();
    while n >= 400 {
        letters.push("ת");
        n -= 400;
    }
    letters.push(HUNDREDS[(n / 100) as usize]);
    n %= 100;
    match n {
        15 => letters.extend(["ט", "ו"]),
        16 => letters.extend(["ט", "ז"]),
        _ => {
            letters.push(TENS[(n / 10) as usize]);
            letters.push(ONES[(n % 10) as usize]);
        }
    }
    let letters: Vec<&str> = letters.into_iter().filter(|l| !l.is_empty()).collect();
    if !marks {
        return letters.concat();
    }
    match letters.len() {
        1 => format!("{}׳", letters[0]),
        k => format!("{}״{}", letters[..k - 1].concat(), letters[k - 1]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_numbers() {
        assert_eq!(hebrew_numeral(1, false), "א");
        assert_eq!(hebrew_numeral(15, false), "טו");
        assert_eq!(hebrew_numeral(16, true), "ט״ז");
        assert_eq!(hebrew_numeral(21, false), "כא");
        assert_eq!(hebrew_numeral(30, false), "ל");
        assert_eq!(hebrew_numeral(25, true), "כ״ה");
        assert_eq!(hebrew_numeral(1, true), "א׳");
    }

    #[test]
    fn years() {
        assert_eq!(hebrew_numeral(5787, true), "תשפ״ז");
        assert_eq!(hebrew_numeral(5784, true), "תשפ״ד");
        assert_eq!(hebrew_numeral(5800, true), "ת״ת");
        assert_eq!(hebrew_numeral(5715, true), "תשט״ו");
    }
}
