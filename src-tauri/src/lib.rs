mod commands;
mod models;
mod utils;

use commands::folder::validate_folder;
use commands::i18n::get_translations;
use commands::notes::{
    check_todays_note_exists, create_todays_note, get_today_note_path, list_notes,
    read_note_content, search_notes,
};
use commands::settings::{get_config, set_locale, set_notes_folder, switch_notes_folder};
use commands::setup::initialize_app;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
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
