# CalendarCraft – Product Requirements Document (PRD)

Version 0.3 (Entwurf zur Prüfung), Stand 17.10.2026

Änderungen in 0.3: Angaben zu Luach Pirsum eingearbeitet (5.7); Taschenkalender ins MVP; Kerzenzündung und Schabbatende nach Ort bzw. myzmanim.com (F-F7a/b); Ortsdatenbank mit deutschen Großgemeinden (F-F8); Geschwindigkeit der Kalendererstellung als Anforderung (NF-2).

Änderungen in 0.2: Islamischer Kalender vorerst gestrichen; gesetzliche Feiertage Israel ergänzt; Paraschot, Zmanim und alle jüdischen Zusatzangaben ins MVP; Papierformate DIN A6 bis über A0 sowie US-Formate; Fotos als Option des Anwenders im MVP; Oberfläche auf Deutsch, Englisch und Hebräisch; zunächst private Nutzung. Vorbild für Aufbau und Bedienung ist die frühere Software „Luach Pirsum" (Abschnitt 5.7, Details stehen noch aus).

## 1. Zusammenfassung

CalendarCraft ist eine in Rust geschriebene Anwendung, mit der man druckfertige und digitale Kalender gestaltet. Sie läuft im Browser (als WebAssembly-Anwendung) und als Desktop-Programm unter Windows, macOS und Linux. Beide Varianten nutzen denselben Rust-Kern und dieselbe Oberfläche.

Fachlich geht es um den jüdischen und den gregorianischen Kalender. Die App rechnet Daten zwischen beiden um, berechnet jüdische Feiertage, Fasttage, Wochenabschnitte und Zeiten (Zmanim) sowie die gesetzlichen Feiertage in Deutschland und Israel, und stellt beide Kalender kombiniert dar. Die Architektur bleibt offen für weitere Kalendersysteme; der islamische Kalender kann später ergänzt werden.

## 2. Ausgangslage und Ziel

Gängige Kalender-Tools behandeln den jüdischen Kalender meist als Nebensache: Feiertage fehlen oder sind falsch, Zmanim und Paraschot gibt es nicht, und Kalenderformen wie Taschen- oder Tischkalender gibt es nur in festen Vorlagen. Gemeinden und Privatleute setzen solche Kalender heute oft von Hand.

Ziel ist ein Werkzeug, das

- korrekte Daten, Feiertage und Zeiten liefert und die Rechenregeln offenlegt,
- jede gängige Kalenderform in jeder gängigen Papiergröße erzeugt,
- druckfähige PDFs und digitale Formate exportiert,
- ohne Server und ohne Konto funktioniert (Daten bleiben beim Nutzer).

## 3. Zielgruppen

Zunächst ist die App für den privaten Gebrauch des Auftraggebers gedacht. Eine spätere Veröffentlichung als Open Source ist geplant, sobald alles funktioniert. Die Anforderungen sind so gefasst, dass sie danach auch passen für:

| Gruppe | Typischer Bedarf |
|---|---|
| Privatpersonen und Familien | Familien- oder Wandkalender mit Fotos, Geburtstagen, Jahrzeiten und Feiertagen |
| Jüdische Gemeinden und Organisationen | Jahreskalender mit Feiertagen, Fasttagen, Paraschot, Kerzenzündezeiten für den eigenen Ort |
| Schulen und Einrichtungen | Kalender mit deutschen und jüdischen Feiertagen |
| Druckereien, Grafiker | Vorlagen in allen Formaten, mit Beschnitt und Schnittmarken |

## 4. Plattformen

| Plattform | Umsetzung | Mindestanforderung |
|---|---|---|
| Browser | WebAssembly + Web-Oberfläche, installierbar als PWA, offline nutzbar | Aktuelle Versionen von Chrome, Edge, Firefox, Safari (jeweils die letzten zwei Hauptversionen) sowie darauf basierende Browser. Mobile Browser werden unterstützt, die Bedienung ist für Desktop-Bildschirme optimiert. |
| Windows | Tauri-Desktop-App | Windows 10 (ab 1809) und 11, x64 und ARM64 |
| macOS | Tauri-Desktop-App | macOS 11 oder neuer, Intel und Apple Silicon |
| Linux | Tauri-Desktop-App | Aktuelle Distributionen mit WebKitGTK 4.1 (z. B. Ubuntu 22.04+, Fedora 38+), Pakete als AppImage, .deb, .rpm |

Browser ohne WebAssembly-Unterstützung (Internet Explorer, sehr alte Versionen) lassen sich technisch nicht unterstützen. Die App zeigt dort einen Hinweis statt einer leeren Seite.

## 5. Funktionale Anforderungen

Priorität: M = Muss (MVP), S = Soll (Version 1.0), K = Kann (später).

### 5.1 Kalendersysteme und Umrechnung

| ID | Anforderung | Prio |
|---|---|---|
| F-K1 | Gregorianischer Kalender für die Jahre 1583–2400, proleptisch auch davor | M |
| F-K2 | Jüdischer Kalender nach den festen Rechenregeln (Molad, Verschiebungsregeln), 19-jähriger Zyklus mit Schaltmonat Adar I | M |
| F-K3 | Umrechner: beliebiges Datum in beide Systeme, inkl. Wochentag, ISO-Kalenderwoche, Parascha der Woche, Jahrestyp | M |
| F-K4 | Jüdische Tage beginnen mit Sonnenuntergang; Feiertage zeigen den Vorabend („Erev") als eigenen Eintrag; Umrechner mit Schalter „nach Sonnenuntergang" | M |
| F-K5 | Julianischer Kalender (für orthodoxe Ostern) | K |
| F-K6 | Weitere Systeme über dieselbe Schnittstelle nachrüstbar, zuerst der islamische Kalender | K |

### 5.2 Feiertage, Zeiten und Ereignisse

| ID | Anforderung | Prio |
|---|---|---|
| F-F1 | Jüdische Feiertage, Fast- und Gedenktage inkl. Verschiebungsregeln, umschaltbar Diaspora/Israel | M |
| F-F2 | Gesetzliche Feiertage Israel (Ruhetage und staatliche Gedenktage) | M |
| F-F3 | Gesetzliche Feiertage Deutschland je Bundesland (Voreinstellung Hessen) | M |
| F-F4 | Christliche Feiertage und Gedenktage, die nicht gesetzlich sind (z. B. Advent, Aschermittwoch), als eigene, abschaltbare Gruppe (vom Auftraggeber bestätigt) | S |
| F-F5 | Wöchentliche Toraabschnitte (Paraschot) mit Haftara, Diaspora/Israel | M |
| F-F6 | Rosch Chodesch, Molad, Schabbat Mewarchim, Omerzählung, besondere Schabbatot | M |
| F-F7 | Zmanim für einen gewählten Ort: Sonnenauf- und -untergang, Kerzenzündung, Schabbat-/Feiertagsende, Beginn und Ende der Fasttage; weitere Zmanim (z. B. Sof Sman Kriat Schma) wählbar | M |
| F-F7a | Kerzenzündung: Minuten vor Sonnenuntergang frei wählbar; Voreinstellung je Ort nach örtlichem Brauch (Frankfurt am Main 30 Minuten, Jerusalem 40 Minuten, sonst 18 Minuten) | M |
| F-F7b | Schabbat- und Feiertagsende (Tzeit): Auswahl mehrerer Berechnungsarten nach dem Vorbild von myzmanim.com (Sonnenwinkel und feste Minuten, siehe Spezifikation 3.5); Auswahl gilt je Projekt und kann je Ort gespeichert werden | M |
| F-F8 | Ortsdatenbank: alle deutschen Großgemeinden und große jüdische Gemeinden weltweit (Koordinaten, Höhe, Zeitzone), dazu freie Eingabe von Längen- und Breitengrad, Höhe und Zeitzone für beliebige Orte | M |
| F-F9 | Lernzyklen (Daf Jomi u. a.) | K |
| F-F10 | Mondphasen, Beginn/Ende der Sommerzeit, Jahreszeitenanfänge, Tekufot | S |
| F-F11 | Schulferien Deutschland (aus importierbaren Datensätzen) | K |
| F-F12 | Eigene Ereignisse: einmalig, jährlich wiederkehrend (gregorianisch oder hebräisch, z. B. Jahrzeit, Bar/Bat Mizwa), Geburtstage | M |
| F-F13 | Jede Feiertagsgruppe einzeln ein-/ausblendbar, eigene Farben und Symbole | M |
| F-F14 | Jeder berechnete Eintrag zeigt auf Wunsch seine Regel an („15. Nisan, Diaspora: 2 Tage") | S |

Die vollständige fachliche Liste mit Regeln steht in [03-kalender-und-feiertage.md](03-kalender-und-feiertage.md).

### 5.3 Kalenderformen (Vorlagen)

| ID | Kalenderform | Prio |
|---|---|---|
| F-V1 | Wandkalender, Monatsblatt (12 bzw. 13 Blätter + Deckblatt) | M |
| F-V2 | Jahresübersicht / Jahresplaner (ein Blatt, auch als Poster) | M |
| F-V3 | Tischkalender (Aufsteller, Monat oder Woche je Seite) | M |
| F-V4 | Taschenkalender (Monatsübersicht, Wochenübersicht, Jahresübersicht, Notizen) | M |
| F-V5 | Desktop-Hintergrund mit Monatskalender (Bildschirmauflösung frei wählbar, PNG) | S |
| F-V6 | Wochenkalender mit Parascha und Zmanim je Woche | S |
| F-V7 | Schabbat- und Feiertagszeitentafel (Liste aller Schabbatot/Feiertage eines Jahres mit Zeiten für einen Ort) | M |
| F-V8 | Familienplaner (Spalten pro Person) | K |
| F-V9 | Immerwährender Geburtstags-/Jahrzeitkalender | K |
| F-V10 | Abreißkalender (ein Tag je Blatt) | K |

Jede Kalenderform lässt sich auf jedes Papierformat aus 5.5 anwenden; die Vorlage passt Schriftgrößen und Raster an.

### 5.4 Gestaltung

| ID | Anforderung | Prio |
|---|---|---|
| F-G1 | Wahl des Hauptkalenders (Raster gregorianisch oder jüdisch) und des Nebenkalenders (Zusatzdatum in den Zellen) | M |
| F-G2 | Jahresbeginn wählbar: Januar, beliebiger Monat, Tischri, Nisan, Schuljahr | M |
| F-G3 | Wochenbeginn wählbar (Sonntag, Montag) | M |
| F-G4 | Schriftarten, Farben, Themen; mitgelieferte Schriften mit vollständigem hebräischem Zeichensatz inkl. Nikud | M |
| F-G5 | Rechts-nach-links-Satz für Hebräisch, gemischte Schriftrichtungen in einer Zelle, komplett hebräische Kalender (RTL-Raster) | M |
| F-G6 | Fotos: Der Anwender entscheidet je Projekt, ob und wo Bilder erscheinen (je Monat, Deckblatt, keines). Zuschneiden und Positionieren | M |
| F-G7 | Sprache der Kalenderinhalte unabhängig von der UI-Sprache: Deutsch, Englisch, Hebräisch; Transliteration aschkenasisch oder sefardisch wählbar | M |
| F-G8 | Ziffern wählbar: westlich oder hebräische Buchstabenzahlen (z. B. כ״ה, תשפ״ז) | M |
| F-G9 | Vorlageneditor: Elemente frei verschieben, eigene Vorlagen speichern | S |
| F-G10 | Live-Vorschau jeder Seite, Zoom, Seitenübersicht | M |

### 5.5 Papierformate und Druck

| ID | Anforderung | Prio |
|---|---|---|
| F-D1 | DIN-A-Reihe A6, A5, A4, A3, A2, A1, A0 sowie größer (2A0, 4A0), jeweils hoch und quer | M |
| F-D2 | US-Formate Letter, Legal, Tabloid/Ledger sowie ANSI C–E | M |
| F-D3 | Weitere gängige Formate (DIN B/C-Reihe, DIN lang, Quadrat) und eigene Maße in mm oder Zoll | M |
| F-D4 | Beschnitt (z. B. 3 mm), Schnittmarken, Sicherheitsabstand | S |
| F-D5 | Großformat auf kleinem Drucker: Seite auf mehrere A4/Letter-Blätter kacheln, mit Überlappung und Klebemarken | S |
| F-D6 | Seitenreihenfolge und Rückseiten für Spiral- und Heftbindung (Tischkalender, Taschenkalender) | S |
| F-D7 | PDF/X und CMYK für Druckereien | K |

### 5.6 Export und Datenhaltung

| ID | Anforderung | Prio |
|---|---|---|
| F-E1 | PDF-Export mit eingebetteten Schriften, Vektorgrafik | M |
| F-E2 | PNG/SVG-Export einzelner Seiten, Auflösung wählbar | M |
| F-E3 | iCalendar-Export (.ics) der berechneten Einträge und eigenen Ereignisse | S |
| F-E4 | Import von Ereignissen aus .ics und CSV | S |
| F-E5 | Projekt speichern/öffnen als Datei (`.ccal`, JSON-basiert, versioniert; Fotos eingebettet oder verknüpft) | M |
| F-E6 | Browser: automatisches Zwischenspeichern lokal (IndexedDB); Desktop: Autosave und zuletzt geöffnete Projekte | M |

### 5.7 Aufbau nach dem Vorbild „Luach Pirsum"

Luach Pirsum war nach Angabe des Auftraggebers ein Windows-Programm von tichnut.de (David und Ulrich Greve, vermutlich um 2001), in Java geschrieben, für jüdische Gemeinden und Privatpersonen. Online ist dazu nichts mehr auffindbar, auch auf tichnut.de wird es nicht mehr aufgeführt; Screenshots, Ausdrucke oder ein Handbuch liegen nicht vor. Die folgenden Anforderungen beruhen deshalb allein auf der Erinnerung des Auftraggebers.

Bekannter Aufbau:

- Eine grafische Oberfläche, in der man Werte berechnen lassen konnte (Datumsumrechnung, Zeiten) und einen eigenen Kalender erzeugte.
- Kalenderformen: Wandkalender, Taschenkalender und weitere.
- Ablauf: Man gibt das gregorianische oder das hebräische Jahr an und wählt die gewünschten Monate; daraus entsteht der Kalender.
- Schwäche des Vorbilds: Der Aufbau der bildlichen Kalenderseite dauerte durch die Java-Umsetzung sehr lange. Die neue App muss hier deutlich schneller sein (NF-2).

| ID | Anforderung | Prio |
|---|---|---|
| F-LP1 | Hauptfenster mit zwei Bereichen: Rechner (Datumsumrechnung, Parascha, Feiertage und Zeiten für ein Datum und einen Ort) und Kalendererstellung | M |
| F-LP2 | Kalendererstellung beginnt mit der Angabe des Jahres, wahlweise gregorianisch oder hebräisch, und der gewünschten Monate (einzelne Monate, Bereich oder ganzes Jahr) | M |
| F-LP3 | Die Kalenderformen aus 5.3 (mindestens Wand-, Taschen-, Tischkalender) sind mit wenigen Eingaben erzeugbar; alles Weitere hat Voreinstellungen | M |
| F-LP4 | Weitere Einzelheiten (Menüs, Bildschirmaufteilung, Funktionsumfang des Vorbilds) werden mit dem Auftraggeber im Oberflächen-Prototyp (Phase 0) geklärt und hier nachgetragen | M |

Taschenkalender ist wegen dieser Vorgabe vom Auftraggeber als Kalenderform des MVP zu behandeln (bisher Version 1.0, siehe 5.3).

### 5.8 Oberfläche

| ID | Anforderung | Prio |
|---|---|---|
| F-U1 | Geführter Ablauf für neue Kalender (vorläufig: Form → Format → Zeitraum → Kalender → Feiertage/Zeiten → Gestaltung → Export; wird an Luach Pirsum angepasst) | M |
| F-U2 | UI-Sprachen Deutsch, Englisch und Hebräisch; bei Hebräisch spiegelt sich die gesamte Oberfläche (RTL) | M |
| F-U3 | Tastaturbedienung und Bildschirmleser-Unterstützung (WCAG 2.1 AA für die Bedienoberfläche) | S |
| F-U4 | Heller und dunkler Modus | S |

## 6. Nicht-funktionale Anforderungen

| ID | Anforderung |
|---|---|
| NF-1 | Korrektheit: Datumsumrechnung, Feiertage, Paraschot und Zmanim werden gegen unabhängige Referenzdaten geprüft (siehe Teststrategie). Für Daten, Feiertage und Paraschot gilt 100 % Übereinstimmung im Bereich 1900–2200; Zmanim weichen höchstens 1 Minute von der Referenz ab. |
| NF-2 | Leistung: Vorschau eines Monatsblatts in unter 200 ms nach einer Änderung; PDF-Export eines 13-seitigen Wandkalenders in unter 5 s (Desktop) bzw. 10 s (Browser) auf einem Mittelklasse-Rechner. A0-Poster ohne Fotos in unter 10 s. Ein Kalender mit allen Monaten eines Jahres steht nach der Eingabe von Jahr und Monaten in unter 2 s als Vorschau bereit (das Vorbild Luach Pirsum war hier sehr langsam). |
| NF-3 | Größe: Initialer Download der Web-App unter 8 MB komprimiert (ohne optionale Schriften); Desktop-Installer unter 30 MB. |
| NF-4 | Offline: Alle Funktionen ohne Netzwerkverbindung, inkl. Ortsdatenbank; keine Telemetrie. |
| NF-5 | Datenschutz: Keine Serverkomponente; Projekte und Fotos verlassen das Gerät nicht. |
| NF-6 | Determinismus: Gleiches Projekt + gleiche Programmversion ergibt identisches PDF auf allen Plattformen. |
| NF-7 | Wartbarkeit: Feiertagsregeln liegen als versionierte Datendateien vor; Regeländerungen erfordern keine Codeänderung. |
| NF-8 | Lizenzen: Obwohl die App zunächst privat bleibt, werden nur Abhängigkeiten und Schriften mit Open-Source-tauglichen Lizenzen (MIT, Apache-2.0, OFL) verwendet, damit eine spätere Veröffentlichung ohne Umbau möglich ist. |
| NF-9 | Sicherheit: Desktop-App mit minimalen Tauri-Berechtigungen. Code-Signierung der Installer erst bei Veröffentlichung; für die private Phase genügen unsignierte Builds. |
| NF-10 | Fotos in Großformaten: Die App warnt, wenn die Auflösung eines Bildes für das gewählte Format unter 150 dpi fällt. |

## 7. Abgrenzung (nicht im Umfang)

- Islamischer Kalender und islamische Feiertage (vorerst; die Architektur hält die Erweiterung offen).
- Gesetzliche Feiertage anderer Länder als Deutschland und Israel.
- Keine Online-Kalendersynchronisation (Google, Outlook, CalDAV); nur .ics-Import/-Export.
- Keine Benutzerkonten, keine Cloud-Speicherung, kein Druckdienst.
- Keine nativen Mobil-Apps (Android/iOS). Tauri 2 kann sie später ermöglichen.
- Keine halachischen Entscheidungen: Wo Bräuche abweichen (z. B. Minuten vor Sonnenuntergang, Schabbatende, Jahrzeit im Schaltjahr), bietet die App Optionen an, entscheidet aber nicht.

## 8. Erfolgskriterien

- Der Auftraggeber erstellt einen jüdisch-gregorianischen Wandkalender mit Paraschot und Kerzenzündezeiten für Frankfurt am Main in unter 5 Minuten und exportiert ihn als PDF.
- Aufbau und Bedienung entsprechen nach Einschätzung des Auftraggebers Luach Pirsum.
- Keine bekannten Fehler bei Datum, Feiertagen, Paraschot und Zmanim zum Release 1.0.
- Die drei Desktop-Plattformen und die großen Browser-Engines bestehen dieselbe End-to-End-Testsuite.

## 9. Risiken

| Risiko | Auswirkung | Gegenmaßnahme |
|---|---|---|
| Vorbild Luach Pirsum nicht dokumentiert | Aufbau weicht von der Erwartung ab | Frühzeitig Material sammeln; klickbarer Oberflächen-Prototyp zur Abnahme vor der Umsetzung |
| Hebräischer Textsatz (Nikud, Bidi) | Falsch dargestellte Schrift im PDF | Bewährte Shaping-Bibliothek, Frühtest im Spike, visuelle Regressionstests |
| Zmanim-Berechnung je nach Brauch unterschiedlich | Nutzer halten Zeiten für falsch | Methode und Parameter sichtbar machen, Voreinstellungen je Gemeindebrauch |
| RTL-Oberfläche in drei Sprachen | Mehraufwand bei jeder UI-Komponente | RTL von Anfang an im Komponentendesign, E2E-Tests in Hebräisch |
| Unterschiede zwischen Browser- und PDF-Darstellung | Vorschau stimmt nicht mit Druck überein | Eine gemeinsame Render-Pipeline für Vorschau und Export |
| Große Formate (A0 und größer) mit Fotos | Speicher- und Zeitbedarf im Browser | Bilder beim Import herunterrechnen auf Zielauflösung, Export in Worker |
| Eine Person entwickelt allein | Lange Gesamtdauer | Strikte Priorisierung, MVP klein halten, Phasen mit nutzbarem Zwischenergebnis |

## 10. Offene Punkte

Siehe [README.md](README.md#offene-punkte).
