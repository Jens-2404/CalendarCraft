//! Übergangslösung für den Spike. In Phase 1 ersetzt durch Fluent-Dateien
//! in `cc-i18n`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    De,
    En,
    He,
}

#[derive(Debug, Clone, Copy)]
pub enum Text {
    Calculator,
    CreateCalendar,
    GregorianDate,
    Error,
    Language,
    HostBrowser,
    HostDesktop,
}

impl Lang {
    pub const ALL: [Lang; 3] = [Lang::De, Lang::En, Lang::He];

    pub fn code(self) -> &'static str {
        match self {
            Lang::De => "de",
            Lang::En => "en",
            Lang::He => "he",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Lang::De => "Deutsch",
            Lang::En => "English",
            Lang::He => "עברית",
        }
    }

    pub fn is_rtl(self) -> bool {
        self == Lang::He
    }

    pub fn text(self, key: Text) -> &'static str {
        use Text::*;
        match (self, key) {
            (Lang::De, Calculator) => "Datumsrechner",
            (Lang::En, Calculator) => "Date calculator",
            (Lang::He, Calculator) => "מחשבון",
            (Lang::De, CreateCalendar) => "Kalender gestalten",
            (Lang::En, CreateCalendar) => "Design calendar",
            (Lang::He, CreateCalendar) => "יצירת לוח שנה",
            (Lang::De, GregorianDate) => "Gregorianisches Datum",
            (Lang::En, GregorianDate) => "Gregorian date",
            (Lang::He, GregorianDate) => "תאריך לועזי",
            (Lang::De, Error) => "Fehler",
            (Lang::En, Error) => "Error",
            (Lang::He, Error) => "שגיאה",
            (Lang::De, Language) => "Sprache",
            (Lang::En, Language) => "Language",
            (Lang::He, Language) => "שפה",
            (Lang::De, HostBrowser) => "läuft im Browser",
            (Lang::En, HostBrowser) => "running in the browser",
            (Lang::He, HostBrowser) => "פועל בדפדפן",
            (Lang::De, HostDesktop) => "läuft als Desktop-App",
            (Lang::En, HostDesktop) => "running as desktop app",
            (Lang::He, HostDesktop) => "פועל כיישום שולחני",
        }
    }
}

/// Hebräischer Monatsname zum CLDR-Monatscode.
pub fn hebrew_month(code: &str, leap_year: bool, lang: Lang) -> &'static str {
    let (de, en, he) = match code {
        "M01" => ("Tischri", "Tishrei", "תשרי"),
        "M02" => ("Cheschwan", "Cheshvan", "חשון"),
        "M03" => ("Kislew", "Kislev", "כסלו"),
        "M04" => ("Tewet", "Tevet", "טבת"),
        "M05" => ("Schwat", "Shevat", "שבט"),
        "M05L" => ("Adar I", "Adar I", "אדר א׳"),
        "M06" if leap_year => ("Adar II", "Adar II", "אדר ב׳"),
        "M06" => ("Adar", "Adar", "אדר"),
        "M07" => ("Nisan", "Nisan", "ניסן"),
        "M08" => ("Ijar", "Iyar", "אייר"),
        "M09" => ("Siwan", "Sivan", "סיון"),
        "M10" => ("Tammus", "Tammuz", "תמוז"),
        "M11" => ("Aw", "Av", "אב"),
        "M12" => ("Elul", "Elul", "אלול"),
        _ => ("?", "?", "?"),
    };
    match lang {
        Lang::De => de,
        Lang::En => en,
        Lang::He => he,
    }
}
