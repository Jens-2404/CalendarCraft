// Unter Windows kein zusätzliches Konsolenfenster im Release-Build.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cc_calendar::HebrewDate;

/// Gleiche Funktion wie im Browser, hier aber nativ im Backend ausgeführt.
#[tauri::command]
fn convert(year: i32, month: u8, day: u8) -> Result<HebrewDate, String> {
    cc_calendar::gregorian_to_hebrew(year, month, day).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![convert])
        .run(tauri::generate_context!())
        .expect("Tauri-Anwendung konnte nicht starten");
}
