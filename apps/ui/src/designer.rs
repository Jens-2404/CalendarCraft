//! Kalender gestalten: Einstellungen links, Vorschau rechts, PDF speichern.
//!
//! Jede Änderung setzt den Kalender nach kurzer Pause neu. Gesetzt wird im
//! Satz-Worker (Typst, siehe `worker`), damit die Oberfläche flüssig bleibt.
//! Die Einstellungen werden lokal im Browser gemerkt.

use std::collections::HashSet;
use std::time::Duration;

use cc_templates::common::GREG_MONTHS;
use cc_templates::settings::{CalendarKind, CalendarSettings, PAPERS, file_name};
use cc_worker_protocol::{Reply, Request};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::engine::{self, Host};
use crate::worker;

const STORE_KEY: &str = "calendarcraft.settings.v1";

const EVENTS_HELP: &str = "Eine Zeile je Eintrag, Felder mit Semikolon:
hebr;8 Tischrei;Jahrzeit …
jährlich;19.09.;Geburtstag …
einmal;2026-10-05;Elternabend
ferien;2026-10-05;2026-10-17;Herbstferien";

#[component]
pub fn Designer() -> impl IntoView {
    let initial = engine::store_get(STORE_KEY)
        .and_then(|s| serde_json::from_str::<CalendarSettings>(&s).ok())
        .unwrap_or_default();
    let settings = RwSignal::new(initial);

    // Schriftfamilien im Worker (mitgelieferte und hinzugefügte)
    let families = RwSignal::new(Vec::<String>::new());
    let system_catalog = RwSignal::new(Vec::<(String, String)>::new());
    let loading_fonts = StoredValue::new_local(HashSet::<String>::new());
    let fonts_version = RwSignal::new(0u32);

    let photos_count = RwSignal::new(0usize);

    let page = RwSignal::new(0usize);
    let pages = RwSignal::new(0usize);
    let svg = RwSignal::new(String::new());
    let status = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let generation = StoredValue::new(0u64);
    let render_seq = StoredValue::new(0u64);

    // Stand des Worker übernehmen (bleibt beim Reiterwechsel erhalten)
    spawn_local(async move {
        match worker::call(Request::State, Vec::new()).await {
            Ok((
                Reply::State {
                    families: f,
                    photos,
                },
                _,
            )) => {
                families.set(f);
                photos_count.set(photos);
            }
            Ok(_) => {}
            Err(e) => error.set(Some(e)),
        }
    });

    // Kalender im Worker setzen; nur die Antwort auf den letzten Auftrag zählt
    let render = move || {
        let s = settings.get_untracked();
        busy.set(true);
        render_seq.update_value(|g| *g += 1);
        let my = render_seq.get_value();
        spawn_local(async move {
            let req = Request::Render {
                settings: Box::new(s),
                page: page.get_untracked(),
            };
            let result = worker::call(req, Vec::new()).await;
            if render_seq.get_value() != my {
                return;
            }
            match result {
                Ok((
                    Reply::Page {
                        pages: n,
                        page: p,
                        svg: s,
                    },
                    _,
                )) => {
                    pages.set(n);
                    page.set(p);
                    svg.set(s);
                    error.set(None);
                }
                Ok(_) => {}
                Err(e) => error.set(Some(e)),
            }
            busy.set(false);
        });
    };

    // Bei jeder Änderung: merken und nach kurzer Pause neu setzen
    Effect::new(move |_| {
        let s = settings.get();
        fonts_version.track();
        photos_count.track();
        if let Ok(json) = serde_json::to_string(&s) {
            engine::store_set(STORE_KEY, &json);
        }
        generation.update_value(|g| *g += 1);
        let my = generation.get_value();
        set_timeout(
            move || {
                if generation.get_value() == my {
                    render();
                }
            },
            Duration::from_millis(350),
        );
    });

    let show_page = move |p: usize| {
        page.set(p);
        spawn_local(async move {
            match worker::call(Request::Page { page: p }, Vec::new()).await {
                // Inzwischen weitergeblättert: nur die aktuelle Seite zeigen
                Ok((
                    Reply::Page {
                        pages: n,
                        page: q,
                        svg: s,
                    },
                    _,
                )) if page.get_untracked() == q => {
                    pages.set(n);
                    svg.set(s);
                }
                Ok(_) => {}
                Err(e) => error.set(Some(e)),
            }
        });
    };

    // Desktop: Systemschrift bei Bedarf nachladen und an den Worker geben
    let ensure_font = move |family: String| {
        if families.with_untracked(|f| f.contains(&family))
            || loading_fonts.with_value(|l| l.contains(&family))
        {
            return;
        }
        let paths: Vec<String> = system_catalog.with_untracked(|c| {
            c.iter()
                .filter(|(f, _)| *f == family)
                .map(|(_, p)| p.clone())
                .collect()
        });
        if paths.is_empty() {
            return;
        }
        loading_fonts.update_value(|l| {
            l.insert(family.clone());
        });
        spawn_local(async move {
            status.set(format!("Lade Schrift {family} …"));
            let mut data = Vec::new();
            for p in paths {
                if let Ok(d) = engine::load_font(&p).await {
                    data.push(d);
                }
            }
            match worker::call(Request::AddFonts, data).await {
                Ok((Reply::Fonts { families: f, .. }, _)) => {
                    families.set(f);
                    status.set(String::new());
                }
                Ok(_) => status.set(String::new()),
                Err(e) => status.set(e),
            }
            fonts_version.update(|v| *v += 1);
        });
    };

    if engine::host() == Host::Desktop {
        spawn_local(async move {
            status.set("Lade Liste der Systemschriften …".into());
            match engine::system_fonts().await {
                Ok(cat) => {
                    system_catalog.set(cat);
                    status.set(String::new());
                    let s = settings.get_untracked();
                    ensure_font(s.font);
                    ensure_font(s.heading_font);
                }
                Err(e) => status.set(format!("Systemschriften nicht verfügbar: {e}")),
            }
        });
    }

    let all_families = Memo::new(move |_| {
        let mut v = families.get();
        v.extend(system_catalog.with(|c| c.iter().map(|(f, _)| f.clone()).collect::<Vec<_>>()));
        v.retain(|f| !f.contains("Math"));
        v.sort_by_key(|f| f.to_lowercase());
        v.dedup();
        v
    });

    // Eigene Schriftdatei hinzufügen (Browser und Desktop)
    let on_font_file = move |ev: leptos::ev::Event| {
        let input: web_sys::HtmlInputElement = event_target(&ev);
        let Some(list) = input.files() else { return };
        let files: Vec<web_sys::File> = (0..list.length()).filter_map(|i| list.get(i)).collect();
        spawn_local(async move {
            let mut names = Vec::new();
            let mut data = Vec::new();
            for f in files {
                match engine::read_file(&f).await {
                    Ok(d) => {
                        names.push(f.name());
                        data.push(d);
                    }
                    Err(e) => status.set(e),
                }
            }
            let added = match worker::call(Request::AddFonts, data).await {
                Ok((Reply::Fonts { families: f, added }, _)) => {
                    families.set(f);
                    added
                }
                Ok(_) => Vec::new(),
                Err(e) => {
                    status.set(e);
                    return;
                }
            };
            for (file, a) in names.iter().zip(&added) {
                if a.is_empty() {
                    status.set(format!("{file}: keine Schrift erkannt"));
                }
            }
            if let Some(n) = added.into_iter().flatten().next() {
                status.set(format!("Schrift „{n}“ hinzugefügt"));
                settings.update(|s| s.font = n);
            }
            fonts_version.update(|v| *v += 1);
        });
    };

    // Fotos wählen
    let on_photos = move |ev: leptos::ev::Event| {
        let input: web_sys::HtmlInputElement = event_target(&ev);
        let Some(list) = input.files() else { return };
        let mut files: Vec<web_sys::File> =
            (0..list.length()).filter_map(|i| list.get(i)).collect();
        files.sort_by_key(|f| f.name());
        spawn_local(async move {
            let mut names = Vec::new();
            let mut data = Vec::new();
            for (k, f) in files.iter().enumerate() {
                let name = f.name().to_lowercase();
                let ext = name.rsplit('.').next().unwrap_or("jpg").to_string();
                if !matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "gif" | "webp") {
                    continue;
                }
                if let Ok(d) = engine::read_file(f).await {
                    names.push(format!("foto-{k:03}.{ext}"));
                    data.push(d);
                }
            }
            match worker::call(Request::SetPhotos { names }, data).await {
                Ok((Reply::Photos { count }, _)) => {
                    photos_count.set(count);
                    status.set(format!("{count} Fotos geladen"));
                }
                Ok(_) => {}
                Err(e) => status.set(e),
            }
        });
    };

    // PDF aus dem zuletzt gesetzten Kalender
    let save = move |_| {
        let name = file_name(&settings.get_untracked());
        status.set("Erzeuge PDF …".into());
        spawn_local(async move {
            let bytes = match worker::call(Request::Pdf, Vec::new()).await {
                Ok((Reply::Pdf, mut data)) if !data.is_empty() => data.swap_remove(0),
                Ok(_) => {
                    status.set("Nicht gespeichert: keine PDF-Daten".into());
                    return;
                }
                Err(e) => {
                    status.set(format!("Nicht gespeichert: {e}"));
                    return;
                }
            };
            status.set("Speichere …".into());
            match engine::save_pdf(&name, &bytes).await {
                Ok(Some(path)) => status.set(format!("Gespeichert: {path}")),
                Ok(None) => status.set(format!("„{name}“ heruntergeladen")),
                Err(e) => status.set(format!("Nicht gespeichert: {e}")),
            }
        });
    };

    let kind = move || settings.with(|s| s.kind);
    let is_month = move || {
        matches!(
            kind(),
            CalendarKind::MonatKlassisch | CalendarKind::MonatFoto
        )
    };

    view! {
        <div class="designer">
            <aside class="settings">
                <fieldset>
                    <legend>"Kalender"</legend>
                    <label>
                        "Kalenderart"
                        <select on:change=move |ev| {
                            if let Some(k) = CalendarKind::from_id(&event_target_value(&ev)) {
                                settings.update(|s| {
                                    s.kind = k;
                                    s.portrait = k.default_portrait();
                                    if k == CalendarKind::Jahr && s.year >= 3000 {
                                        s.year = 2027;
                                    }
                                    if k == CalendarKind::Gemeinde && s.year < 3000 {
                                        s.year = 5787;
                                    }
                                });
                            }
                        }>
                            {CalendarKind::ALL
                                .into_iter()
                                .map(|k| {
                                    view! {
                                        <option value=k.id() selected=move || kind() == k>
                                            {k.label()}
                                        </option>
                                    }
                                })
                                .collect_view()}
                        </select>
                    </label>
                    <label>
                        "Jahr"
                        <input
                            type="number"
                            prop:value=move || settings.with(|s| s.year.to_string())
                            on:change=move |ev| {
                                if let Ok(y) = event_target_value(&ev).parse::<i32>() {
                                    settings.update(|s| s.year = y);
                                }
                            }
                        />
                        <small>
                            {move || match kind() {
                                CalendarKind::Jahr => "gregorianisch, z. B. 2027",
                                CalendarKind::Gemeinde => "jüdisch, z. B. 5787",
                                _ => "5787 = jüdisches Jahr ab Rosch HaSchana, 2027 = ab Startmonat",
                            }}
                        </small>
                    </label>
                    <Show when=move || {
                        kind() == CalendarKind::Jahr || (is_month() && settings.with(|s| s.year < 3000))
                    }>
                        <label>
                            "Startmonat"
                            <select on:change=move |ev| {
                                if let Ok(m) = event_target_value(&ev).parse::<u8>() {
                                    settings.update(|s| s.start_month = m);
                                }
                            }>
                                {(1u8..=12)
                                    .map(|m| {
                                        view! {
                                            <option
                                                value=m.to_string()
                                                selected=move || settings.with(|s| s.start_month == m)
                                            >
                                                {GREG_MONTHS[usize::from(m) - 1]}
                                            </option>
                                        }
                                    })
                                    .collect_view()}
                            </select>
                        </label>
                    </Show>
                    <Show when=is_month>
                        <label>
                            "Anzahl Monate"
                            <input
                                type="number"
                                min="1"
                                max="24"
                                prop:value=move || settings.with(|s| s.months.to_string())
                                on:change=move |ev| {
                                    if let Ok(m) = event_target_value(&ev).parse::<u8>() {
                                        settings.update(|s| s.months = m.clamp(1, 24));
                                    }
                                }
                            />
                        </label>
                    </Show>
                    <Show when=move || kind() == CalendarKind::Jahr>
                        <label>
                            "Seiten"
                            <select on:change=move |ev| {
                                if let Ok(p) = event_target_value(&ev).parse::<u8>() {
                                    settings.update(|s| s.pages = p);
                                }
                            }>
                                <option value="2" selected=move || settings.with(|s| s.pages != 1)>
                                    "2 Seiten (je 6 Monate)"
                                </option>
                                <option value="1" selected=move || settings.with(|s| s.pages == 1)>
                                    "1 Seite (alle Monate)"
                                </option>
                            </select>
                        </label>
                    </Show>
                    <Show when=move || matches!(kind(), CalendarKind::Jahr | CalendarKind::Gemeinde)>
                        <label>
                            {move || if kind() == CalendarKind::Gemeinde { "Name der Gemeinde (eine Zeile je Eintrag)" } else { "Überschrift" }}
                            <textarea
                                rows="2"
                                prop:value=move || settings.with(|s| s.title.clone())
                                on:change=move |ev| settings.update(|s| s.title = event_target_value(&ev))
                            ></textarea>
                        </label>
                    </Show>
                    <Show when=move || kind() == CalendarKind::Gemeinde>
                        <label>
                            "Name auf Hebräisch"
                            <input
                                type="text"
                                dir="rtl"
                                prop:value=move || settings.with(|s| s.name_hebrew.clone())
                                on:change=move |ev| settings.update(|s| s.name_hebrew = event_target_value(&ev))
                            />
                        </label>
                        <label>
                            "Adresse und Kontakt (Blöcke durch Leerzeile trennen)"
                            <textarea
                                rows="5"
                                prop:value=move || settings.with(|s| s.info.clone())
                                on:change=move |ev| settings.update(|s| s.info = event_target_value(&ev))
                            ></textarea>
                        </label>
                    </Show>
                </fieldset>

                <fieldset>
                    <legend>"Format und Schrift"</legend>
                    <div class="row">
                        <label>
                            "Format"
                            <select on:change=move |ev| settings.update(|s| s.paper = event_target_value(&ev))>
                                {PAPERS
                                    .iter()
                                    .map(|(id, label)| {
                                        let id = id.to_string();
                                        let id2 = id.clone();
                                        view! {
                                            <option value=id selected=move || settings.with(|s| s.paper == id2)>
                                                {*label}
                                            </option>
                                        }
                                    })
                                    .collect_view()}
                            </select>
                        </label>
                        <label>
                            "Ausrichtung"
                            <select on:change=move |ev| settings.update(|s| s.portrait = event_target_value(&ev) == "hoch")>
                                <option value="quer" selected=move || settings.with(|s| !s.portrait)>"Quer"</option>
                                <option value="hoch" selected=move || settings.with(|s| s.portrait)>"Hoch"</option>
                            </select>
                        </label>
                    </div>
                    <label>
                        "Schrift"
                        <select on:change=move |ev| {
                            let f = event_target_value(&ev);
                            ensure_font(f.clone());
                            settings.update(|s| s.font = f);
                        }>
                            {move || {
                                all_families
                                    .get()
                                    .into_iter()
                                    .map(|f| {
                                        let sel = settings.with_untracked(|s| s.font == f);
                                        view! { <option value=f.clone() selected=sel>{f.clone()}</option> }
                                    })
                                    .collect_view()
                            }}
                        </select>
                    </label>
                    <label>
                        "Schrift der Monatsköpfe"
                        <select on:change=move |ev| {
                            let f = event_target_value(&ev);
                            ensure_font(f.clone());
                            settings.update(|s| s.heading_font = f);
                        }>
                            {move || {
                                all_families
                                    .get()
                                    .into_iter()
                                    .map(|f| {
                                        let sel = settings.with_untracked(|s| s.heading_font == f);
                                        view! { <option value=f.clone() selected=sel>{f.clone()}</option> }
                                    })
                                    .collect_view()
                            }}
                        </select>
                    </label>
                    <label>
                        {move || format!("Schriftgröße: {:.0} %", settings.with(|s| s.font_scale * 100.0))}
                        <input
                            type="range"
                            min="0.6"
                            max="1.8"
                            step="0.05"
                            prop:value=move || settings.with(|s| s.font_scale.to_string())
                            on:change=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                    settings.update(|s| s.font_scale = v);
                                }
                            }
                        />
                    </label>
                    <label class="file">
                        "Eigene Schriftdatei hinzufügen (TTF, OTF)"
                        <input type="file" accept=".ttf,.otf,.ttc" multiple on:change=on_font_file />
                    </label>
                </fieldset>

                <Show when=move || kind() == CalendarKind::MonatFoto>
                    <fieldset>
                        <legend>"Fotos"</legend>
                        <label>
                            "Fotos pro Monat"
                            <select on:change=move |ev| {
                                if let Ok(n) = event_target_value(&ev).parse::<u8>() {
                                    settings.update(|s| s.photos_per_month = n);
                                }
                            }>
                                {[1u8, 2, 3, 4, 6]
                                    .into_iter()
                                    .map(|n| {
                                        view! {
                                            <option
                                                value=n.to_string()
                                                selected=move || settings.with(|s| s.photos_per_month == n)
                                            >
                                                {n.to_string()}
                                            </option>
                                        }
                                    })
                                    .collect_view()}
                            </select>
                        </label>
                        <label class="file">
                            "Fotos wählen (werden nach Dateinamen auf die Monate verteilt)"
                            <input type="file" accept="image/*" multiple on:change=on_photos />
                        </label>
                        <small>{move || format!("{} Fotos geladen", photos_count.get())}</small>
                    </fieldset>
                </Show>

                <Show when=is_month>
                    <fieldset>
                        <legend>"Eigene Einträge"</legend>
                        <textarea
                            class="events"
                            rows="7"
                            placeholder=EVENTS_HELP
                            prop:value=move || settings.with(|s| s.events_text.clone())
                            on:change=move |ev| settings.update(|s| s.events_text = event_target_value(&ev))
                        ></textarea>
                        <small class="help">{EVENTS_HELP}</small>
                    </fieldset>
                </Show>

                <div class="actions">
                    <button class="primary" on:click=save>"PDF speichern"</button>
                    <button on:click=move |_| settings.set(CalendarSettings::default())>
                        "Zurücksetzen"
                    </button>
                </div>
            </aside>

            <section class="preview">
                <div class="toolbar">
                    <button
                        disabled=move || { page.get() == 0 }
                        on:click=move |_| show_page(page.get_untracked().saturating_sub(1))
                    >
                        "‹"
                    </button>
                    <span>{move || format!("Seite {} von {}", page.get() + 1, pages.get().max(1))}</span>
                    <button
                        disabled=move || { page.get() + 1 >= pages.get() }
                        on:click=move |_| show_page(page.get_untracked() + 1)
                    >
                        "›"
                    </button>
                    <span class="status">
                        {move || if busy.get() { "Wird erzeugt …".to_string() } else { status.get() }}
                    </span>
                </div>
                <Show when=move || error.get().is_some()>
                    <p class="error">{move || error.get().unwrap_or_default()}</p>
                </Show>
                <div class="page" inner_html=move || svg.get()></div>
            </section>
        </div>
    }
}
