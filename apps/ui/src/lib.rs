//! Gemeinsame Oberfläche für Browser und Desktop.
//!
//! Reiter "Kalender gestalten" (Einstellungen, Vorschau, PDF) und
//! "Datumsrechner". Sprachumschaltung Deutsch/Englisch/Hebräisch spiegelt
//! die Oberfläche; die Gestaltungsseite ist im Prototyp nur deutsch.

mod designer;
mod engine;
mod i18n;
mod worker;

use engine::Host;
use i18n::{Lang, Text};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Designer,
    Converter,
}

#[component]
pub fn App() -> impl IntoView {
    let (lang, set_lang) = signal(Lang::De);
    let (tab, set_tab) = signal(Tab::Designer);

    // Schreibrichtung und Sprache am <html>-Element setzen, damit CSS mit
    // logischen Eigenschaften die ganze Oberfläche spiegelt.
    Effect::new(move |_| {
        let l = lang.get();
        if let Some(root) = document().document_element() {
            let _ = root.set_attribute("lang", l.code());
            let _ = root.set_attribute("dir", if l.is_rtl() { "rtl" } else { "ltr" });
        }
    });

    let t = move |key: Text| lang.get().text(key);
    let host = match engine::host() {
        Host::Browser => Text::HostBrowser,
        Host::Desktop => Text::HostDesktop,
    };

    view! {
        <header class="bar">
            <h1>"CalendarCraft"</h1>
            <nav class="tabs">
                <button class:active=move || tab.get() == Tab::Designer on:click=move |_| set_tab.set(Tab::Designer)>
                    {move || t(Text::CreateCalendar)}
                </button>
                <button class:active=move || tab.get() == Tab::Converter on:click=move |_| set_tab.set(Tab::Converter)>
                    {move || t(Text::Calculator)}
                </button>
            </nav>
            <span class="host">{move || t(host)}</span>
            <nav class="langs" aria-label=move || t(Text::Language)>
                {Lang::ALL
                    .into_iter()
                    .map(|l| {
                        view! {
                            <button class:active=move || lang.get() == l on:click=move |_| set_lang.set(l)>
                                {l.label()}
                            </button>
                        }
                    })
                    .collect_view()}
            </nav>
        </header>
        <main>
            <Show when=move || tab.get() == Tab::Designer fallback=move || view! { <Converter lang /> }>
                <designer::Designer />
            </Show>
        </main>
    }
}

#[component]
fn Converter(lang: ReadSignal<Lang>) -> impl IntoView {
    let (date, set_date) = signal(String::from("2026-09-12"));
    let (result, set_result) = signal(None::<Result<cc_calendar::HebrewDate, String>>);
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
    view! {
        <section class="pane converter">
            <h2>{move || t(Text::Calculator)}</h2>
            <label>
                {move || t(Text::GregorianDate)}
                <input
                    type="date"
                    prop:value=date
                    on:input=move |ev| {
                        set_date.set(event_target_value(&ev));
                        run();
                    }
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
    }
}
