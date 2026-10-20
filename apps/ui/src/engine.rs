//! Schnittstelle zwischen Oberfläche und Rust-Kern sowie zur Laufzeit-
//! umgebung (Browser oder Tauri-Desktop).
//!
//! Der Kalender wird in beiden Umgebungen im Satz-Worker gesetzt (Typst,
//! siehe `worker`). Die Desktop-App ergänzt Systemschriften und einen
//! Speichern-Dialog über Tauri-Befehle.

use cc_calendar::HebrewDate;
use js_sys::{Function, Object, Promise, Reflect, Uint8Array};
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

/// `window.__TAURI__.core.invoke`, falls die App in Tauri läuft
/// (setzt `app.withGlobalTauri = true` voraus).
fn tauri_invoke() -> Option<Function> {
    let window = web_sys::window()?;
    let tauri = Reflect::get(&window, &"__TAURI__".into()).ok()?;
    // `core` ist ein ES-Modul-Namespace ohne Prototyp; `instanceof Object`
    // (dyn_ref::<Object>) schlägt dafür fehl, daher nur `is_object` prüfen.
    if !tauri.is_object() {
        return None;
    }
    let core = Reflect::get(&tauri, &"core".into()).ok()?;
    if !core.is_object() {
        return None;
    }
    Reflect::get(&core, &"invoke".into()).ok()?.dyn_into().ok()
}

pub(crate) fn js_err(v: JsValue) -> String {
    v.as_string().unwrap_or_else(|| format!("{v:?}"))
}

async fn invoke(cmd: &str, args: &impl Serialize) -> Result<JsValue, String> {
    let invoke = tauri_invoke().ok_or("nicht in der Desktop-App")?;
    let json = serde_json::to_string(args).map_err(|e| e.to_string())?;
    let args = js_sys::JSON::parse(&json).map_err(js_err)?;
    let promise: Promise = invoke
        .call2(&JsValue::NULL, &JsValue::from_str(cmd), &args)
        .map_err(js_err)?
        .dyn_into()
        .map_err(js_err)?;
    JsFuture::from(promise).await.map_err(js_err)
}

fn from_js<T: serde::de::DeserializeOwned>(v: JsValue) -> Result<T, String> {
    let json = js_sys::JSON::stringify(&v).map_err(js_err)?;
    serde_json::from_str(&String::from(json)).map_err(|e| e.to_string())
}

pub async fn convert(year: i32, month: u8, day: u8) -> Result<HebrewDate, String> {
    match host() {
        Host::Desktop => {
            #[derive(Serialize)]
            struct Args {
                year: i32,
                month: u8,
                day: u8,
            }
            from_js(invoke("convert", &Args { year, month, day }).await?)
        }
        Host::Browser => {
            cc_calendar::gregorian_to_hebrew(year, month, day).map_err(|e| e.to_string())
        }
    }
}

/// Desktop: installierte Schriften (Familie, Datei).
pub async fn system_fonts() -> Result<Vec<(String, String)>, String> {
    #[derive(Serialize)]
    struct NoArgs {}
    from_js(invoke("system_fonts", &NoArgs {}).await?)
}

/// Desktop: Schriftdatei lesen (nur Dateien aus `system_fonts`).
pub async fn load_font(path: &str) -> Result<Vec<u8>, String> {
    #[derive(Serialize)]
    struct Args<'a> {
        path: &'a str,
    }
    let v = invoke("load_font", &Args { path }).await?;
    Ok(Uint8Array::new(&v).to_vec())
}

/// PDF speichern: Desktop mit Speichern-Dialog (gibt den Pfad zurück),
/// Browser als Download (gibt `None` zurück).
pub async fn save_pdf(name: &str, bytes: &[u8]) -> Result<Option<String>, String> {
    match tauri_invoke() {
        Some(invoke) => {
            let body = Uint8Array::from(bytes);
            let headers = Object::new();
            let _ = Reflect::set(&headers, &"x-filename".into(), &JsValue::from_str(name));
            let options = Object::new();
            let _ = Reflect::set(&options, &"headers".into(), &headers);
            let promise: Promise = invoke
                .call3(
                    &JsValue::NULL,
                    &JsValue::from_str("save_pdf"),
                    &body,
                    &options,
                )
                .map_err(js_err)?
                .dyn_into()
                .map_err(js_err)?;
            let v = JsFuture::from(promise).await.map_err(js_err)?;
            Ok(v.as_string())
        }
        None => {
            download(name, bytes, "application/pdf")?;
            Ok(None)
        }
    }
}

/// Datei im Browser herunterladen.
pub fn download(name: &str, bytes: &[u8], mime: &str) -> Result<(), String> {
    let array = js_sys::Array::new();
    array.push(&Uint8Array::from(bytes).buffer());
    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type(mime);
    let blob = web_sys::Blob::new_with_buffer_source_sequence_and_options(&array, &opts)
        .map_err(js_err)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(js_err)?;
    let document = web_sys::window()
        .and_then(|w| w.document())
        .ok_or("kein Dokument")?;
    let a: web_sys::HtmlAnchorElement = document
        .create_element("a")
        .map_err(js_err)?
        .dyn_into()
        .map_err(|_| "kein Link")?;
    a.set_href(&url);
    a.set_download(name);
    a.click();
    let _ = web_sys::Url::revoke_object_url(&url);
    Ok(())
}

/// Inhalt einer vom Benutzer gewählten Datei.
pub async fn read_file(file: &web_sys::File) -> Result<Vec<u8>, String> {
    let buf = JsFuture::from(file.array_buffer()).await.map_err(js_err)?;
    Ok(Uint8Array::new(&buf).to_vec())
}

/// Lokaler Speicher (nur Komfort: letzte Einstellungen). Fehler werden
/// ignoriert, z. B. im privaten Modus.
pub fn store_get(key: &str) -> Option<String> {
    web_sys::window()?
        .local_storage()
        .ok()??
        .get_item(key)
        .ok()?
}

pub fn store_set(key: &str, value: &str) {
    if let Some(Ok(Some(s))) = web_sys::window().map(|w| w.local_storage()) {
        let _ = s.set_item(key, value);
    }
}
