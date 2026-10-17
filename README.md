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

## Starten

Web-App (http://localhost:1420):

```bash
cd apps/web && trunk serve
```

Desktop-App (startet Trunk selbst):

```bash
cd apps/desktop && cargo tauri dev
```

Kommandozeile:

```bash
cargo run -p cc-cli -- convert 2026-09-12
```

Tests:

```bash
cargo test --workspace
```
