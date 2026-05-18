mod commands;
mod models;
mod services;
mod utils;

use commands::folder::validate_folder;
use commands::i18n::get_translations;
use commands::notes::{
    check_todays_note_exists, create_todays_note, delete_note_line, detect_threads, ensure_thread,
    get_last_available_note_path, get_note_path_by_offset, get_today_note_path, insert_note_line,
    list_notes, read_note_content, save_note_content, update_note_line,
};
use commands::search::{
    aggregate_thread, search_notes, search_notes_by_tag, search_tags, search_threads,
};
use commands::settings::{
    get_config, set_control_center_width, set_locale, set_notes_folder, set_notes_list_layout,
    set_remember_app_layout, set_remember_settings, set_search_is_fuzzy, set_search_mode,
    set_search_selected_tag, switch_notes_folder,
};
use commands::setup::initialize_app;
use commands::tags::{add_note_tag, get_all_tags, get_tag_suggestions, remove_note_tag};
use commands::theme::{get_theme_colors, set_theme};
use models::app_state::AppState;
use models::config::AppConfig;
use models::note_session::NoteSession;
use services::note_manager::NoteManager;
use services::tag_manager::TagManager;
use std::sync::Mutex;
use utils::window::show_window;

/// The main entry point of the application's core logic.
///
/// This function:
/// 1. Loads the application's persistent configuration.
/// 2. Initializes the Tauri builder with necessary plugins.
/// 3. Registers application-wide state managers (AppState).
/// 4. Registers all the `invoke_handler` commands used by the frontend.
/// 5. Configures the initial window state.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::load();
    let note_manager = NoteManager::new(config.notes_folder.clone(), config.locale.clone());

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init());

    if config.remember_app_layout {
        builder = builder.plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::all()
                        & !tauri_plugin_window_state::StateFlags::VISIBLE,
                )
                .build(),
        );
    }

    builder
        .manage(AppState {
            note_manager: Mutex::new(note_manager),
            tag_manager: Mutex::new(TagManager::new()),
            note_session: Mutex::new(NoteSession::new()),
        })
        .invoke_handler(tauri::generate_handler![
            add_note_tag,
            remove_note_tag,
            check_todays_note_exists,
            create_todays_note,
            detect_threads,
            get_last_available_note_path,
            get_note_path_by_offset,
            get_today_note_path,
            read_note_content,
            save_note_content,
            update_note_line,
            insert_note_line,
            delete_note_line,
            ensure_thread,
            initialize_app,
            show_window,
            search_notes,
            search_threads,
            search_tags,
            search_notes_by_tag,
            aggregate_thread,
            get_config,
            get_translations,
            get_theme_colors,
            get_all_tags,
            get_tag_suggestions,
            list_notes,
            set_control_center_width,
            set_locale,
            set_notes_folder,
            set_notes_list_layout,
            set_remember_app_layout,
            set_remember_settings,
            set_search_mode,
            set_search_is_fuzzy,
            set_search_selected_tag,
            set_theme,
            switch_notes_folder,
            validate_folder
        ])
        .setup(|app| {
            let config = AppConfig::load();
            if !config.remember_app_layout {
                let _ = utils::window::setup_main_window(app.handle());
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
