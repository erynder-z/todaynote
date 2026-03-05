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
use commands::settings::{
    get_config, set_locale, set_notes_folder, set_remember_window_size, switch_notes_folder,
};
use commands::setup::initialize_app;
use commands::theme::{get_theme_colors, set_theme};
use models::config::AppConfig;
use services::note_manager::NoteManager;
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub note_manager: Mutex<NoteManager>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::load();
    let note_manager = NoteManager::new(config.notes_folder.clone(), config.locale.clone());

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init());

    if config.remember_window_size {
        builder = builder.plugin(tauri_plugin_window_state::Builder::default().build());
    }

    builder
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
            get_theme_colors,
            list_notes,
            set_locale,
            set_notes_folder,
            set_remember_window_size,
            set_theme,
            switch_notes_folder,
            validate_folder
        ])
        .setup(|app| {
            let config = AppConfig::load();
            if !config.remember_window_size {
                let _ = utils::window::setup_main_window(app.handle());
            } else if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
