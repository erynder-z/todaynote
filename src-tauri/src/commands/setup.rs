use crate::commands::settings::get_translations;
use crate::models::config::AppConfig;
use crate::models::response_types::InitialAppState;
use std::fs;
use std::path::PathBuf;

#[tauri::command]
pub async fn initialize_app() -> Result<InitialAppState, String> {
    let config = AppConfig::load();
    Ok(get_initial_state(config))
}

pub fn get_initial_state(config: AppConfig) -> InitialAppState {
    let notes_folder =
        if config.notes_folder.exists() || fs::create_dir_all(&config.notes_folder).is_ok() {
            Some(config.notes_folder.to_string_lossy().into_owned())
        } else {
            None
        };

    let mut state = InitialAppState {
        notes_folder,
        locale: config.locale.clone(),
        translations: get_translations(config.locale),
        today_note_path: None,
        today_note_content: None,
    };

    if let Some(folder) = &state.notes_folder {
        let current_date = crate::utils::date::get_current_date();
        let file_path = PathBuf::from(folder).join(format!("{}.md", current_date));

        let path_str = file_path.to_string_lossy().into_owned();
        state.today_note_path = Some(path_str.clone());

        if !file_path.exists() {
            let note_content = format!("# Note: {}", current_date);
            let _ = fs::write(&file_path, note_content);
        }

        if let Ok(content) = fs::read_to_string(&file_path) {
            state.today_note_content = Some(content);
        }
    }

    state
}
