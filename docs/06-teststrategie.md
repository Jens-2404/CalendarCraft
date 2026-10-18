# CalendarCraft – Teststrategie

Version 0.4 (Entwurf), Stand 18.10.2026

Ein Kalender mit falschem Datum ist wertlos, auch wenn er gut aussieht. Die Tests konzentrieren sich deshalb auf die fachliche Korrektheit und darauf, dass Vorschau, Export und Plattformen dasselbe Ergebnis liefern.

## 1. Testebenen

| Ebene | Werkzeug | Inhalt | Läuft |
|---|---|---|---|
| Unit-Tests | `cargo test` | Einzelfunktionen, Regeltypen, Zahlformate | jeder Push |
| Property-Tests | `proptest` | Invarianten über Millionen zufälliger Daten | jeder Push (reduziert), nightly (voll) |
| Referenztests | `cargo test` + `testdata/` | Vergleich mit externen Referenzdaten | jeder Push |
| Kreuztests | `cargo test` | eigene Implementierung ↔ ICU4X | jeder Push |
| Snapshot-Tests | `insta`, Bildvergleich | Layout-Baum, SVG, PNG von Musterprojekten | jeder Push |
| WASM-Tests | `wasm-bindgen-test` | Kern läuft in Browser-Engines identisch | jeder Push |
| E2E Browser | Playwright (Chromium, Firefox, WebKit) | Hauptabläufe der Oberfläche | PR |
| E2E Desktop | WebdriverIO + `tauri-driver` | Dateidialog, Export, Autosave | PR (Windows, Linux), Release (macOS manuell) |
| Manuelle Abnahme | Checkliste | Druckergebnis auf Papier, Installer, Signatur | je Release |

Hinweis: `tauri-driver` unterstützt macOS derzeit nicht; dort wird bis auf Weiteres manuell und über die gemeinsamen Browser-Tests (WebKit) abgesichert.

## 2. Kalenderkern

Invarianten (Property-Tests):

- `to_fixed(from_fixed(d)) == d` für alle Systeme im Bereich −10 000 bis +10 000 Jahre um heute.
- Aufeinanderfolgende Tageszahlen ergeben aufeinanderfolgende Kalenderdaten (kein Tag fehlt oder doppelt).
- Jüdische Jahreslänge ∈ {353, 354, 355, 383, 384, 385}; Rosch HaSchana nie Sonntag, Mittwoch, Freitag.
- Pessach (15. Nisan) nie Montag, Mittwoch, Freitag.
- Wochentag stimmt in allen Systemen für dieselbe Tageszahl überein.

Referenztests:

- Jüdisch: alle Rosch-HaSchana-Daten 1900–2200 und alle Feiertage 1950–2100 gegen Hebcal-Export.
- Paraschot und Haftarot 1950–2100, Diaspora und Israel, gegen Hebcal-Export.
- Molad-Ankündigungen gegen Hebcal.
- Ostern: westlich und orthodox 1583–2400 gegen eine zweite, unabhängige Implementierung; Stichproben gegen veröffentlichte Ostertabellen.

## 3. Feiertage

- Jede Regel hat mindestens drei Testjahre, darunter jeder Sonderfall (z. B. Ta'anit Esther auf Schabbat, Jom HaAtzma'ut am Montag, 17. Tammus auf Schabbat).
- Gesetzliche Feiertage: Tabelle je Bundesland und Jahr 2000–2050 als Golden File; Regeln mit `valid_from` (Frauentag Berlin 2019, MV 2023, Weltkindertag Thüringen 2019) bekommen Tests direkt vor und nach dem Stichtag.
- Diaspora/Israel jeweils in beiden Einstellungen.
- Israelische Gedenktage mit allen Verschiebungsfällen und `valid_from`-Grenzen (z. B. Sigd 2008, Jom HaAlija 2016).

## 3a. Zmanim

- Referenzorte: Frankfurt, Berlin, München, Hamburg, Jerusalem, Tel Aviv, New York, London, Buenos Aires (Südhalbkugel), Tromsø (hohe Breite), dazu ein Ort, der nur über Längen- und Breitengrad eingegeben wird.
- Für Schabbatende jede angebotene Art (Winkel und feste Minuten) gegen MyZmanim, Hebcal oder KosherJava, soweit dort verfügbar.
- Je Ort ein volles Jahr täglicher Zeiten gegen Hebcal und KosherJava mit identischen Parametern; Toleranz 1 Minute.
- Tage mit Zeitumstellung, Polartag/-nacht und Fälle, in denen ein Sonnentiefstand nicht erreicht wird, haben eigene Tests.
- Kerzenzündung am zweiten Feiertagsabend und an Schabbat-Ausgang vor einem Feiertag.
- Erev-/Nacht-Einträge liegen auf dem richtigen gregorianischen Tag.

## 3b. Referenz: Gemeindekalender Frankfurt 5784

Das PDF `Kalender_5784.pdf` (Jüdische Gemeinde Frankfurt, A4 quer, 2 Seiten) enthält maschinenlesbaren Text. Daraus entsteht eine Prüfdatei mit je Tag: gregorianisches und hebräisches Datum, Tagesname, Zeiten, Symbole. Sie dient als Referenz für:

- Datumsumrechnung und hebräischer Wochentag je Zeile
- Feiertage, Fasttage, Parascha, besondere Schabbatot, Rosch Chodesch, Omer, Chanukka-Tage
- Zeiten: Kerzenzünden, Schabbatende, Fastenbeginn/-ende, Chametz-Zeiten, Mitternacht (Toleranz 1 Minute, Abweichungen einzeln erklärt)
- Staatliche Feiertage der Fußzeile (Hessen)

Rechtlicher Hinweis: Das PDF ist ein Werk der Gemeinde. Die Prüfdatei liegt nur privat unter `testdata/private/` (von Git ausgeschlossen). Vor einer Open-Source-Veröffentlichung wird die Gemeinde um Zustimmung gebeten oder die Prüfdaten werden durch Hebcal-Daten ersetzt.

Zusätzlich ein Layout-Test: Das Projekt „Gemeinde Frankfurt 5784" erzeugt ein PDF, dessen Struktur dem Beispiel entspricht (13 Monatsspalten auf 2 Seiten, 6 + 7, Kopf, Fußzeile, Zeilenstile); Prüfung über Snapshot des Layout-Baums und Sichtprüfung durch den Auftraggeber.

## 4. Layout und Rendering

- Musterprojekte unter `testdata/projects/` (je Kalenderform mindestens eins, dazu Grenzfälle: Monat mit 6 Wochenzeilen, Tag mit fünf Feiertagen, jüdisches Schaltjahr als Hauptkalender, nur RTL-Inhalte, jede Kalenderform in A6, A4, A0 und US Letter, mit und ohne Fotos).
- Snapshot des Layout-Baums (`insta`) fängt unbeabsichtigte Verschiebungen.
- PNG-Vergleich mit Toleranz für visuelle Regression; Änderungen werden im PR als Vorher/Nachher-Bild angezeigt.
- PDF-Prüfungen: Seitenanzahl, Seitenmaße, eingebettete Schriften, Text extrahierbar (Durchsuchbarkeit), Beschnittbox gesetzt.
- Gleiches Projekt auf Linux, Windows, macOS und im Browser ergibt identisches SVG (Determinismus, NF-6).

## 5. Oberfläche

E2E-Szenarien (Auswahl):

1. Assistent mit Voreinstellungen durchlaufen, PDF exportieren, Datei prüfen.
2. Hauptkalender auf jüdisch umstellen; Schaltjahr erzeugt 13 Monatsblätter.
3. Ort von Frankfurt auf Jerusalem wechseln; Kerzenzündezeiten, Diaspora/Israel-Einstellung und Paraschot ändern sich entsprechend.
4. Eigenes Ereignis „jährlich nach hebräischem Datum" anlegen; erscheint im Folgejahr am richtigen gregorianischen Tag.
5. Projekt speichern, App schließen, wieder öffnen; Zustand identisch.
6. Absturz simulieren (Tab schließen ohne Speichern); Wiederherstellung angeboten.
7. Bedienung nur mit Tastatur; automatischer Barrierefreiheitstest mit axe-core.
8. Oberfläche auf Hebräisch umschalten; alle Szenarien laufen auch mit gespiegelter (RTL-)Oberfläche.
9. Fotos aktivieren, Bild in einen Monat einfügen, Format auf A0 ändern; Auflösungswarnung erscheint.

## 6. Leistung

Benchmarks mit `criterion` (nativ) und Messungen im Browser in der CI; Grenzwerte aus PRD NF-2 und NF-3. Eigener Test für die Schwäche des Vorbilds: Jahr und alle 12 bzw. 13 Monate eingeben, bis die Vorschau steht, in unter 2 Sekunden (Taschen-, Wand- und Tischkalender, A4 und A0, ohne Fotos). Ein PR, der die WASM-Größe um mehr als 5 % erhöht, wird markiert.

## 7. Fachliche Prüfung durch Menschen

Der Auftraggeber prüft nach jeder Phase die Ergebnisse (CLI-Ausgaben, PDFs) gegen seine Erfahrung mit Luach Pirsum und gedruckten Gemeindekalendern. Vor Release 1.0 prüft zusätzlich mindestens eine weitere kundige Person einen exportierten Jahreskalender auf Vollständigkeit, Bezeichnungen, Schreibweisen und Zmanim. Rückmeldungen werden als Issues mit Quellenangabe erfasst.
