# CalendarCraft

Kalender-App in Rust für Browser und Desktop (Windows, macOS, Linux): jüdischer und gregorianischer Kalender, Feiertage, Paraschot, Zmanim, alle Kalenderformen und Papierformate. Planungsunterlagen in [docs/](docs/README.md).

Stand: Phase 0 (Setup und Spikes), privat.

## Aufbau

| Ordner | Inhalt |
|---|---|
| `crates/` | Rust-Kern: `cc-calendar` (Umrechnung), weitere Crates noch leer |
| `apps/ui` | Gemeinsame Leptos-Oberfläche |
| `apps/web` | Einstieg für die Web-App (Trunk) |
| `apps/desktop` | Desktop-App (Tauri 2) |
| `apps/cli` | Kommandozeile |
| `spikes/` | Wegwerf-Code für die Architekturentscheidungen |

## Voraussetzungen

- Rust (stable) mit Ziel `wasm32-unknown-unknown`: `rustup target add wasm32-unknown-unknown`
- Trunk und Tauri CLI: `cargo install --locked trunk tauri-cli`
- Linux zusätzlich: `libwebkit2gtk-4.1-dev`, `librsvg2-dev`, `libayatana-appindicator3-dev`

## Kalender gestalten (Oberfläche)

Im Reiter „Kalender gestalten“ wählt man links Kalenderart, Jahr, Format, Ausrichtung, Schrift, Schrift der Monatsköpfe, Schriftgröße, Fotos und eigene Einträge; rechts erscheint die Vorschau Seite für Seite. „PDF speichern“ lädt das PDF herunter (Browser) bzw. öffnet einen Speichern-Dialog (Desktop-App). Die Einstellungen bleiben im Browser bzw. in der App gespeichert.

Schriften: Im Browser stehen die mitgelieferten Schriften (Libertinus Serif mit Hebräisch) zur Wahl; eigene Schriftdateien (TTF/OTF) lassen sich hinzufügen. In der Desktop-App erscheinen zusätzlich alle installierten Schriften.

## Starten

Web-App (http://localhost:1420; `--release`, weil Typst im Debug-Build sehr langsam ist):

```bash
cd apps/web && trunk serve --release
```

Desktop-App (startet Trunk selbst):

```bash
cd apps/desktop && cargo tauri dev
```

Kommandozeile:

```bash
cargo run -p cc-cli -- convert 2026-09-12
```

Kalender als PDF erzeugen (Zeiten für Frankfurt am Main, Feiertage Hessen):

```bash
cargo run --release -p cc-cli -- jahreskalender --year 2027 --out ausgabe/Jahreskalender_2027.pdf
```

```bash
cargo run --release -p cc-cli -- gemeindekalender --year 5787 --out ausgabe/Kalender_5787.pdf
```

Monatskalender, ein Monat pro Seite (Stil "klassisch" quer oder "foto" hoch mit Fotofläche):

```bash
cargo run --release -p cc-cli -- monatskalender --jahr 5787 --stil klassisch --out ausgabe/Monatskalender_5787.pdf
```

```bash
cargo run --release -p cc-cli -- monatskalender --jahr 5787 --stil foto --fotos meine-fotos --fotos-pro-monat 6 --eintraege eintraege.txt --out ausgabe/Fotokalender_5787.pdf
```

`--jahr 5787` beginnt mit dem Monat von Rosch HaSchana und umfasst 13 Monate; alternativ `--von 2026-09 --monate 12`. Fotos werden nach Dateinamen sortiert der Reihe nach auf die Monate verteilt.

Eigene Einträge (`--eintraege`), eine Zeile je Eintrag:

```text
hebr;8 Tischrei;Jahrzeit Israel ben Zechariahu
jährlich;19.09.;Geburtstag Anna
einmal;2026-10-05;Elternabend
ferien;2026-10-05;2026-10-17;Herbstferien
```

Gestaltung (für alle Kalender):

| Option | Wirkung |
|---|---|
| `--format a4` | Papierformat: a0–a5, letter, legal, tabloid oder eigene Maße wie `300x200` (mm); Standard a4 quer |
| `--hoch` / `--quer` | Hoch- oder Querformat |
| `--schrift "Segoe UI"` | Schriftfamilie des Inhalts (jede installierte Schrift) |
| `--titelschrift "Times New Roman"` | Schrift der Monatsköpfe |
| `--schriftgroesse 1.2` | alle Schriften 20 % größer (0.5 bis 3.0) |
| `--png` | zusätzlich ein Bild je Seite |

Nur Jahreskalender: `--seiten 1` (alle Monate auf einer Seite) oder `2` (Standard), `--start-monat 7` (z. B. Juli bis Juni), `--titel "Text"`.

Beispiel Hochformat mit größerer Schrift:

```bash
cargo run --release -p cc-cli -- jahreskalender --year 2027 --hoch --schrift "Segoe UI" --schriftgroesse 1.15 --out ausgabe/Jahreskalender_2027_hoch.pdf
```

Tests:

```bash
cargo test --workspace
```
