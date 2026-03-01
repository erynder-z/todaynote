use crate::commands::i18n::get_translations;
use crate::models::config::AppConfig;
use crate::models::response_types::{InitialAppState, LocaleInfo};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn initialize_app(state: State<'_, AppState>) -> Result<InitialAppState, String> {
    let config = AppConfig::load();
    Ok(get_initial_state(config, state))
}

pub fn get_initial_state(config: AppConfig, state: State<'_, AppState>) -> InitialAppState {
    let note_manager = state.note_manager.lock().unwrap();

    let notes_folder =
        if config.notes_folder.exists() || std::fs::create_dir_all(&config.notes_folder).is_ok() {
            Some(config.notes_folder.to_string_lossy().into_owned())
        } else {
            None
        };

    let translations = get_translations(config.locale.clone());

    let mut response = InitialAppState {
        notes_folder,
        locale: config.locale.clone(),
        available_locales: vec![
            LocaleInfo {
                id: "en".into(),
                name: "English".into(),
            },
            LocaleInfo {
                id: "de".into(),
                name: "Deutsch".into(),
            },
            LocaleInfo {
                id: "jp".into(),
                name: "日本語".into(),
            },
        ],
        translations,
        today_note_path: None,
        today_note_content: None,
    };

    if response.notes_folder.is_some() {
        let file_path = note_manager.get_today_note_path();
        let path_str = file_path.to_string_lossy().into_owned();
        response.today_note_path = Some(path_str);

        let note_header = response
            .translations
            .get("note.header")
            .map(|s| s.as_str())
            .unwrap_or("Note");

        if let Ok(created_path) = note_manager.create_todays_note(note_header) {
            if let Ok(content) = note_manager.read_note_content(&created_path) {
                response.today_note_content = Some(content);
            }
        }
    }

    response
}
