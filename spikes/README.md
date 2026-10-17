# Spikes (Phase 0)

Wegwerf-Code, mit dem die Architekturentscheidungen ADR-001 und ADR-002 abgesichert werden. Nichts hiervon geht unverändert in die App; die Ergebnisse stehen in `docs/02-architektur.md`.

| Ordner | Frage |
|---|---|
| `render-typst` | Option A: Typst als eingebettete Engine (PDF, SVG, PNG aus einer Quelle) |
| `render-krilla` | Option B: eigenes Layout, PDF direkt mit krilla |

Beide erzeugen dasselbe Testblatt: Monatsblatt September 2026 mit hebräischem Datum je Tag, Feiertagsnamen mit Nikud, gemischt mit lateinischem Text; dazu eine Variante mit RTL-Raster und eine A0-Variante mit Foto.

Ausführen (Windows; die Spikes nutzen die mitgelieferten Windows-Schriften Times New Roman und Arial, damit nichts heruntergeladen werden muss):

```bash
cargo run --release -p spike-render-typst
cargo run --release -p spike-render-krilla
```

Die Dateien landen in `target/spikes/`.
