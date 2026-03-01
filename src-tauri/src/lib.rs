mod commands;
mod models;
mod services;
mod utils;

use commands::folder::validate_folder;
use commands::i18n::get_translations;
use commands::notes::{
    check_todays_note_exists, create_todays_note, get_today_note_path, list_notes,
    read_note_content, search_notes,
};
use commands::settings::{get_config, set_locale, set_notes_folder, switch_notes_folder};
use commands::setup::initialize_app;
use models::config::AppConfig;
use services::note_manager::NoteManager;
use std::sync::Mutex;

pub struct AppState {
    pub note_manager: Mutex<NoteManager>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::load();
    let note_manager = NoteManager::new(config.notes_folder.clone(), config.locale.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            note_manager: Mutex::new(note_manager),
        })
        .invoke_handler(tauri::generate_handler![
            check_todays_note_exists,
            create_todays_note,
            get_today_note_path,
            read_note_content,
            initialize_app,
            search_notes,
            get_config,
            get_translations,
            list_notes,
            set_locale,
            set_notes_folder,
            switch_notes_folder,
            validate_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
