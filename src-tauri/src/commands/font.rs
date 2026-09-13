//! Tauri commands for font management.

use crate::models::app_state::AppState;
use tauri::State;

/// Retrieves a list of available system fonts.
/// Returns a list of font family names that can be used in CSS.
#[tauri::command]
pub async fn get_system_fonts() -> Result<Vec<String>, String> {
    // Get the complete list of installed fonts for the current platform
    get_all_system_fonts()
}

/// Platform-specific function to get all installed system fonts
fn get_all_system_fonts() -> Result<Vec<String>, String> {
    #[cfg(target_os = "macos")]
    {
        get_all_macos_fonts()
    }

    #[cfg(target_os = "windows")]
    {
        get_all_windows_fonts()
    }

    #[cfg(target_os = "linux")]
    {
        get_all_linux_fonts()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err("Font enumeration is not supported on this platform".to_string())
    }
}

#[cfg(target_os = "macos")]
fn get_all_macos_fonts() -> Result<Vec<String>, String> {
    use core_text::font_manager::copy_available_font_family_names;

    let font_names = copy_available_font_family_names();

    let mut fonts: Vec<String> = font_names
        .iter()
        .map(|font| font.to_string())
        .filter(|font| !font.is_empty())
        .collect();

    fonts.sort();
    fonts.dedup();

    if fonts.is_empty() {
        Err("CoreText returned no installed fonts".to_string())
    } else {
        Ok(fonts)
    }
}
#[cfg(target_os = "windows")]
fn get_all_windows_fonts() -> Result<Vec<String>, String> {
    unsafe { get_all_windows_fonts_impl() }
}

#[cfg(target_os = "windows")]
unsafe fn get_all_windows_fonts_impl() -> Result<Vec<String>, String> {
    use windows::Win32::Graphics::DirectWrite::{
        DWriteCreateFactory, IDWriteFactory, IDWriteFontCollection, DWRITE_FACTORY_TYPE_SHARED,
    };

    let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)
        .map_err(|e| format!("Failed to create DirectWrite factory: {e}"))?;

    let mut collection: Option<IDWriteFontCollection> = None;

    factory
        .GetSystemFontCollection(&mut collection, false)
        .map_err(|e| format!("Failed to get system font collection: {e}"))?;

    let collection =
        collection.ok_or_else(|| "DirectWrite returned no system font collection".to_string())?;

    let family_count = collection.GetFontFamilyCount();
    let mut fonts = Vec::with_capacity(family_count as usize);

    for i in 0..family_count {
        let family = collection
            .GetFontFamily(i)
            .map_err(|e| format!("Failed to get font family {i}: {e}"))?;

        let names = family
            .GetFamilyNames()
            .map_err(|e| format!("Failed to get family names for font {i}: {e}"))?;

        let mut index = 0u32;
        let mut exists = windows::core::BOOL(0);

        names
            .FindLocaleName(windows::core::w!("en-us"), &mut index, &mut exists)
            .ok();

        if !exists.as_bool() {
            index = 0;
        }

        let length = names
            .GetStringLength(index)
            .map_err(|e| format!("Failed to get font name length: {e}"))?;

        let mut buffer = vec![0u16; length as usize + 1];

        names
            .GetString(index, &mut buffer)
            .map_err(|e| format!("Failed to get font family name: {e}"))?;

        let name = String::from_utf16_lossy(&buffer[..length as usize]);

        if !name.is_empty() {
            fonts.push(name);
        }
    }

    fonts.sort_unstable();
    fonts.dedup();

    if fonts.is_empty() {
        Err("DirectWrite returned no installed fonts".into())
    } else {
        Ok(fonts)
    }
}

#[cfg(target_os = "linux")]
fn get_all_linux_fonts() -> Result<Vec<String>, String> {
    use std::process::Command;

    let output = Command::new("fc-list")
        .args(["--format=%{family}\\n"])
        .output()
        .map_err(|e| format!("Failed to execute fc-list: {e}"))?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    let mut fonts = output_str
        .lines()
        .flat_map(|line| line.split(','))
        .map(str::trim)
        .filter(|font| !font.is_empty())
        .map(String::from)
        .collect::<Vec<_>>();

    fonts.sort();
    fonts.dedup();

    if fonts.is_empty() {
        Err("fontconfig returned no installed fonts".to_string())
    } else {
        Ok(fonts)
    }
}

/// Sets the application font family.
#[tauri::command]
pub async fn set_font_family(
    font_family: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config()?;
    config.font_family = Some(font_family);
    config.save();

    Ok(())
}

/// Sets whether to use custom font or default app font.
#[tauri::command]
pub async fn set_use_custom_font(
    use_custom: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config()?;
    config.use_custom_font = use_custom;
    config.save();

    Ok(())
}

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use crate::models::config::AppConfig;

    ///
    /// Config Mutation Tests
    ///
    #[test]
    fn test_config_font_family_default() {
        let config = AppConfig::default();
        assert_eq!(
            config.font_family, None,
            "Default font family should be None"
        );
    }

    #[test]
    fn test_config_use_custom_font_default() {
        let config = AppConfig::default();
        assert_eq!(
            config.use_custom_font, false,
            "Default use_custom_font should be false"
        );
    }

    #[test]
    fn test_config_font_family_can_be_set() {
        let mut config = AppConfig::default();
        config.font_family = Some("Helvetica".to_string());
        assert_eq!(config.font_family, Some("Helvetica".to_string()));
    }

    #[test]
    fn test_config_use_custom_font_can_be_set() {
        let mut config = AppConfig::default();
        config.use_custom_font = true;
        assert_eq!(config.use_custom_font, true);
    }

    #[test]
    fn test_config_serialization_with_font_settings() {
        let mut config = AppConfig::default();
        config.font_family = Some("Fira Code".to_string());
        config.use_custom_font = true;

        let json = serde_json::to_string(&config).expect("Failed to serialize config");
        let deserialized: AppConfig =
            serde_json::from_str(&json).expect("Failed to deserialize config");

        assert_eq!(deserialized.font_family, Some("Fira Code".to_string()));
        assert_eq!(deserialized.use_custom_font, true);
    }

    #[test]
    fn test_config_serialization_without_custom_font() {
        let config = AppConfig::default();

        let json = serde_json::to_string(&config).expect("Failed to serialize config");
        let deserialized: AppConfig =
            serde_json::from_str(&json).expect("Failed to deserialize config");

        assert_eq!(deserialized.font_family, None);
        assert_eq!(deserialized.use_custom_font, false);
    }

    ///
    /// Font Name Validation Tests
    ///
    #[test]
    fn test_font_names_are_non_empty() {
        let fonts = vec![
            "Helvetica".to_string(),
            "Times New Roman".to_string(),
            "Arial".to_string(),
            "Fira Code".to_string(),
        ];

        for font in &fonts {
            assert!(!font.is_empty(), "Font name should not be empty");
        }
    }

    #[test]
    fn test_font_names_can_contain_spaces() {
        let font = "Times New Roman";
        assert!(font.contains(' '), "Font names can contain spaces");
    }

    #[test]
    fn test_font_names_can_contain_commas() {
        let font = "Fira Code, Monospace";
        assert!(font.contains(','), "Font names can contain commas");
    }

    #[test]
    fn test_font_names_can_contain_special_characters() {
        let font = "SF Pro Display-Regular";
        assert!(font.contains('-'), "Font names can contain hyphens");
    }
}
