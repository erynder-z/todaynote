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
	rememberAppLayout = $state(true);
	notesListLayout = $state<"list" | "masonry">("list");
	rememberSettings = $state(true);
	searchMode = $state<"notes" | "threads" | "tags">("notes");
	searchIsFuzzy = $state(true);
	searchSelectedTag = $state<string | null>(null);
	controlCenterWidth = $state(22);
	defaultThreadName = $state<string | null>(null);

	/**
	 * Private promise chain to ensure saves happen sequentially.
	 */
	#saveQueue = Promise.resolve(true);

	/**
	 * Loads initial configuration from the backend and initializes UI stores.
	 */
	async load() {
		try {
			const config: AppSettings = await invoke("get_config");
			this.notesFolder = config.notesFolder;
			this.locale = config.locale;
			this.theme = config.theme;
			this.rememberAppLayout = config.rememberAppLayout;
			this.notesListLayout = config.notesListLayout;
			this.rememberSettings = config.rememberSettings;
			this.searchMode = config.searchMode;
			this.searchIsFuzzy = config.searchIsFuzzy;
			this.searchSelectedTag = config.searchSelectedTag;
			this.defaultThreadName = config.defaultThreadName;

			// Migration check for old pixel values
			const width = config.controlCenterWidth;
			this.controlCenterWidth = width > 100 ? width / 16 : width;

			await updateTranslations(this.locale);
			await updateTheme(this.theme);
			return config;
		} catch (error) {
			console.error("Error loading settings:", error);
			return {
				notesFolder: "",
				locale: "en",
				theme: "blind-spot",
				rememberAppLayout: true,
				notesListLayout: "list",
				rememberSettings: true,
				searchMode: "notes",
				searchIsFuzzy: true,
				searchSelectedTag: null,
				controlCenterWidth: 22,
				defaultThreadName: null,
			};
		}
	}

	/**
	 * Updates the configuration in the backend and reflects changes in the UI.
	 * All saves are queued and processed sequentially.
	 */
	async save(updates: Partial<AppSettings>) {
		this.#saveQueue = this.#saveQueue.then(async () => {
			try {
				// Determine the next state by merging updates with current state
				const next: AppSettings = {
					notesFolder: updates.notesFolder ?? this.notesFolder,
					locale: updates.locale ?? this.locale,
					theme: updates.theme ?? this.theme,
					rememberAppLayout:
						updates.rememberAppLayout ?? this.rememberAppLayout,
					notesListLayout: updates.notesListLayout ?? this.notesListLayout,
					rememberSettings: updates.rememberSettings ?? this.rememberSettings,
					searchMode: updates.searchMode ?? this.searchMode,
					searchIsFuzzy: updates.searchIsFuzzy ?? this.searchIsFuzzy,
					searchSelectedTag:
						updates.searchSelectedTag !== undefined
							? updates.searchSelectedTag
							: this.searchSelectedTag,
					controlCenterWidth:
						updates.controlCenterWidth ?? this.controlCenterWidth,
					defaultThreadName:
						updates.defaultThreadName !== undefined
							? updates.defaultThreadName
							: this.defaultThreadName,
				};

				await invoke("update_config", { newConfig: next });

				// Handle UI-specific side effects
				if (updates.locale && updates.locale !== this.locale) {
					await updateTranslations(updates.locale);

					if (sessionState.todayNotePath) {
						const updatedContent = await readNoteContent(
							sessionState.todayNotePath,
						);
						if (updatedContent) this.updateNoteContent(updatedContent);
					}
				}

				if (updates.theme && updates.theme !== this.theme) {
					await updateTheme(updates.theme);
				}

				// Update local state variables
				this.notesFolder = next.notesFolder;
				this.locale = next.locale;
				this.theme = next.theme;
				this.rememberAppLayout = next.rememberAppLayout;
				this.notesListLayout = next.notesListLayout;
				this.rememberSettings = next.rememberSettings;
				this.searchMode = next.searchMode;
				this.searchIsFuzzy = next.searchIsFuzzy;
				this.searchSelectedTag = next.searchSelectedTag;
				this.controlCenterWidth = next.controlCenterWidth;
				this.defaultThreadName = next.defaultThreadName;

				// If turning OFF "Remember Settings", reset those specific fields to defaults
				if (updates.rememberSettings === false) {
					await this.resetToDefaults();
				}

				return true;
			} catch (error) {
				console.error("Error saving settings:", error);
				return false;
			}
		});

		return this.#saveQueue;
	}

	/**
	 * Granular setter for control center width that handles conditional persistence.
	 */
	async setControlCenterWidth(width: number) {
		if (this.rememberAppLayout) {
			await this.save({ controlCenterWidth: width });
		} else {
			this.controlCenterWidth = width;
		}
	}

	/**
	 * Granular setter for layout that handles conditional persistence.
	 */
	async setNotesListLayout(layout: "list" | "masonry") {
		if (this.rememberSettings) {
			await this.save({ notesListLayout: layout });
		} else {
			this.notesListLayout = layout;
		}
	}

	/**
	 * Granular setter for search mode that handles conditional persistence.
	 */
	async setSearchMode(mode: "notes" | "threads" | "tags") {
		if (this.rememberSettings) {
			await this.save({ searchMode: mode });
		} else {
			this.searchMode = mode;
		}
	}

	/**
	 * Granular setter for fuzzy search that handles conditional persistence.
	 */
	async setSearchIsFuzzy(isFuzzy: boolean) {
		if (this.rememberSettings) {
			await this.save({ searchIsFuzzy: isFuzzy });
		} else {
			this.searchIsFuzzy = isFuzzy;
		}
	}

	/**
	 * Granular setter for selected tag that handles conditional persistence.
	 */
	async setSearchSelectedTag(tag: string | null) {
		if (this.rememberSettings) {
			await this.save({ searchSelectedTag: tag });
		} else {
			this.searchSelectedTag = tag;
		}
	}

	/**
	 * Resets remembered settings to their default values.
	 */
	async resetToDefaults() {
		await this.save({
			notesListLayout: "list",
			searchMode: "notes",
			searchIsFuzzy: true,
			searchSelectedTag: null,
		});
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
