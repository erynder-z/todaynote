import { invoke } from "@tauri-apps/api/core";
import type { AppPayload } from "$lib/interfaces/appState";
import type { NoteContentResponse } from "$lib/interfaces/notes";
import type { AppSettings } from "$lib/interfaces/settings";
import { syncFullAppState } from "$lib/utils/appSetup";
import { readNoteContent } from "$lib/utils/notes";
import { updateTranslations } from "../utils/i18n";
import { updateTheme } from "../utils/theme";
import { sessionState } from "./sessionState.svelte";

/**
 * Manages application-wide user settings and persists them to the backend.
 */
export class SettingsStore {
	notesFolder = $state("");
	locale = $state("en");
	theme = $state("blind-spot");
	rememberWindowSize = $state(true);
	notesListLayout = $state<"list" | "masonry">("list");
	rememberSettings = $state(true);
	searchMode = $state<"notes" | "threads" | "tags">("notes");
	searchIsFuzzy = $state(true);
	searchSelectedTag = $state<string | null>(null);

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
			this.notesListLayout = config.notesListLayout;
			this.rememberSettings = config.rememberSettings;
			this.searchMode = config.searchMode;
			this.searchIsFuzzy = config.searchIsFuzzy;
			this.searchSelectedTag = config.searchSelectedTag;

			await updateTranslations(this.locale);
			await updateTheme(this.theme);
			return config;
		} catch (error) {
			console.error("Error loading settings:", error);
			return {
				notesFolder: "",
				locale: "en",
				theme: "blind-spot",
				rememberWindowSize: true,
				notesListLayout: "list",
				rememberSettings: true,
				searchMode: "notes",
				searchIsFuzzy: true,
				searchSelectedTag: null,
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

				if (sessionState.todayNotePath) {
					const updatedContent = await readNoteContent(
						sessionState.todayNotePath,
					);
					if (updatedContent) this.updateNoteContent(updatedContent);
				}
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

			// Transient settings handling
			if (newSettings.notesListLayout) {
				await this.setNotesListLayout(newSettings.notesListLayout);
			}

			if (newSettings.rememberSettings !== undefined) {
				await invoke("set_remember_settings", {
					remember: newSettings.rememberSettings,
				});
				this.rememberSettings = newSettings.rememberSettings;

				// If turning OFF, reset extra settings to defaults immediately in backend/store
				if (!newSettings.rememberSettings) {
					await this.resetToDefaults();
				}
			}

			if (newSettings.searchMode) {
				await this.setSearchMode(newSettings.searchMode);
			}

			if (newSettings.searchIsFuzzy !== undefined) {
				await this.setSearchIsFuzzy(newSettings.searchIsFuzzy);
			}

			if (newSettings.searchSelectedTag !== undefined) {
				await this.setSearchSelectedTag(newSettings.searchSelectedTag);
			}

			return true;
		} catch (error) {
			console.error("Error saving settings:", error);
			return false;
		}
	}

	/**
	 * Granular setter for layout that handles conditional persistence.
	 */
	async setNotesListLayout(layout: "list" | "masonry") {
		this.notesListLayout = layout;
		if (this.rememberSettings) {
			await invoke("set_notes_list_layout", { layout });
		}
	}

	/**
	 * Granular setter for search mode that handles conditional persistence.
	 */
	async setSearchMode(mode: "notes" | "threads" | "tags") {
		this.searchMode = mode;
		if (this.rememberSettings) {
			await invoke("set_search_mode", { mode });
		}
	}

	/**
	 * Granular setter for fuzzy search that handles conditional persistence.
	 */
	async setSearchIsFuzzy(isFuzzy: boolean) {
		this.searchIsFuzzy = isFuzzy;
		if (this.rememberSettings) {
			await invoke("set_search_is_fuzzy", { is_fuzzy: isFuzzy });
		}
	}

	/**
	 * Granular setter for selected tag that handles conditional persistence.
	 */
	async setSearchSelectedTag(tag: string | null) {
		this.searchSelectedTag = tag;
		if (this.rememberSettings) {
			await invoke("set_search_selected_tag", { tag });
		}
	}

	/**
	 * Resets remembered settings to their default values.
	 */
	async resetToDefaults() {
		this.notesListLayout = "list";
		this.searchMode = "notes";
		this.searchIsFuzzy = true;
		this.searchSelectedTag = null;

		// We reset backend values to defaults regardless of rememberSettings,
		// because this is called when disabling the feature.
		await invoke("set_notes_list_layout", { layout: "list" });
		await invoke("set_search_mode", { mode: "notes" });
		await invoke("set_search_is_fuzzy", { is_fuzzy: true });
		await invoke("set_search_selected_tag", { tag: null });
	}

	/**
	 * Updates the content of the currently active note in the UI.
	 */
	updateNoteContent(updatedContent: NoteContentResponse) {
		sessionState.todayNoteContent = updatedContent;
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
