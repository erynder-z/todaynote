import { invoke } from "@tauri-apps/api/core";
import type { LocaleInfo } from "$lib/types/locale";
import type { NoteContentResponse } from "$lib/types/notes";
import type { AppSettings, ThemeInfo } from "$lib/types/settings";
import {
	availableLocales,
	locale,
	translations,
	updateTranslations,
} from "../utils/i18n";
import {
	applyThemeColors,
	availableThemes,
	currentTheme,
	updateTheme,
} from "../utils/theme";
import { appState } from "./appState.svelte";

/**
 * Manages application-wide user settings and persists them to the backend.
 */
export class SettingsStore {
	/** Path to the folder where notes are stored. */
	notes_folder = $state("");
	/** Active application language/locale. */
	locale = $state("en");
	/** Currently active UI theme. */
	theme = $state("light");
	/** Whether to persist and restore the window size across launches. */
	remember_window_size = $state(true);

	/**
	 * Loads initial configuration from the backend and initializes UI stores.
	 */
	async load() {
		try {
			const settings: AppSettings = await invoke("get_config");
			this.notes_folder = settings.notes_folder;
			this.locale = settings.locale;
			this.theme = settings.theme;
			this.remember_window_size = settings.remember_window_size;

			await updateTranslations(this.locale);
			await updateTheme(this.theme);
			return settings;
		} catch (error) {
			console.error("Error loading settings:", error);
			return {
				notes_folder: "",
				locale: "en",
				theme: "light",
				remember_window_size: true,
			};
		}
	}

	/**
	 * Updates the configuration in the backend and reflects changes in the UI.
	 */
	async save(newSettings: AppSettings) {
		try {
			if (newSettings.notes_folder !== undefined) {
				await invoke("set_notes_folder", { path: newSettings.notes_folder });
				this.notes_folder = newSettings.notes_folder;
			}
			if (newSettings.locale) {
				await invoke("set_locale", { locale: newSettings.locale });
				this.locale = newSettings.locale;
				await updateTranslations(newSettings.locale);
			}
			if (newSettings.theme) {
				await invoke("set_theme", { theme: newSettings.theme });
				this.theme = newSettings.theme;
				await updateTheme(newSettings.theme);
			}
			if (newSettings.remember_window_size !== undefined) {
				await invoke("set_remember_window_size", {
					remember: newSettings.remember_window_size,
				});
				this.remember_window_size = newSettings.remember_window_size;
			}
			return true;
		} catch (error) {
			console.error("Error saving settings:", error);
			return false;
		}
	}

	/**
	 * Switches the root notes folder and resets the application state accordingly.
	 */
	async switchNotesFolder(path: string) {
		try {
			const newState: {
				notes_folder: string | null;
				locale: string;
				theme: string;
				remember_window_size: boolean;
				available_locales: LocaleInfo[];
				available_themes: ThemeInfo[];
				translations: Record<string, string>;
				theme_colors: Record<string, string>;
				today_note_path: string | null;
				today_note_content: NoteContentResponse | null;
			} = await invoke("switch_notes_folder", { path });

			if (newState.notes_folder) {
				translations.set(newState.translations);
				availableLocales.set(newState.available_locales);
				locale.set(newState.locale);

				availableThemes.set(newState.available_themes);
				currentTheme.set(newState.theme);
				applyThemeColors(newState.theme_colors);

				this.notes_folder = newState.notes_folder;
				this.locale = newState.locale;
				this.theme = newState.theme;
				this.remember_window_size = newState.remember_window_size;
			}

			appState.todayNotePath = newState.today_note_path;
			appState.todayNoteContent = newState.today_note_content;
			appState.activePopup = null;

			return true;
		} catch (error) {
			console.error("Error switching notes folder:", error);
			return false;
		}
	}
}

export const settings = new SettingsStore();
