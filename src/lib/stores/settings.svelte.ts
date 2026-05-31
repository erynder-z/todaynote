import { invoke } from "@tauri-apps/api/core";
import type { AppPayload } from "$lib/interfaces/appState";
import type { ShortcutConfig } from "$lib/interfaces/input";
import type { NoteContentResponse } from "$lib/interfaces/notes";
import type { AppSettings } from "$lib/interfaces/settings";
import type { ShortcutAction } from "$lib/types/input";
import { syncFullAppState } from "$lib/utils/appSetup";
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
	sidebarOpen = $state(true);
	controlCenterWidth = $state(22);
	defaultThreadName = $state<string | null>(null);
	shortcuts = $state<Partial<Record<ShortcutAction, ShortcutConfig>>>({});

	/**
	 * Private promise chain to ensure saves happen sequentially.
	 */
	#saveQueue = Promise.resolve(true);

	/**
	 * Returns a plain object representing the current settings state.
	 */
	serialize(): AppSettings {
		return {
			notesFolder: this.notesFolder,
			locale: this.locale,
			theme: this.theme,
			rememberAppLayout: this.rememberAppLayout,
			notesListLayout: this.notesListLayout,
			rememberSettings: this.rememberSettings,
			searchMode: this.searchMode,
			searchIsFuzzy: this.searchIsFuzzy,
			searchSelectedTag: this.searchSelectedTag,
			sidebarOpen: this.sidebarOpen,
			controlCenterWidth: this.controlCenterWidth,
			defaultThreadName: this.defaultThreadName,
			shortcuts: this.shortcuts,
		};
	}

	/**
	 * Updates the configuration in the backend and reflects changes in the UI.
	 * All saves are queued and processed sequentially.
	 * Updates local state optimistically.
	 */
	async save(updates: Partial<AppSettings>): Promise<boolean> {
		Object.assign(this, updates);

		this.#saveQueue = this.#saveQueue.then(async () => {
			try {
				const updatedState: AppPayload = await invoke("update_config", {
					newConfig: this.serialize(),
				});

				syncFullAppState(updatedState);

				// If turning OFF "Remember Settings", reset those specific fields to defaults
				if (updates.rememberSettings === false) await this.resetToDefaults();

				return true;
			} catch (error) {
				console.error("Error saving settings:", error);
				return false;
			}
		});

		return this.#saveQueue;
	}

	/**
	 * Granular setter for sidebar visibility that handles conditional persistence.
	 */
	async setSidebarOpen(open: boolean): Promise<boolean> {
		if (this.rememberSettings) return await this.save({ sidebarOpen: open });

		this.sidebarOpen = open;
		return true;
	}

	/**
	 * Granular setter for control center width that handles conditional persistence.
	 */
	async setControlCenterWidth(width: number): Promise<boolean> {
		if (this.rememberAppLayout)
			return await this.save({ controlCenterWidth: width });

		this.controlCenterWidth = width;
		return true;
	}

	/**
	 * Granular setter for layout that handles conditional persistence.
	 */
	async setNotesListLayout(layout: "list" | "masonry"): Promise<boolean> {
		if (this.rememberSettings)
			return await this.save({ notesListLayout: layout });

		this.notesListLayout = layout;
		return true;
	}

	/**
	 * Granular setter for search mode that handles conditional persistence.
	 */
	async setSearchMode(mode: "notes" | "threads" | "tags"): Promise<boolean> {
		if (this.rememberSettings) return await this.save({ searchMode: mode });

		this.searchMode = mode;
		return true;
	}

	/**
	 * Granular setter for fuzzy search that handles conditional persistence.
	 */
	async setSearchIsFuzzy(isFuzzy: boolean): Promise<boolean> {
		if (this.rememberSettings)
			return await this.save({ searchIsFuzzy: isFuzzy });

		this.searchIsFuzzy = isFuzzy;
		return true;
	}

	/**
	 * Granular setter for selected tag that handles conditional persistence.
	 */
	async setSearchSelectedTag(tag: string | null): Promise<boolean> {
		if (this.rememberSettings)
			return await this.save({ searchSelectedTag: tag });

		this.searchSelectedTag = tag;
		return true;
	}

	/**
	 * Resets remembered settings to their default values.
	 */
	async resetToDefaults() {
		try {
			await invoke("reset_config_to_defaults");

			const initialState: AppPayload = await invoke("initialize_app");
			syncFullAppState(initialState);

			return true;
		} catch (error) {
			console.error("Error resetting settings to defaults:", error);
			return false;
		}
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

			if (newState.notesFolder) syncFullAppState(newState, true);

			sessionState.activePopup = null;

			return true;
		} catch (error) {
			console.error("Error switching notes folder:", error);
			return false;
		}
	}
}

export const settings = new SettingsStore();
