//! Gemeinsame Oberfläche für Browser und Desktop.
//!
//! Stand Phase 0 (Spike ADR-001): Sprachumschaltung Deutsch/Englisch/Hebräisch
//! mit gespiegelter Oberfläche und ein Datumsumrechner, der den Rust-Kern je
//! nach Laufzeitumgebung direkt oder über Tauri aufruft.

mod engine;
mod i18n;

use engine::Host;
use i18n::{Lang, Text};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn App() -> impl IntoView {
    let (lang, set_lang) = signal(Lang::De);
    let (date, set_date) = signal(String::from("2026-09-12"));
    let (result, set_result) = signal(None::<Result<cc_calendar::HebrewDate, String>>);

    // Schreibrichtung und Sprache am <html>-Element setzen, damit CSS mit
    // logischen Eigenschaften die ganze Oberfläche spiegelt.
    Effect::new(move |_| {
        let l = lang.get();
        if let Some(root) = document().document_element() {
            let _ = root.set_attribute("lang", l.code());
            let _ = root.set_attribute("dir", if l.is_rtl() { "rtl" } else { "ltr" });
        }
    });

    let run = move || {
        let input = date.get();
        let parts: Vec<&str> = input.split('-').collect();
        let parsed = match parts.as_slice() {
            [y, m, d] => y
                .parse::<i32>()
                .ok()
                .zip(m.parse::<u8>().ok())
                .zip(d.parse::<u8>().ok()),
            _ => None,
        };
        let Some(((y, m), d)) = parsed else {
            set_result.set(Some(Err(input)));
            return;
        };
        spawn_local(async move {
            set_result.set(Some(engine::convert(y, m, d).await));
        });
    };
    run();

    let t = move |key: Text| lang.get().text(key);
    let host = match engine::host() {
        Host::Browser => Text::HostBrowser,
        Host::Desktop => Text::HostDesktop,
    };

    view! {
        <header class="bar">
            <h1>"CalendarCraft"</h1>
            <span class="host">{move || t(host)}</span>
            <nav class="langs" aria-label=move || t(Text::Language)>
                {Lang::ALL.into_iter().map(|l| view! {
                    <button
                        class:active=move || lang.get() == l
                        on:click=move |_| set_lang.set(l)
                    >{l.label()}</button>
                }).collect_view()}
            </nav>
        </header>
        <main class="panes">
            <section class="pane">
                <h2>{move || t(Text::Calculator)}</h2>
                <label>
                    {move || t(Text::GregorianDate)}
                    <input
                        type="date"
                        prop:value=date
                        on:input=move |ev| { set_date.set(event_target_value(&ev)); run(); }
                    />
                </label>
                <p class="result" aria-live="polite">
                    {move || match result.get() {
                        None => String::new(),
                        Some(Ok(h)) => {
                            let l = lang.get();
                            format!("{} {} {}", h.day, i18n::hebrew_month(&h.month_code, h.leap_year, l), h.year)
                        }
                        Some(Err(e)) => format!("{}: {e}", t(Text::Error)),
                    }}
                </p>
            </section>
            <section class="pane">
                <h2>{move || t(Text::CreateCalendar)}</h2>
                <p class="muted">{move || t(Text::ComingSoon)}</p>
            </section>
        </main>
    }
}
