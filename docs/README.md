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

## Oberfläche zum Gestalten (18.10.2026)

Alle Kalenderarten lassen sich jetzt ohne Kommandozeile gestalten, im Browser und in der Desktop-App: Kalenderart, Jahr, Startmonat, Anzahl Monate, Seiten, Format, Hoch/Quer, Schrift, Titelschrift, Schriftgröße, Gemeindename und Adresse, Fotos und eigene Einträge. Die Vorschau aktualisiert sich nach jeder Änderung (etwa 0,6 Sekunden), „PDF speichern“ erzeugt das PDF.

Technik und Abweichungen von der Planung:

- Der Kalender wird in einem Web Worker gesetzt (apps/worker, wie in ADR-002 vorgesehen), auch in der Desktop-App (gleiches Verhalten, keine doppelte Logik). Die Oberfläche schickt nur Einstellungen, Fotos und Schriftdateien an den Worker und bekommt Seitenzahl, SVG der angezeigten Seite und beim Speichern das PDF zurück. Sie bleibt beim Neusetzen bedienbar.
- Die Desktop-App ergänzt Systemschriften (die Oberfläche lädt die Datei über Tauri und gibt sie an den Worker) und den Speichern-Dialog.
- Oberfläche und Worker sind getrennte WebAssembly-Module: die Oberfläche 178 KB gzip (ohne Typst), der Worker 10,5 MB gzip (Typst mit Schriften). Die Oberfläche ist sofort bedienbar, die erste Vorschau erscheint, sobald der Worker geladen ist. Die CI prüft beide Grenzen getrennt (Oberfläche unter 1 MB, Worker höchstens 12 MB). Mitgeliefert werden nur Libertinus Serif und DejaVu Sans Mono (statt aller Typst-Schriften); Lizenzen in assets/fonts/LICENSES.md.
- Die Gestaltungsseite ist im Prototyp nur auf Deutsch.

## Monatskalender nach Ihren Beispielen (18.10.2026)

`cc-cli monatskalender` erzeugt einen Monat pro Seite (Sonntag bis Schabbat) in zwei Stilen nach Ihren Vorlagen:

- **klassisch** (A4 quer): gregorianischer Tag groß, hebräischer Tag in Buchstaben, hebräisches Datum hochkant, Einträge rechts oben, Zeiten mit Symbolen, Schabbat und Feiertage grau, Legende am rechten Rand.
- **foto** (A4 hoch): Fotofläche mit 1 bis 6 Fotos, Monatsraster mit grauem Balken für das hebräische Datum, Zeiten unten rechts.

Neu im Kern: Schabbat Mewarchim (gegen Hebcal über 251 Jahre geprüft), Gedenktag an das Massaker vom 7. Oktober (ab 5786, auf Sonntag verschoben, wenn er auf Schabbat fällt), Zeitumstellung, hebräische Zahlbuchstaben, eigene Einträge aus einer Textdatei (Jahrzeit nach hebräischem Datum, Geburtstage, Termine, Ferien).

Abgleich mit Ihren Beispielen für Oktober und September 2026: Feiertage, Parascha, Mewarchim und Kerzenzünden stimmen. Ihr Foto-Kalender rundet Schabbatende offenbar ab (einige Zeiten 1 Minute früher) und setzt das Fastenende früher an (Zom Gedalja 20:16 statt 20:21 nach Frankfurter Methode). Für eine andere Gemeinde müsste ein eigenes Zeitenprofil eingestellt werden; das ist noch nicht über die Kommandozeile wählbar.

Offen: Schulferien als Daten (bisher nur über die Eintragsdatei), Zeitenprofil und Rundung wählbar, Deckblatt.

## Jahreskalender im Stil von Kalenderpedia (18.10.2026)

`cc-cli jahreskalender --year 2027` erzeugt einen Jahreskalender Januar–Dezember: gregorianische Monate als Spalten, eine Zeile je Tag, Kalenderwochen, gesetzliche Feiertage (Hessen, rot) und dazu je Tag das hebräische Datum, jüdische Feiertage, Parascha, Omer, Kerzenzünden und Schabbatende. Er kann auf 1 oder 2 Seiten stehen und mit einem beliebigen Startmonat beginnen.

Für beide Vorlagen gibt es jetzt diese Einstellungen: Papierformat (A0–A5, US-Formate, eigene Maße), Hoch- oder Querformat, Schrift des Inhalts, Schrift der Monatsköpfe und Schriftgröße (Faktor). Aufruf siehe README im Projektordner.

Offen: Halbjahres-, Monats- und Wochenkalender; Bundesland wählbar (bisher Hessen); Bedienung in der App statt Kommandozeile.

## Prototyp Gemeindekalender (18.10.2026)

Mit `cc-cli gemeindekalender --year 5787 --out kalender.pdf` entsteht ein Kalender im Aufbau Ihres Frankfurter Beispiels (A4 quer, 2 Seiten, eine Spalte je Monat). Er läuft in unter 0,2 Sekunden und rechnet Feiertage, Wochenabschnitte und Zeiten selbst.

Geprüft:

| Prüfung | Ergebnis |
|---|---|
| Feiertage, Fasttage, Wochenabschnitte gegen Hebcal, 251 Jahre (5650–5900) | 0 Abweichungen |
| Gedruckte Zeiten Ihres 5784-PDFs (Kerzen, Schabbatende, Fasten, Chanukka, Chametz, Mitternacht) | 145 verglichen, 135 exakt, 10 um 1 Minute abweichend |
| Kerzenzünden 5784 und 5787 gegen Hebcal | 118 verglichen, alle ±1 Minute |

Die Frankfurter Methode ist aus den gedruckten Zeiten abgeleitet: Kerzenzünden 30 Minuten vor Sonnenuntergang (gerundet), Schabbatende bei 8,5° (aufgerundet), Fastenbeginn bei 16,1°, Fastenende und Chanukka-Lichter bei 7,083° (aufgerundet), Chametz-Zeiten nach 16,1° (abgerundet).

Unterschiede zum Vorbild und offene Punkte:

1. Kopfbereich: Logo und Adresse der Gemeinde fehlen (Platzhaltertext). Ein Gemeindeprofil lässt sich als JSON übergeben (`--profile`).
2. Schriften: Arial und Times New Roman von Windows statt der Schriften des Vorbilds; auf anderen Systemen fehlen sie noch.
3. Fußzeile von Seite 2 im Vorbild: „Staatliche Feiertage 2024: 7. April, 10. April, 1. Mai, 18. Mai, 29. Mai, 8. Juni“. Für Hessen gelten 29. März, 1. April, 1. Mai, 9. Mai, 20. Mai und 30. Mai; das Vorbild scheint hier einen Fehler zu enthalten. Der Prototyp gibt die hessischen Daten aus.
4. Im Vorbild steht am 3. Mai 2024 „Jom HaHazala“; das ist nicht berechnet, weil unklar ist, nach welcher Regel.
5. Nur Frankfurt, Diaspora, Hessen. Andere Orte, Zeitzonen außerhalb Mitteleuropas, Israel-Regeln und Hochformat folgen.

Ausgaben des Prototyps liegen in `ausgabe/` (von Git ausgeschlossen).

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
