// Unter Windows kein zusätzliches Konsolenfenster im Release-Build.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use cc_calendar::HebrewDate;
use tauri::ipc::{InvokeBody, Request, Response};
use tauri_plugin_dialog::DialogExt;

/// Systemschriften (Familie, Datei); `load_font` liefert nur Dateien aus
/// dieser Liste aus.
#[derive(Default)]
struct FontCatalog(Mutex<Vec<(String, String)>>);

/// Gleiche Funktion wie im Browser, hier aber nativ im Backend ausgeführt.
#[tauri::command]
fn convert(year: i32, month: u8, day: u8) -> Result<HebrewDate, String> {
    cc_calendar::gregorian_to_hebrew(year, month, day).map_err(|e| e.to_string())
}

/// Liste der installierten Schriften (läuft im Hintergrund, kann einige
/// Sekunden dauern).
#[tauri::command]
async fn system_fonts(
    catalog: tauri::State<'_, FontCatalog>,
) -> Result<Vec<(String, String)>, String> {
    let list = tauri::async_runtime::spawn_blocking(cc_render::system_font_catalog)
        .await
        .map_err(|e| e.to_string())?;
    *catalog.0.lock().map_err(|e| e.to_string())? = list.clone();
    Ok(list)
}

/// Schriftdatei als Binärdaten (nur aus der Liste von `system_fonts`).
#[tauri::command]
fn load_font(path: String, catalog: tauri::State<'_, FontCatalog>) -> Result<Response, String> {
    let allowed = catalog
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .any(|(_, p)| *p == path);
    if !allowed {
        return Err("Datei ist keine bekannte Systemschrift".into());
    }
    std::fs::read(&path)
        .map(Response::new)
        .map_err(|e| format!("{path}: {e}"))
}

/// PDF speichern: Binärdaten im Aufruf, Dateiname im Kopf `x-filename`.
/// Öffnet den Speichern-Dialog; gibt den Pfad zurück oder `None` bei Abbruch.
#[tauri::command]
async fn save_pdf(app: tauri::AppHandle, request: Request<'_>) -> Result<Option<String>, String> {
    let InvokeBody::Raw(bytes) = request.body() else {
        return Err("keine PDF-Daten erhalten".into());
    };
    let bytes = bytes.clone();
    let name = request
        .headers()
        .get("x-filename")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Kalender.pdf")
        .to_string();
    let Some(path) = app
        .dialog()
        .file()
        .set_file_name(&name)
        .add_filter("PDF", &["pdf"])
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let path = path.into_path().map_err(|e| e.to_string())?;
    std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Some(path.display().to_string()))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(FontCatalog::default())
        .invoke_handler(tauri::generate_handler![
            convert,
            system_fonts,
            load_font,
            save_pdf
        ])
        .run(tauri::generate_context!())
        .expect("Tauri-Anwendung konnte nicht starten");
}
