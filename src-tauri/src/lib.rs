mod commands;
mod models;
mod services;
mod utils;

use commands::folder::validate_folder;
use commands::i18n::get_translations;
use commands::markdown::render_markdown;
use commands::notes::{
    check_todays_note_exists, create_todays_note, delete_note_line, get_today_note_path,
    insert_note_line, list_notes, read_note_content, save_note_content, search_notes,
    update_note_line,
};
use commands::settings::{
    get_config, set_locale, set_notes_folder, set_remember_window_size, switch_notes_folder,
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

use crate::commands::notes::jump_to_section;

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

    if config.remember_window_size {
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
            get_today_note_path,
            read_note_content,
            render_markdown,
            save_note_content,
            update_note_line,
            insert_note_line,
            delete_note_line,
            jump_to_section,
            initialize_app,
            show_window,
            search_notes,
            get_config,
            get_translations,
            get_theme_colors,
            get_all_tags,
            get_tag_suggestions,
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
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
