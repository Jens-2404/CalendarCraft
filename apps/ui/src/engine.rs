//! Schnittstelle zwischen Oberfläche und Rust-Kern.
//!
//! Im Browser läuft der Kern im selben WASM-Modul (später in einem Web
//! Worker). In der Desktop-App läuft er nativ im Tauri-Backend und wird über
//! `invoke` aufgerufen. Die Oberfläche merkt davon nichts.

use cc_calendar::HebrewDate;
use js_sys::{Function, Object, Promise, Reflect};
use serde::Serialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Host {
    Browser,
    Desktop,
}

pub fn host() -> Host {
    if tauri_invoke().is_some() {
        Host::Desktop
    } else {
        Host::Browser
    }
}

pub async fn convert(year: i32, month: u8, day: u8) -> Result<HebrewDate, String> {
    match tauri_invoke() {
        Some(invoke) => {
            #[derive(Serialize)]
            struct Args {
                year: i32,
                month: u8,
                day: u8,
            }
            let args =
                serde_json::to_string(&Args { year, month, day }).map_err(|e| e.to_string())?;
            let args = js_sys::JSON::parse(&args).map_err(js_err)?;
            let promise: Promise = invoke
                .call2(&JsValue::NULL, &JsValue::from_str("convert"), &args)
                .map_err(js_err)?
                .dyn_into()
                .map_err(js_err)?;
            let value = JsFuture::from(promise).await.map_err(js_err)?;
            let json = js_sys::JSON::stringify(&value).map_err(js_err)?;
            serde_json::from_str(&String::from(json)).map_err(|e| e.to_string())
        }
        None => cc_calendar::gregorian_to_hebrew(year, month, day).map_err(|e| e.to_string()),
    }
}

/// `window.__TAURI__.core.invoke`, falls die App in Tauri läuft
/// (setzt `app.withGlobalTauri = true` voraus).
fn tauri_invoke() -> Option<Function> {
    let window = web_sys::window()?;
    let tauri = Reflect::get(&window, &"__TAURI__".into()).ok()?;
    let core = Reflect::get(tauri.dyn_ref::<Object>()?, &"core".into()).ok()?;
    Reflect::get(core.dyn_ref::<Object>()?, &"invoke".into())
        .ok()?
        .dyn_into()
        .ok()
}

fn js_err(v: JsValue) -> String {
    v.as_string().unwrap_or_else(|| format!("{v:?}"))
}
