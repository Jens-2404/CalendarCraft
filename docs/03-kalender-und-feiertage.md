# CalendarCraft – Fachspezifikation Kalendersysteme und Feiertage

Version 0.3 (Entwurf), Stand 17.10.2026

Änderungen in 0.3: Kerzenzündung Frankfurt 30 Minuten; Auswahl des Schabbatendes nach myzmanim.com; erweiterte Ortsdatenbank mit freier Koordinateneingabe.

Änderungen in 0.2: Islamischer Kalender vorerst entfernt (Abschnitt 4 jetzt „Gesetzliche Feiertage Israel"); Zmanim, Paraschot und Transliteration ausführlicher, da Teil des MVP.

Dieses Dokument legt fest, was `cc-calendar`, `cc-holidays` und `cc-astro` berechnen. Alle Regeln werden vor der Implementierung gegen die in Abschnitt 7 genannten Referenzen geprüft. Wo hier Angaben zu gesetzlichen Regelungen stehen (Feiertage der Bundesländer, israelische Feiertage), gelten sie als Arbeitsstand und werden mit den aktuellen Gesetzestexten abgeglichen.

## 1. Grundsätze

- Jedes Datum wird intern als fortlaufende Tageszahl gespeichert. Kalendersysteme sind Sichten darauf.
- Jüdische Kalendertage beginnen mit Sonnenuntergang. In einem gregorianischen Raster erscheint ein Feiertag auf dem Tag, an dem sein Tageslicht liegt. Der Vorabend (Erev) wird auf Wunsch am Vortag angezeigt, z. B. „Erev Jom Kippur".
- Regeln haben einen Gültigkeitszeitraum (`valid_from`, `valid_to`), weil Feiertage eingeführt, abgeschafft oder verlegt werden.
- Jede berechnete Angabe kann ihre Herkunft nennen (Regel-ID und Klartext der Regel).

## 2. Gregorianischer Kalender

### 2.1 Grundregeln

- Schaltjahr: durch 4 teilbar, außer durch 100 teilbar, außer durch 400 teilbar.
- Gültig ab 15.10.1582; davor proleptisch oder (für historische Ansichten) julianisch.
- ISO-8601-Kalenderwoche: Woche mit dem ersten Donnerstag des Jahres ist KW 1, Wochenbeginn Montag.

### 2.2 Ostertermin

- Westlich: Gregorianische Osterrechnung (Algorithmus nach Gauß in der korrigierten Form bzw. nach Meeus/Butcher).
- Orthodox: Julianische Osterrechnung, das Ergebnis wird ins gregorianische Datum umgerechnet (1900–2099: +13 Tage).

Prüfwerte: Ostern 2026 westlich 5. April, orthodox 12. April; Ostern 2027 westlich 28. März, orthodox 2. Mai.

### 2.3 Bewegliche christliche Feste (Abstand zu Ostersonntag)

| Fest | Tage |
|---|---|
| Weiberfastnacht | −52 |
| Rosenmontag | −48 |
| Fastnachtsdienstag | −47 |
| Aschermittwoch | −46 |
| Palmsonntag | −7 |
| Gründonnerstag | −3 |
| Karfreitag | −2 |
| Karsamstag | −1 |
| Ostersonntag / Ostermontag | 0 / +1 |
| Weißer Sonntag | +7 |
| Christi Himmelfahrt | +39 |
| Pfingstsonntag / Pfingstmontag | +49 / +50 |
| Dreifaltigkeitssonntag | +56 |
| Fronleichnam | +60 |

Weitere Regeln: 1. Advent ist der Sonntag zwischen 27. November und 3. Dezember. Totensonntag ist der Sonntag davor, Buß- und Bettag der Mittwoch vor dem Totensonntag (fällt immer zwischen 16. und 22. November). Erntedank ist in Deutschland meist der erste Sonntag im Oktober. Muttertag ist der zweite Sonntag im Mai.

Feste Termine (Auswahl): Neujahr, Heilige Drei Könige (6.1.), Mariä Lichtmess (2.2.), Mariä Himmelfahrt (15.8.), Reformationstag (31.10.), Allerheiligen (1.11.), Allerseelen (2.11.), Martinstag (11.11.), Nikolaus (6.12.), Heiligabend, 1./2. Weihnachtstag, Silvester.

### 2.4 Gesetzliche Feiertage Deutschland (Arbeitsstand)

| Feiertag | Datum/Regel | Länder |
|---|---|---|
| Neujahr | 1.1. | alle |
| Heilige Drei Könige | 6.1. | BW, BY, ST |
| Internationaler Frauentag | 8.3. | BE (seit 2019), MV (seit 2023) |
| Karfreitag | Ostern −2 | alle |
| Ostermontag | Ostern +1 | alle |
| Tag der Arbeit | 1.5. | alle |
| Christi Himmelfahrt | Ostern +39 | alle |
| Pfingstmontag | Ostern +50 | alle |
| Fronleichnam | Ostern +60 | BW, BY, HE, NW, RP, SL; in Teilen von SN und TH |
| Augsburger Hohes Friedensfest | 8.8. | nur Stadt Augsburg |
| Mariä Himmelfahrt | 15.8. | SL; BY in Gemeinden mit überwiegend katholischer Bevölkerung |
| Weltkindertag | 20.9. | TH (seit 2019) |
| Tag der Deutschen Einheit | 3.10. | alle |
| Reformationstag | 31.10. | BB, HB, HH, MV, NI, SN, ST, SH, TH |
| Allerheiligen | 1.11. | BW, BY, NW, RP, SL |
| Buß- und Bettag | Mittwoch vor Totensonntag | SN |
| 1. und 2. Weihnachtstag | 25./26.12. | alle |

Einmalige Feiertage (z. B. der 8. Mai 2025 in Berlin) werden als Regel mit `valid_from = valid_to` abgebildet. Ostersonntag und Pfingstsonntag sind in einzelnen Ländern ausdrücklich gesetzliche Feiertage; sie werden mit den Landesgesetzen abgeglichen und als Flag geführt.

Für Österreich und die Schweiz (Prio S) gilt dieselbe Struktur; die Schweiz wird nach Kantonen geführt.

### 2.5 Weitere Angaben

- Sommerzeit (EU): Beginn letzter Sonntag im März, Ende letzter Sonntag im Oktober.
- Jahreszeitenanfänge (Tagundnachtgleichen, Sonnenwenden) und Mondphasen werden astronomisch berechnet (`cc-astro`) und auf die gewählte Zeitzone bezogen.

## 3. Jüdischer Kalender

### 3.1 Aufbau

Monate (biblische Zählung ab Nisan, das Jahr wechselt am 1. Tischri):

| Nr. | Monat | Tage |
|---|---|---|
| 7 | Tischri | 30 |
| 8 | Cheschwan (Marcheschwan) | 29 oder 30 |
| 9 | Kislew | 29 oder 30 |
| 10 | Tewet | 29 |
| 11 | Schwat | 30 |
| 12 | Adar (im Schaltjahr Adar I mit 30 Tagen) | 29 |
| 13 | Adar II (nur im Schaltjahr) | 29 |
| 1 | Nisan | 30 |
| 2 | Ijar | 29 |
| 3 | Siwan | 30 |
| 4 | Tammus | 29 |
| 5 | Aw | 30 |
| 6 | Elul | 29 |

Schaltjahre: Jahre 3, 6, 8, 11, 14, 17 und 19 im 19-jährigen Zyklus, gleichbedeutend mit `(7 · Jahr + 1) mod 19 < 7`.

Jahreslängen: 353, 354 oder 355 Tage (Gemeinjahr), 383, 384 oder 385 Tage (Schaltjahr). Die Länge entsteht aus Cheschwan und Kislew (mangelhaft 29/29, regelmäßig 29/30, vollständig 30/30).

### 3.2 Bestimmung von Rosch HaSchana

1. Molad Tischri berechnen: Epoche (Molad Tohu) plus Anzahl der seither vergangenen Monate mal 29 Tage 12 Stunden 793 Chalakim (1 Stunde = 1080 Chalakim).
2. Verschiebungsregeln (Dechijot):
   - Lo ADU Rosch: Rosch HaSchana fällt nie auf Sonntag, Mittwoch oder Freitag; dann +1 Tag.
   - Molad Saken: Liegt der Molad um oder nach 18 Uhr (gerechnet ab 18 Uhr des Vorabends, also mittags), wird um einen Tag verschoben (danach erneut Lo ADU prüfen).
   - GaTaRaD: Im Gemeinjahr, Molad am Dienstag ab 9 Std. 204 Chal. → Donnerstag.
   - BeTUTaKPaT: Im Jahr nach einem Schaltjahr, Molad am Montag ab 15 Std. 589 Chal. → Dienstag.

Die Implementierung folgt *Calendrical Calculations* und wird gegen ICU4X und Hebcal-Daten geprüft.

### 3.3 Feiertage, Fast- und Gedenktage

„D" = Diaspora, „I" = Israel. Jom Tov = Tag mit Arbeitsverbot.

| Ereignis | Datum | Besonderheiten |
|---|---|---|
| Rosch HaSchana | 1.–2. Tischri | 2 Tage auch in Israel; Vorabend 29. Elul |
| Zom Gedalja | 3. Tischri | fällt der Tag auf Schabbat → 4. Tischri |
| Jom Kippur | 10. Tischri | Fasttag, Vorabend 9. Tischri |
| Sukkot | 15.–21. Tischri | Jom Tov: 15. (I) bzw. 15.–16. (D); danach Chol HaMoed |
| Hoschana Rabba | 21. Tischri | |
| Schemini Azeret | 22. Tischri | I: zugleich Simchat Tora |
| Simchat Tora | 23. Tischri (D), 22. Tischri (I) | |
| Chanukka | 25. Kislew, 8 Tage | endet am 2. oder 3. Tewet, abhängig von der Länge des Kislew |
| Assara beTewet | 10. Tewet | Fasttag, wird nicht verschoben (kann auf Freitag fallen) |
| Tu biSchwat | 15. Schwat | |
| Purim Katan | 14. Adar I | nur im Schaltjahr |
| Ta'anit Esther | 13. Adar (Adar II) | fällt der Tag auf Schabbat → Donnerstag, 11. Adar |
| Purim | 14. Adar (Adar II) | |
| Schuschan Purim | 15. Adar (Adar II) | |
| Pessach | 15.–21. Nisan (I), 15.–22. Nisan (D) | Jom Tov: 15. und 21. (I); 15., 16., 21., 22. (D); Seder am Vorabend |
| Omerzählung | 16. Nisan bis 5. Siwan | Tag 1–49, Anzeige optional |
| Jom HaSchoa | 27. Nisan | Freitag → Donnerstag (26.); Sonntag → Montag (28.) |
| Jom HaSikaron | 4. Ijar | wird zusammen mit Jom HaAtzma'ut verschoben |
| Jom HaAtzma'ut | 5. Ijar | 5. Ijar am Freitag → Donnerstag 4. Ijar; am Schabbat → Donnerstag 3. Ijar; am Montag → Dienstag 6. Ijar (seit 2004). Jom HaSikaron jeweils am Vortag |
| Lag baOmer | 18. Ijar | |
| Jom Jeruschalajim | 28. Ijar | seit 1968 |
| Schawuot | 6. Siwan (I), 6.–7. Siwan (D) | |
| Schiw'a Assar beTammus | 17. Tammus | Schabbat → Sonntag (18.) |
| Tischa beAw | 9. Aw | Schabbat → Sonntag (10.) |
| Tu beAw | 15. Aw | |
| Rosch Chodesch | 1. des Monats; hat der Vormonat 30 Tage, auch dessen 30. Tag | kein Rosch Chodesch für Tischri (ist Rosch HaSchana) |

Weitere Angaben (alle Prio M):

- Besondere Schabbatot: Schabbat Schuwa, Schekalim, Sachor, Para, HaChodesch, HaGadol, Chason, Nachamu, Schira.
- Schabbat Mewarchim mit Ankündigung des Molad (Wochentag, Stunde, Minuten, Chalakim).
- Omerzählung als Zahl und auf Wunsch in Wochen und Tagen.
- Halbfeiertage wie Chol HaMoed, Erev Pessach (Ta'anit Bechorot, 14. Nisan; fällt er auf Schabbat → Donnerstag, 12. Nisan).

### 3.4 Wochenabschnitte (Paraschot)

Die 54 Abschnitte werden auf die Schabbatot verteilt, wobei bestimmte Paare je nach Jahrestyp zusammengelegt werden. Es gibt 14 Jahrestypen (Kewiot), bestimmt durch Wochentag von Rosch HaSchana, Jahreslänge und Wochentag von Pessach. In Jahren, in denen der 8. Tag Pessach (D) oder der 2. Tag Schawuot (D) auf Schabbat fällt, weichen Diaspora und Israel zeitweise voneinander ab. An Schabbatot, die auf einen Feiertag fallen, wird statt der Parascha die Feiertagslesung angegeben. Umsetzung als Tabelle je Jahrestyp, geprüft gegen Hebcal.

Zusätzlich je Schabbat die Haftara, aschkenasisch oder sefardisch (die Bräuche unterscheiden sich bei einigen Abschnitten), einschließlich der Sonder-Haftarot an besonderen Schabbatot und an Rosch Chodesch.

### 3.5 Zmanim (benötigt Ort und Zeitzone)

Grundlage ist die Berechnung von Sonnenauf- und -untergang sowie Sonnentiefständen nach den Algorithmen von Meeus (`cc-astro`). Alle Parameter sind einstellbar; die App bietet Voreinstellungen („Profile") für gängige Bräuche an, z. B. „Standard Diaspora", „Jerusalem".

| Zeit | Berechnung (Voreinstellung) | Optionen |
|---|---|---|
| Alot HaSchachar | Sonne 16,1° unter dem Horizont | 72 Min. vor Sonnenaufgang |
| Mischejakir | Sonne 11,5° unter dem Horizont | 10,2°, 11° |
| Sonnenaufgang (Netz) | Oberrand der Sonne am Horizont, mit Refraktion | auf Meereshöhe oder mit Ortshöhe |
| Sof Sman Kriat Schma | 3 Saisonstunden nach Sonnenaufgang (GRA) | nach Magen Awraham |
| Sof Sman Tefila | 4 Saisonstunden (GRA) | nach Magen Awraham |
| Chazot | Mitte zwischen Sonnenauf- und -untergang | |
| Mincha Gedola / Ketana | 6,5 bzw. 9,5 Saisonstunden | |
| Plag HaMincha | 10,75 Saisonstunden | |
| Kerzenzündung | Sonnenuntergang minus 18 Min.; Ortsvoreinstellung: Frankfurt am Main 30 Min., Jerusalem 40 Min. | frei wählbar (Minuten), je Ort speicherbar |
| Sonnenuntergang (Schkia) | wie Sonnenaufgang | |
| Tzet HaKochawim / Schabbat- und Feiertagsende | Sonne 8,5° unter dem Horizont | siehe Auswahl unten |
| Fastbeginn (kleine Fasttage) | Alot HaSchachar | |
| Fastende | Tzet HaKochawim | eigener Winkel möglich |

Auswahl für Schabbat- und Feiertagsende: Der Auftraggeber möchte die Wahl wie auf myzmanim.com. Die Seite war bei der Recherche nicht vollständig abrufbar; ihre genaue Optionsliste ist deshalb noch nicht abgeglichen. Nach den gesichteten Quellen (u. a. dem dort veröffentlichten Aufsatz „When is Shabbos Over?") sind mindestens diese Arten verbreitet und werden angeboten:

| Art | Definition |
|---|---|
| Sonnenwinkel 7,083° | Sonne 7°5′ unter dem Horizont |
| Sonnenwinkel 8,5° | Sonne 8,5° unter dem Horizont (Voreinstellung) |
| Feste Minuten nach Sonnenuntergang | frei wählbar; gängig 42 und 50 Minuten |
| Rabbenu Tam | 72 Minuten nach Sonnenuntergang |
| Eigener Winkel / eigene Minuten | frei eingebbar |

Die Liste wird in Phase 0 anhand der Webseite von myzmanim.com Punkt für Punkt abgeglichen und hier vervollständigt (Namen, Winkel, Minuten, Verhalten bei hohen Breiten). Auch die Zmanim-Werte der Tests (Abschnitt Teststrategie 3a) nutzen MyZmanim als zusätzliche Referenz.

Sonderfälle:

- Hohe Breiten: Wenn die Sonne im Sommer nicht weit genug sinkt (z. B. 16,1° in Nordeuropa im Juni), gibt die App „keine Zeit" mit Hinweis aus oder nutzt eine wählbare Ersatzregel.
- Kerzenzündung am zweiten Abend eines Feiertags und an Schabbat-Ausgang vor einem Feiertag: frühestens nach Tzet HaKochawim; die App gibt dann diese Zeit an.
- Zeitzone und Sommerzeit kommen aus der IANA-Zeitzonendatenbank, die mitgeliefert wird.
- Ortsdatenbank: Städte mit Koordinaten, Höhe und Zeitzone (Quelle z. B. GeoNames, Lizenz CC BY 4.0, prüfen). Enthalten sind alle deutschen Großgemeinden, in der ersten Fassung mindestens Berlin, Frankfurt am Main, München, Hamburg, Köln, Düsseldorf, Stuttgart, Hannover, Leipzig, Dresden, Nürnberg, Bremen, Mannheim, Essen, Dortmund, Karlsruhe, Freiburg, Augsburg, Würzburg, Mainz, Darmstadt, Kassel, Offenbach (die endgültige Liste legt der Auftraggeber fest), außerdem große jüdische Gemeinden weltweit (u. a. Jerusalem, Tel Aviv, New York, London, Paris, Zürich, Wien, Moskau, Buenos Aires).
- Freie Eingabe von Längengrad und Breitengrad (Dezimalgrad), Höhe über dem Meer und Zeitzone für Orte, die nicht in der Liste stehen.
- Je Ort speicherbare Einstellungen: Minuten Kerzenzündung, Art des Schabbatendes.

### 3.6 Darstellung

- Jahreszahl hebräisch: z. B. 5787 = ה׳תשפ״ז (Tausender meist weggelassen: תשפ״ז).
- Hebräische Zahlbuchstaben mit Geresch/Gerschajim; 15 = ט״ו und 16 = ט״ז (nicht י״ה / י״ו).
- Namen in Hebräisch (mit oder ohne Nikud), auf Deutsch und Englisch in Transliteration. Transliteration aschkenasisch (z. B. „Schowuos", „Sukkos") oder sefardisch/modern-israelisch („Schawuot", „Sukkot"), wählbar je Projekt.
- Deutsche Umschrift folgt deutschen Lautregeln („Schawuot", „Chanukka"), englische der englischen („Shavuot", „Chanukah").

## 4. Gesetzliche Feiertage Israel

Arbeitsstand, abzugleichen mit den israelischen Gesetzen und den Veröffentlichungen der Regierung. Alle Termine folgen dem jüdischen Kalender (Israel-Regeln, also z. B. ein Tag Schawuot).

### 4.1 Gesetzliche Ruhetage

Rosch HaSchana (2 Tage), Jom Kippur, erster Tag Sukkot, Schemini Azeret/Simchat Tora, erster und siebter Tag Pessach, Jom HaAtzma'ut, Schawuot. Die Vorabende sind verkürzte Arbeitstage; die App kann sie als „Erev" kennzeichnen.

### 4.2 Staatliche Gedenk- und Feiertage ohne Arbeitsruhe

| Tag | Datum | Verschiebung |
|---|---|---|
| Jom HaSchoa | 27. Nisan | siehe 3.3 |
| Jom HaSikaron | 4. Ijar | siehe 3.3 |
| Jom Jeruschalajim | 28. Ijar | |
| Jom HaAlija | 10. Nisan (in Schulen 7. Cheschwan) | seit 2016 |
| Herzl-Tag | 10. Ijar | |
| Jabotinsky-Tag | 29. Tammus | |
| Ben-Gurion-Tag | 6. Kislew | |
| Gedenktag für Jitzchak Rabin | 12. Cheschwan | fällt er auf Freitag → Donnerstag |
| Sigd | 29. Cheschwan | seit 2008 |

Die Verschiebungsregeln dieser Tage (wenn sie auf Freitag oder Schabbat fallen) werden vor der Umsetzung einzeln anhand der jeweiligen Gesetze geprüft und mit `valid_from` versehen.

## 5. Kombinierte Darstellung

Ein Projekt hat einen Hauptkalender, der das Raster bestimmt (Monatsblätter, Wochen), und einen Nebenkalender, dessen Datum in jeder Zelle klein erscheint.

| Hauptkalender | Folge für das Raster |
|---|---|
| Gregorianisch | Monatsblatt = gregorianischer Monat; hebräische Monatswechsel werden in der Zelle markiert (z. B. „1 Kislew"); im Kopf stehen die beiden betroffenen hebräischen Monate |
| Jüdisch | Monatsblatt = jüdischer Monat; im Schaltjahr 13 Blätter; Jahr beginnt wahlweise mit Tischri oder Nisan; Raster wahlweise von rechts nach links |

Zmanim und Parascha erscheinen je nach Vorlage in der Schabbat-Zelle, in einer eigenen Spalte oder als Tabelle am Seitenrand.

Kollisionen (mehrere Feiertage an einem Tag) werden nach Priorität sortiert und bei Platzmangel gekürzt; die Vorlage legt fest, ob gekürzt, verkleinert oder in eine Fußnote ausgelagert wird.

## 6. Eigene Ereignisse

- Einmalig, jährlich, monatlich, wöchentlich.
- Jährlich nach einem beliebigen Kalendersystem, z. B. Geburtstag nach hebräischem Datum oder Jahrzeit.
- Regeln für Daten, die es nicht jedes Jahr gibt:
  - 30. Cheschwan/Kislew in einem Jahr ohne diesen Tag: Verhalten wählbar (auf den 1. des Folgemonats oder den 29.). Für Jahrzeit gelten eigene Bräuche; die App bietet beide Varianten an und entscheidet nicht.
  - Adar im Schaltjahr: Ereignisse aus einem Gemeinjahr-Adar wahlweise in Adar I oder Adar II.
  - 29. Februar: wahlweise 28.2. oder 1.3.
- Bar-/Bat-Mizwa-Rechner: aus dem hebräischen Geburtsdatum (mit Schalter „nach Sonnenuntergang geboren") das Datum des 13. bzw. 12. Geburtstags und den Schabbat mit Parascha.

## 7. Referenzen für die Verifikation

- Edward M. Reingold, Nachum Dershowitz: *Calendrical Calculations* (Cambridge University Press, Ultimate Edition 2018) – Algorithmen für alle Kalender, auch als Grundlage der Prüfimplementierung.
- Jean Meeus: *Astronomical Algorithms* (Willmann-Bell, 2. Aufl. 1998) – Sonne, Mond, Ostertermin.
- ICU4X `icu_calendar` – zweite unabhängige Umrechnung.
- Hebcal (hebcal.com, Open Source) – Referenz für jüdische Feiertage, Paraschot und Zmanim.
- KosherJava Zmanim (Open-Source-Bibliothek) – zweite Referenz für Zmanim und Parametrisierung.
- Feiertagsgesetze der Bundesländer – Referenz für gesetzliche Feiertage in Deutschland.
- Israelische Gesetze und Regierungsveröffentlichungen – Referenz für gesetzliche Feiertage und Gedenktage in Israel.

Die konkreten Referenzdatensätze (Zeiträume, Versionen, Lizenzen) werden in `testdata/README.md` dokumentiert, bevor sie im Projekt genutzt werden.
