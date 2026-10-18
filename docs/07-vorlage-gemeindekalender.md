# CalendarCraft – Vorlage „Gemeindekalender" (Wandkalender mit Monatsspalten)

Version 0.1 (Entwurf), Stand 18.10.2026

Grundlage ist ein Beispiel des Auftraggebers: der Wandkalender 5784 (2023/24) der Jüdischen Gemeinde Frankfurt, Datei `Kalender_5784.pdf`. Der Auftraggeber sagt dazu: „So hatte ich mir den Tisch- bzw. Wandkalender vorgestellt." Diese Vorlage ist damit das wichtigste Ziel der Kalenderformen im MVP. Die Beschreibung unten stammt aus der Durchsicht des PDFs; sie ist keine Vorgabe der Gemeinde.

## 1. Eckdaten des Beispiels

| Merkmal | Beispiel |
|---|---|
| Format | A4 quer (297 × 210 mm), Beschnittbox gesetzt |
| Seiten | 2 (Seite 1: 6 Monate, Seite 2: 7 Monate; das Jahr 5784 ist ein Schaltjahr mit 13 Monaten) |
| Erstellung | Adobe InDesign, Ausgabe als PDF/X-4 (Druckerei-tauglich) |
| Schriften | Favorit (lateinisch, kostenpflichtig), Favorit Hebrew, David (hebräisch), Ebrima; Symbole aus Wingdings |
| Sprache | Deutsch, Hebräisch; deutsche Umschrift der Namen nach aschkenasischer/hebräischer Mischform („Haasinu", „Chol Hamoed", „Jiskor") |
| Hauptkalender | jüdisch: eine Spalte je jüdischem Monat, eine Zeile je Tag |
| Nebenkalender | gregorianisch: Datum und Wochentag in jeder Zeile links |
| Ort der Zeiten | Frankfurt am Main |

## 2. Aufbau einer Seite

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ Logo, Name        große Jahreszahl "5784"  2023/24     בס״ד   Adresse,    │
│ der Gemeinde      (blau)                   (gold)             Kontakt     │
├────────┬────────┬────────┬────────┬────────┬────────┬────────────────────┤
│Elul –  │Cheschw.│Kislew  │Tewet   │Schwat  │Adar I  │  Monatsspalten     │
│Tischrei│Okt–Nov │Nov–Dez │Dez–Jan │Jan–Feb │Feb–Mär │  (Kopf: Monat+Jahr │
│Sep–Okt │        │        │        │        │        │   hebr., darunter  │
│        │        │        │        │        │        │   greg. Monate)    │
│ 30     │        │        │        │        │        │                    │
│ Zeilen │        │        │        │        │        │                    │
├────────┴────────┴────────┴────────┴────────┴────────┴────────────────────┤
│ Legende (Fußzeile)           Staatliche Feiertage 2023/2024: …           │
└──────────────────────────────────────────────────────────────────────────┘
```

Kopfzeile (auf beiden Seiten gleich):

- Logo und Name der Gemeinde (hebräisch und deutsch), Bild und Text frei austauschbar.
- Sehr große Jahreszahl (hebräisch gezählt, hier in lateinischen Ziffern) in Blau, daneben das gregorianische Doppeljahr in Gold.
- „בס״ד" (mit Gottes Hilfe) rechts oben, als frei wählbares Kopfzeichen.
- Adresse, Telefon, E-Mail und Webadresse der Gemeinde in zwei Textblöcken.

Das ist die Fläche, die im Vorbild „Luach Pirsum" vermutlich für Gemeinde- und Werbeinformationen vorgesehen war. In CalendarCraft ist sie ein frei gestaltbarer Kopfbereich mit Textfeldern und Bildern.

## 3. Aufbau einer Monatsspalte

Spaltenkopf: Name des jüdischen Monats hebräisch/deutsch mit Jahr („Tewet 5784"), darunter die zugehörigen gregorianischen Monate („Dezember 2023 – Januar 2024"). Der erste Kopf kombiniert zwei Monate („Elul 5783 – Tischrei 5784"), weil der Kalender am letzten Tag des Elul beginnt (Erew Rosch HaSchana).

Eine Zeile je Tag mit diesen Feldern:

| Feld | Inhalt | Beispiel |
|---|---|---|
| Links | gregorianisches Tagesdatum und Wochentag (Mo, Di …) | „16 Sa" |
| Mitte | Name des Tages: Feiertag, Parascha, Fast- oder Gedenktag, ggf. Zusatzzeile | „1. Tag Rosch Haschana", „Haasinu / Schabat Schuwa" |
| Rechts außen | jüdisches Tagesdatum als Zahl und Wochentag hebräisch (ראשון … שבת) | „8 שבת" |
| Zeit (rot) | Kerzenzünden am Vorabend; Schabbat-/Feiertagsende | „19.10", „20.27" |

Zeiten werden im Format `hh.mm` mit Punkt gesetzt. Die Linie über der Zeit bedeutet Schabbat-/Feiertagsbeginn, die Linie darunter Ende (so in der Legende erklärt).

### Farben und Hervorhebungen

| Tagestyp | Darstellung im Beispiel |
|---|---|
| Schabbat | blaue Zeile; Parascha-Name zentriert in der Zeile, bei besonderen Schabbatot mit Zusatzzeile („Schabbat Schuwa", „Schabbat Para", „Schabbat haChodesch") |
| Feiertag (Jom Tov) | fetter Name, Zeile in Blau oder hellem Grau |
| Chol HaMoed, Chanukka | graue Zeile |
| Vorabend (Erew) | helle Lavendelzeile, kleiner Text („Erew Rosch Haschana") |
| Fasttag | fetter Name mit Zeiten „FB 5.26 FE 20.13" (Fastenbeginn, Fastenende) |
| Rosch Chodesch | Punkt ● im Tageskopf; ❶ erster, ❷ zweiter Tag |
| Chanukka | Kerzensymbol mit der Uhrzeit des Kerzenzündens |
| Omerzählung | Zahl 1–49 mit Kürzel „L'O" in der Zeile |
| Jiskor | kleiner Zusatz „Jiskor" an Tagen mit Totengedenken (im Beispiel an Schemini Azeret zu sehen; welche weiteren Tage er trägt, ist noch zu prüfen) |

### Zusätzliche Zeiten und Hinweise im Beispiel

- Fastenbeginn (FB) und -ende (FE) an Fasttagen
- Mitternacht (Chazot) in der Nacht vor Rosch HaSchana („Mitternacht 01:24")
- Bdikat Chametz (Suche), Chametz Verbot (Ende der Essenszeit), Chametz verbrannt (Ende der Verbrennungszeit), Taanit Bechorot (Fasten der Erstgeborenen) um Pessach
- Hinweis, dass Pessach-Zeiten mit Sternchen unter dem Kalender erklärt werden (Fußnote)

## 4. Fußzeile

- Legende der Zeichen: „FB = Fastenbeginn, FE = Fastenende, Rosch Chodesch = ●, Rosch Chodesch 1. Tag = ❶, Rosch Chodesch 2. Tag = ❷, Linie über Uhrzeit = Schabbat-Beginn, Linie unter Uhrzeit = Ende, Kerzenzünden = (Symbol)".
- „Staatliche Feiertage 2023/2024:" mit Datumsliste der gesetzlichen Feiertage (Hessen: 3. Oktober, 25.–26. Dezember, 1. Januar usw.). Das bestätigt die Anforderung F-F3 (Voreinstellung Hessen, Feiertage je Bundesland).
- Fußnoten für Pessach-Zeiten.

## 5. Folgerungen für CalendarCraft

### 5.1 Neue Anforderungen (in PRD aufgenommen)

| ID | Anforderung | Prio |
|---|---|---|
| F-V11 | Vorlage „Gemeindekalender": jüdischer Hauptkalender mit einer Spalte je Monat und einer Zeile je Tag, gregorianisches Datum links, A4 quer bis A0, 1–3 Seiten je nach Spaltenzahl | M |
| F-V12 | Dieselbe Vorlage lässt sich als Hochformat-Variante (weniger Spalten je Seite) und für Tischkalender (ein Monat je Seite, gleiche Zeilenlogik) ableiten | S |
| F-G11 | Frei gestaltbarer Kopfbereich: Logo, großer Jahrestitel, Textblöcke (Adresse, Kontakt), Kopfzeichen „בס״ד"; Fußzeile mit Legende und Fußnoten; Texte und Bilder je Projekt, später als Gemeindeprofil speicherbar | M |
| F-G12 | Zeilenstile je Tagestyp (Schabbat, Feiertag, Chol HaMoed, Erew, Fasttag, Rosch Chodesch) mit Farbe, Schriftschnitt und Symbolen, alles einstellbar | M |
| F-G13 | Zeiten rot und mit Linie über/unter der Zeit (Beginn/Ende), Format `hh.mm` oder `hh:mm` wählbar | M |
| F-F15 | Kalenderbeginn am letzten Tag des Vormonats (Erew Rosch HaSchana) und Ende nach 12 bzw. 13 Monaten, Monatskopf mit zwei Monaten, wenn nötig | M |
| F-F16 | Zusätzliche Zeiten: Fastenbeginn/-ende, Mitternacht (Chazot), Bdikat Chametz, Chametz-Verbot, Chametz-Verbrennung, Taanit Bechorot | M |
| F-F17 | Totengedenktage „Jiskor" als kennzeichenbare Gruppe; welche Tage dazugehören, wird anhand des Beispiels und der Gemeindebräuche festgelegt | S |
| F-F18 | Gesetzliche Feiertage als Datumsliste in der Fußzeile („Staatliche Feiertage …") | M |
| F-D7 | PDF/X-4 für Druckereien (bisher Prio K) wird auf S angehoben, weil das Vorbild so ausgegeben wurde; Machbarkeit mit Typst in Phase 4 prüfen | S |

### 5.2 Zmanim

Der Auftraggeber nennt Frankfurt mit 30 Minuten Kerzenzünden. Das PDF dient als Prüfstein: Die Zeiten für Frankfurt (Kerzenzünden, Schabbatende, Fastenbeginn und -ende, Chametz-Zeiten, Mitternacht) werden mit den Berechnungen von `cc-astro` verglichen. Daraus lässt sich ablesen, welche Berechnungsarten die Gemeinde verwendet (Winkel oder Minuten für Schabbatende). Diese Zuordnung ist eine Aufgabe in Phase 3. Ergebnis wird als Voreinstellung „Frankfurt" gespeichert.

### 5.3 Technik

- Die Vorlage ist ein Typst-Dokument, das aus den berechneten Tagesdaten erzeugt wird (ADR-002). Spaltenraster und Zeilenfelder sind einfacher als der Spike-Kalender; kritisch sind der Kopfbereich (Logo, Schriften) und der Zeilenumbruch bei langen Tagesnamen.
- Das hebräische Wochentagskürzel und der Tagesname auf derselben Zeile mischen Schriftrichtungen. Der Spike hat genau diese Fälle bereits korrekt gesetzt.
- Schriften: Das Beispiel nutzt kostenpflichtige Schriften (Favorit). CalendarCraft liefert freie Schriften (OFL) mit; das Aussehen weicht dadurch leicht ab. Gemeinden mit eigener Lizenz können eigene Schriften laden (Schriften-Import je Projekt).
- Format und Seitenzahl: 13 Monate plus Kopf lassen sich auf zwei A4-Seiten verteilen, indem die Spalten je Seite gleichmäßig aufgeteilt werden. Die Vorlage berechnet die Aufteilung (6 + 7 im Beispiel). Zeilenhöhe ergibt sich aus der Seitenhöhe geteilt durch 30 plus Kopf. Bei A0 skalieren Schrift und Linienstärken mit.

### 5.4 Abnahmetest

Das Projekt „Gemeinde Frankfurt 5784" soll aus den Eingaben (Jahr 5784, Ort Frankfurt am Main, Hessen, Diaspora, Gemeindeprofil mit Texten) einen Kalender erzeugen, der dem Beispiel in diesen Punkten entspricht und im Rest in Gestaltung und Inhalt ähnlich ist:

1. Gleiche Monatsspalten, gleiche Zeilen (Datum, Wochentag, hebräisches Datum, hebräischer Wochentag).
2. Alle Feiertage, Fasttage, Schabbatot (Parascha, Sonder-Schabbatot), Rosch Chodesch, Omer und Jiskor-Tage stimmen mit dem Beispiel überein.
3. Alle Zeiten stimmen auf die Minute überein oder weichen höchstens um eine Minute ab; jede Abweichung wird einzeln erklärt (Rundung, andere Methode).
4. Staatliche Feiertage in der Fußzeile.

Die Texte des Beispiels (Daten, Namen, Zeiten) werden dazu in eine private Prüfdatei übernommen (siehe Teststrategie, Abschnitt 3b).

## 6. Offene Punkte zu dieser Vorlage

1. Sollen Nutzer die Namen der Tage frei umbenennen können (z. B. „Schabat" statt „Schabbat")? Das Beispiel nutzt eine eigene Schreibweise.
2. Soll das Gemeindeprofil (Logo, Adresse, Schreibweisen, Zmanim-Methode) als eigene Datei speicherbar sein, damit die Gemeinde jedes Jahr nur die Jahreszahl ändert? Vorschlag: ja, `.ccprofil`.
3. Darf das Beispiel (Layout, Texte) für Tests genutzt werden? Es ist das Werk der Gemeinde. Für die private Nutzung genügt das; vor der Open-Source-Veröffentlichung sollte die Gemeinde zustimmen oder die Prüfdaten werden durch Hebcal-Daten ersetzt.
