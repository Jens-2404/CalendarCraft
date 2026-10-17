# CalendarCraft – Ablaufpläne

Version 0.3 (Entwurf), Stand 17.10.2026

Änderungen in 0.3: Nutzerablauf an die Erinnerung des Auftraggebers an Luach Pirsum angepasst (Jahr und Monate zuerst, Rechner und Kalendererstellung im Hauptfenster).

Die Diagramme sind in Mermaid geschrieben und werden z. B. auf GitHub, in VS Code (mit Erweiterung) oder in Obsidian direkt dargestellt.

## 1. Nutzerablauf

Das Hauptfenster hat zwei Bereiche, wie im Vorbild Luach Pirsum: den Rechner und die Kalendererstellung. Die Einzelheiten des Vorbilds werden im Oberflächen-Prototyp (Phase 0) mit dem Auftraggeber geklärt.

```mermaid
flowchart TD
    Start([App öffnen]) --> Main{Hauptfenster}
    Main -->|Bereich Rechner| Calc[Datum eingeben<br/>gregorianisch oder hebräisch<br/>+ Ort] --> CalcOut[Ergebnis: Umrechnung, Parascha,<br/>Feiertage, Zmanim des Tages]
    CalcOut --> Main
    Main -->|Bereich Kalender| K1
    Main -->|Projekt öffnen| Open[Datei wählen / Autosave laden] --> Edit

    subgraph Create[Kalendererstellung]
        K1[1. Jahr angeben<br/>gregorianisch oder hebräisch]
        K2[2. Monate wählen<br/>einzeln, Bereich oder ganzes Jahr]
        K3[3. Kalenderform<br/>Wand, Taschen, Tisch, Jahresplaner …]
        K4[4. Format<br/>A6 bis A0 und größer, Letter,<br/>hoch/quer, eigenes Maß]
        K5[5. Inhalte<br/>Hauptkalender, Feiertagsgruppen,<br/>Ort, Zmanim, Paraschot]
        K6[6. Stil<br/>Thema, Schriften, Sprache,<br/>Transliteration, Fotos ja/nein]
        K1 --> K2 --> K3 --> K4 --> K5 --> K6
    end

    K6 --> Edit[Vorschau und Feinabstimmung]
    Edit --> Ev[Eigene Ereignisse / Fotos hinzufügen]
    Ev --> Edit
    Edit --> Check{Hinweise prüfen<br/>z. B. Text abgeschnitten,<br/>Bild zu gering aufgelöst}
    Check -->|Korrigieren| Edit
    Check -->|OK| Exp[Export: PDF / PNG / SVG / ICS]
    Exp --> Save[Projekt speichern]
    Save --> End([Fertig])
```

Jeder Schritt hat sinnvolle Voreinstellungen, sodass nach Jahr und Monaten sofort ein brauchbarer Kalender in der Vorschau steht (Ziel: unter 2 Sekunden, PRD NF-2). Alle Einstellungen lassen sich später ändern. Die Reihenfolge der Schritte 3 bis 6 ist als eine Seite mit Bereichen denkbar statt als starrer Assistent; das entscheidet der Prototyp.

## 2. Ablauf: Feiertage für einen Zeitraum berechnen

```mermaid
flowchart TD
    A[Eingabe: Zeitraum als Tageszahlen von–bis,<br/>aktive Regelgruppen, Optionen] --> B[Betroffene Jahre je Kalendersystem bestimmen<br/>z. B. gregor. 2027 = jüd. 5787–5788]
    B --> C[Für jede Regel und jedes betroffene Jahr]
    C --> D{Regel gültig<br/>in diesem Jahr?}
    D -->|nein| C
    D -->|ja| E[Grunddatum berechnen<br/>Fixed / EasterOffset / NthWeekday / Custom]
    E --> F[Verschiebungsregeln anwenden<br/>z. B. Schabbat → Sonntag]
    F --> G[Mehrtägige Feste ausdehnen,<br/>Erev / Nacht-Einträge erzeugen]
    G --> H[Zmanim für betroffene Tage berechnen<br/>Ort, Zeitzone, Profil]
    H --> I{Im Zeitraum?}
    I -->|nein| C
    I -->|ja| J[Occurrence speichern]
    J --> C
    C -->|alle verarbeitet| K[Sortieren, Prioritäten,<br/>Kollisionen markieren]
    K --> L[Ergebnis an Vorlage]
```

## 3. Ablauf: Vorschau und Export

### 3.1 Browser

```mermaid
sequenceDiagram
    actor U as Nutzer
    participant UI as Leptos-UI (Hauptthread)
    participant W as Render-Worker (WASM)
    participant DB as IndexedDB

    U->>UI: Ändert Einstellung (z. B. Farbe)
    UI->>UI: Projektzustand aktualisieren
    UI-->>DB: Autosave (verzögert, 2 s)
    UI->>W: render_page(projekt_diff, seite)
    W->>W: Daten → Vorlage → Layout → SVG
    W-->>UI: SVG der Seite
    UI-->>U: Vorschau aktualisiert

    U->>UI: Export PDF
    UI->>W: export_pdf(projekt)
    loop jede Seite
        W-->>UI: Fortschritt n/m
    end
    W-->>UI: PDF als Bytes
    UI-->>U: Download startet
```

### 3.2 Desktop (Tauri)

```mermaid
sequenceDiagram
    actor U as Nutzer
    participant UI as Leptos-UI (WebView)
    participant B as Tauri-Backend (nativ, Rust-Kern)
    participant FS as Dateisystem

    U->>UI: Ändert Einstellung
    UI->>B: invoke("render_page", diff, seite)
    B-->>UI: SVG
    UI-->>U: Vorschau
    B-->>FS: Autosave im App-Datenordner

    U->>UI: Export PDF
    UI->>B: invoke("export_pdf")
    B->>U: Speichern-Dialog
    U-->>B: Zielpfad
    B->>FS: PDF schreiben
    B-->>UI: fertig, Pfad
    UI-->>U: „Exportiert" + Ordner öffnen
```

## 4. Zustände eines Projekts

```mermaid
stateDiagram-v2
    [*] --> Neu: Assistent
    [*] --> Geladen: Datei öffnen
    Geladen --> Migriert: ältere Schemaversion
    Migriert --> Bearbeitet
    Geladen --> Bearbeitet
    Neu --> Bearbeitet
    Bearbeitet --> Bearbeitet: Änderung
    Bearbeitet --> Gesichert: Autosave
    Gesichert --> Bearbeitet: Änderung
    Bearbeitet --> Gespeichert: Speichern
    Gesichert --> Gespeichert: Speichern
    Gespeichert --> Bearbeitet: Änderung
    Gespeichert --> [*]: Schließen
    Bearbeitet --> Nachfrage: Schließen
    Nachfrage --> Gespeichert: Speichern
    Nachfrage --> [*]: Verwerfen
```

Nach einem Absturz bietet die App beim Start an, den letzten Autosave wiederherzustellen.

## 5. Entwicklungsablauf

### 5.1 Arbeitsweise im Repository

```mermaid
flowchart LR
    I[Issue / Anforderung<br/>mit PRD-ID] --> B[Feature-Branch]
    B --> C[Code + Tests]
    C --> L[Lokal: cargo fmt, clippy,<br/>cargo test]
    L --> PR[Pull Request]
    PR --> CI[CI-Pipeline]
    CI -->|rot| C
    CI -->|grün| R[Review]
    R -->|Änderungen| C
    R -->|OK| M[Merge in main]
    M --> N[Nightly-Build Web + Desktop]
```

Regeln:

- Jede fachliche Änderung (Kalender, Feiertag) braucht einen Test mit Referenzwert und eine Quellenangabe im Test oder in der Regeldatei.
- `main` ist jederzeit baubar. Releases entstehen aus Tags (`v0.1.0` …).
- Commits nach Conventional Commits (`feat:`, `fix:`, `docs:` …), daraus wird das Änderungsprotokoll erzeugt.

### 5.2 CI-Pipeline

```mermaid
flowchart TD
    T[Push / PR] --> F[fmt + clippy]
    F --> U[Unit- und Property-Tests<br/>Linux, Windows, macOS]
    U --> R[Referenztests Kalender/Feiertage]
    R --> WA[WASM-Build + Größenprüfung]
    WA --> S[Snapshot-Tests Render<br/>SVG + PNG-Vergleich]
    S --> E1[E2E Browser: Playwright<br/>Chromium, Firefox, WebKit]
    S --> E2[E2E Desktop: tauri-driver<br/>Windows, Linux]
    E1 --> OK{alle grün?}
    E2 --> OK
    OK -->|ja| ART[Artefakte: Web-Bundle,<br/>Desktop-Testbuilds]
    OK -->|nein| X[PR blockiert]
```

### 5.3 Release-Ablauf

```mermaid
flowchart TD
    A[Release-Kandidat taggen<br/>v1.2.0-rc.1] --> B[Builds für alle Plattformen]
    B --> C[Signieren: Windows Authenticode,<br/>macOS Notarisierung, Linux GPG]
    C --> D[Manueller Abnahmetest<br/>Checkliste je Plattform]
    D -->|Fehler| F[Fix auf main, neuer RC]
    F --> A
    D -->|OK| E[Release-Tag v1.2.0]
    E --> G[GitHub Release:<br/>MSI/NSIS, DMG, AppImage/deb/rpm]
    E --> H[Web-App deployen<br/>statisches Hosting]
    E --> I[Updater-Manifest veröffentlichen]
```

In der privaten Phase entfallen Signierung, Updater-Manifest und öffentliches Deployment; Releases sind dann private GitHub-Releases mit unsignierten Installern. Der volle Ablauf gilt ab der Open-Source-Veröffentlichung.

### 5.4 Pflege der Feiertagsregeln

```mermaid
flowchart LR
    Q[Neue Information<br/>z. B. Gesetzesänderung,<br/>Nutzermeldung] --> V[Quelle prüfen]
    V --> RD[Regeldatei anpassen<br/>mit valid_from / valid_to]
    RD --> T[Referenztest ergänzen]
    T --> PR[Pull Request mit Quellenlink]
    PR --> Rel[Nächstes Release<br/>oder Daten-Update]
```

Regeldateien sind in die App kompiliert. Ein späterer, separater Daten-Update-Kanal (signierte Regelpakete) ist möglich, aber nicht Teil des MVP.
