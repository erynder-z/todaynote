import { invoke } from "@tauri-apps/api/core";
import type { AppPayload } from "$lib/types/appState";
import type { AppSettings } from "$lib/types/settings";
import { syncFullAppState } from "$lib/utils/appSetup";
import { updateTranslations } from "../utils/i18n";
import { updateTheme } from "../utils/theme";
import { sessionState } from "./sessionState.svelte";

/**
 * Manages application-wide user settings and persists them to the backend.
 */
export class SettingsStore {
	notesFolder = $state("");
	locale = $state("en");
	theme = $state("light");
	rememberWindowSize = $state(true);

	/**
	 * Loads initial configuration from the backend and initializes UI stores.
	 */
	async load() {
		try {
			const config: AppSettings = await invoke("get_config");
			this.notesFolder = config.notesFolder;
			this.locale = config.locale;
			this.theme = config.theme;
			this.rememberWindowSize = config.rememberWindowSize;

			await updateTranslations(this.locale);
			await updateTheme(this.theme);
			return config;
		} catch (error) {
			console.error("Error loading settings:", error);
			return {
				notesFolder: "",
				locale: "en",
				theme: "light",
				rememberWindowSize: true,
			};
		}
	}

	/**
	 * Updates the configuration in the backend and reflects changes in the UI.
	 */
	async save(newSettings: AppSettings) {
		try {
			if (newSettings.notesFolder !== undefined) {
				await invoke("set_notes_folder", { path: newSettings.notesFolder });
				this.notesFolder = newSettings.notesFolder;
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
			if (newSettings.rememberWindowSize !== undefined) {
				await invoke("set_remember_window_size", {
					remember: newSettings.rememberWindowSize,
				});
				this.rememberWindowSize = newSettings.rememberWindowSize;
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
			const newState: AppPayload = await invoke("switch_notes_folder", {
				path,
			});

			if (newState.notesFolder) {
				syncFullAppState(newState);
			}

			sessionState.activePopup = null;

			return true;
		} catch (error) {
			console.error("Error switching notes folder:", error);
			return false;
		}
	}
}

export const settings = new SettingsStore();
