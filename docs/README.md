# CalendarCraft – Planungsunterlagen

Stand 18.10.2026, Version 0.4 (Entwurf zur Prüfung)

CalendarCraft ist eine Rust-Anwendung für Browser und Desktop (Windows, macOS, Linux), mit der man Kalender in allen gängigen Formen und Papiergrößen gestaltet. Schwerpunkt sind der jüdische und der gregorianische Kalender mit Feiertagen, Paraschot und Zmanim sowie den gesetzlichen Feiertagen in Deutschland und Israel. Vorbild für Aufbau und Bedienung ist die frühere Software „Luach Pirsum".

## Dokumente

| Nr. | Dokument | Inhalt |
|---|---|---|
| 1 | [01-PRD.md](01-PRD.md) | Produktanforderungen: Ziele, Plattformen, Anforderungen mit Prioritäten, Papierformate, Abgrenzung, Risiken |
| 2 | [02-architektur.md](02-architektur.md) | Tech-Stack, Workspace-Struktur, Datenmodelle, Render-Pipeline, Architekturentscheidungen |
| 3 | [03-kalender-und-feiertage.md](03-kalender-und-feiertage.md) | Fachspezifikation: Rechenregeln, jüdische und israelische Feiertage, deutsche Feiertage, Paraschot, Zmanim, Darstellung, Referenzen |
| 4 | [04-ablaufplaene.md](04-ablaufplaene.md) | Nutzerablauf, Feiertagsberechnung, Vorschau/Export, Projektzustände, Entwicklungs-, CI- und Release-Ablauf |
| 5 | [05-roadmap.md](05-roadmap.md) | Phasen 0–6 mit Zielen, Abnahmekriterien und Schätzungen |
| 6 | [06-teststrategie.md](06-teststrategie.md) | Testebenen, Referenztests, Zmanim-Tests, Rendering- und E2E-Tests |
| 7 | [07-vorlage-gemeindekalender.md](07-vorlage-gemeindekalender.md) | Vorlage „Gemeindekalender" nach dem Beispiel des Auftraggebers (Frankfurt 5784): Aufbau, Zeilenstile, Zeiten, Abnahmetest |

## Entscheidungen (Antworten vom 17.10.2026)

| Thema | Entscheidung | Auswirkung |
|---|---|---|
| Gesetzliche Feiertage | Deutschland und Israel | Andere Länder entfallen |
| Islamischer Kalender | vorerst auslassen, ebenso islamische Feiertage | Aus MVP und 1.0 gestrichen; Architektur bleibt erweiterbar |
| Jüdischer Kalender | alles ins MVP: Kerzenzündezeiten, Paraschot, weitere Zmanim | Neue Phase 3 „Zmanim und Orte"; Transliteration aschkenasisch und sefardisch wählbar |
| Desktopkalender | Tischkalender und Bildschirmhintergrund | Tischkalender im MVP, Hintergrund in 1.0 |
| Druck | DIN A6 bis A0 und größer, US-Letter-Formate | Alle Formate im MVP; Kacheln großer Formate in 1.0 |
| Fotokalender | entscheidet der Anwender | Fotos als Option je Projekt im MVP |
| Oberflächensprachen | Deutsch, Englisch, Hebräisch | Hebräische RTL-Oberfläche im MVP |
| Lizenz | zuerst privat, später Open Source | Privates Repository; nur Open-Source-taugliche Abhängigkeiten; Signierung erst zur Veröffentlichung |
| Team | eine Person | Roadmap für eine Person, Phasen mit eigenständig nutzbarem Ergebnis |
| Zmanim-Orte | Alle deutschen Großgemeinden und große Gemeinden weltweit, dazu freie Eingabe von Längen- und Breitengrad | Ortsdatenbank plus Koordinateneingabe (PRD F-F8) |
| Kerzenzündung | Frankfurt am Main traditionell 30 Minuten vor Sonnenuntergang | Ortsvoreinstellung, Minuten frei wählbar (F-F7a) |
| Schabbatende | Auswahl wie auf myzmanim.com | Mehrere Berechnungsarten wählbar; Liste wird in Phase 0 abgeglichen (F-F7b) |
| Christliche Gedenktage | behalten, abschaltbar | Version 1.0 |
| Taschenkalender | Teil des Vorbilds Luach Pirsum | Vom MVP an dabei |
| Hosting | GitHub | Privates GitHub-Repository; GitHub Pages zur Veröffentlichung (siehe ADR-004) |

## Luach Pirsum: Stand

Aus Ihren Angaben: Windows-Programm von tichnut.de (David und Ulrich Greve), etwa 2001, in Java geschrieben. Hauptfenster mit Rechner und Kalendererstellung (Wand-, Taschenkalender u. a.). Man gab das gregorianische oder hebräische Jahr und die gewünschten Monate an. Für Gemeinden und Privatpersonen. Schwäche: Der Aufbau der Kalenderseite dauerte sehr lange.

Das ist in PRD 5.7 und im Nutzerablauf eingearbeitet, die Geschwindigkeit als messbare Anforderung (Vorschau unter 2 Sekunden). Auf tichnut.de wird Luach Pirsum heute nicht mehr aufgeführt, und sonst habe ich online nichts gefunden. Weitere Einzelheiten (Menüs, Bildschirmaufteilung, Funktionsumfang) klären wir mit dem Oberflächen-Prototyp in Phase 0. Ob bei den Autoren noch Unterlagen oder eine Installationsdatei existieren, können nur Sie erfragen.

## Offene Punkte

1. **Zeitbudget:** Ihre Antwort „Ja" passt nicht zur Frage (Stunden pro Woche). Ich plane weiter mit einer Person in Vollzeit; bitte nennen Sie einen Richtwert, falls es weniger ist.
2. **Schabbatende wie myzmanim.com:** Die Seite war für mich nicht vollständig abrufbar. Falls Sie die Optionsliste dort zur Hand haben (Namen, Winkel, Minuten), schicken Sie sie mir, sonst gleiche ich sie in Phase 0 ab.
3. **Orte:** Die Liste der deutschen Großgemeinden in Spezifikation 3.5 ist ein Vorschlag. Bitte streichen oder ergänzen Sie, was Ihnen fehlt.
4. **Prototyp statt Handbuch:** Sind Sie einverstanden, dass Phase 0 einen klickbaren Prototyp aus Ihrer Beschreibung baut, den Sie so lange korrigieren, bis er Luach Pirsum entspricht?

## Vorbild-PDF „Kalender 5784" (18.10.2026)

Ihr Beispiel ist der Wandkalender der Jüdischen Gemeinde Frankfurt: A4 quer, zwei Seiten, eine Spalte je jüdischem Monat, eine Zeile je Tag, Kopfbereich mit Logo und Adresse, Legende und Liste der staatlichen Feiertage. Ausgewertet in [07-vorlage-gemeindekalender.md](07-vorlage-gemeindekalender.md); daraus sind neue Anforderungen geworden (F-V11, F-G11 bis F-G14, F-F15 bis F-F18) und ein Abnahmetest.

Offen dazu:

1. Dürfen die Namen der Tage frei umbenannt werden (das Beispiel schreibt z. B. „Schabat Schuwa" und „Haasinu" in eigener Schreibweise)?
2. Soll ein Gemeindeprofil (Logo, Adresse, Schreibweisen, Zmanim-Methode) als Datei speicherbar sein, damit jedes Jahr nur die Jahreszahl geändert wird? Vorschlag: ja.
3. Das PDF ist ein Werk der Gemeinde. Für die private Prüfung nutze ich es nur lokal (nicht in Git). Vor einer Veröffentlichung sollte die Gemeinde zustimmen.
4. Das Beispiel nutzt kostenpflichtige Schriften (Favorit). CalendarCraft liefert freie Schriften mit, das Aussehen weicht daher leicht ab.

## Stand Phase 0 (17.10.2026)

| Aufgabe | Stand |
|---|---|
| Workspace (Crates, Apps, Spikes), Git-Repository | erledigt, noch nicht committet |
| CI (fmt, clippy, cargo-deny, Tests auf 3 Systemen, WASM-Build mit Größenprüfung, Desktop-Installer) | Workflow geschrieben; läuft erst, wenn das Repository auf GitHub liegt. Lokal sind fmt, clippy, deny und Tests grün |
| Spike ADR-001 (Leptos + Tauri) | angenommen; Browser geprüft (Umrechnung, RTL-Umschaltung), Windows-Installer gebaut (MSI 2,1 MB, Setup 1,5 MB). Das Desktop-Fenster selbst ist noch nicht angesehen |
| Spike ADR-002 (Render-Engine) | angenommen: Typst. Messwerte in [02-architektur.md](02-architektur.md#adr-002-render-engine) |
| Oberflächen-Prototyp nach Luach Pirsum | offen |
| Abgleich der Optionen von myzmanim.com | offen |

Vorschlag zur Anpassung von NF-3 (Ihre Entscheidung): Typst ist als WebAssembly 8,2 MB groß (gzip). Statt „initialer Download unter 8 MB" gilt künftig: Die Oberfläche lädt in unter 1 MB und ist sofort bedienbar; die Render-Engine lädt im Hintergrund nach und ist höchstens 10 MB groß (gzip).

## Nächster Schritt

Restliche Aufgaben aus Phase 0: Oberflächen-Prototyp nach Luach Pirsum und Abgleich der Zmanim-Optionen. Danach Phase 1 (Kalenderkern).
