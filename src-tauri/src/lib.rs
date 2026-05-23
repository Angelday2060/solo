mod db;
mod diary;
mod ledger;
mod notes;

use db::open_database;
use diary::{
    diary_create_entry, diary_delete_entry, diary_get_entry, diary_list_entries,
    diary_update_entry,
};
use ledger::{
    ledger_create_category, ledger_create_transaction, ledger_delete_category,
    ledger_delete_transactions, ledger_get_period_rollups, ledger_get_statistics,
    ledger_get_transaction, ledger_list_categories, ledger_list_transactions,
    ledger_update_transaction,
};
use notes::{
    note_add_attachment, note_create_note, note_create_notebook, note_create_tag,
    note_delete_attachment, note_delete_note, note_delete_notebook, note_get_note,
    note_list_notebooks, note_list_notes, note_list_tags, note_update_note,
    note_update_notebook,
};
use serde::Serialize;
use std::sync::Mutex;
use tauri::Manager;

pub struct DbState(pub Mutex<rusqlite::Connection>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DbHealth {
    pub ok: bool,
    pub sqlite_version: String,
    pub db_path: String,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn db_health(state: tauri::State<'_, DbState>) -> Result<DbHealth, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let sqlite_version: String = conn
        .query_row("SELECT sqlite_version()", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let db_path = conn
        .path()
        .map(|p| p.to_string())
        .unwrap_or_default();
    Ok(DbHealth {
        ok: true,
        sqlite_version,
        db_path,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let (conn, path) = open_database().map_err(|e| {
                eprintln!("solo: database init failed: {e}");
                std::io::Error::new(std::io::ErrorKind::Other, e)
            })?;
            eprintln!("solo: database ready at {}", path.display());
            app.manage(DbState(Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            db_health,
            diary_list_entries,
            diary_get_entry,
            diary_create_entry,
            diary_update_entry,
            diary_delete_entry,
            ledger_list_categories,
            ledger_create_category,
            ledger_delete_category,
            ledger_list_transactions,
            ledger_get_transaction,
            ledger_create_transaction,
            ledger_update_transaction,
            ledger_delete_transactions,
            ledger_get_period_rollups,
            ledger_get_statistics,
            note_list_notebooks,
            note_create_notebook,
            note_update_notebook,
            note_delete_notebook,
            note_list_notes,
            note_get_note,
            note_create_note,
            note_update_note,
            note_delete_note,
            note_list_tags,
            note_create_tag,
            note_add_attachment,
            note_delete_attachment,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
