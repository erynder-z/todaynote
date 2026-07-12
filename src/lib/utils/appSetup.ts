import { invoke } from "@tauri-apps/api/core";
import type { AppPayload } from "$lib/interfaces/appState";
import { sessionState } from "../stores/sessionState.svelte";
import { settings } from "../stores/settings.svelte";
import { availableLocales, locale, translations } from "./i18n";
import { applyThemeColors, availableThemes, currentTheme } from "./theme";

/**
 * Manages application initialization and state synchronization.
 * Encapsulates all logic for setting up the frontend stores and UI state.
 */
export class AppInitializer {
	private initialized = false;

	/**
	 * Initializes the application by fetching state from backend and synchronizing stores.
	 */
	async initialize() {
		if (this.initialized) {
			console.warn("App already initialized");
			return;
		}

		try {
			const initialState: AppPayload = await invoke("initialize_app");
			this.syncFullAppState(initialState, true);
			this.initialized = true;

			// Small delay to ensure Svelte has finished rendering before showing the window
			setTimeout(async () => {
				await invoke("show_window");
			}, 100);
		} catch (error) {
			console.error("Failed to initialize app:", error);
			throw error; // Re-throw to allow caller to handle
		}
	}

	/**
	 * Synchronizes all frontend stores with the provided state.
	 */
	syncFullAppState(state: AppPayload, syncSession = false) {
		this.syncI18nState(state);
		this.syncThemeState(state);
		this.syncSettingsState(state);
		this.syncSessionState(state, syncSession);
	}

	/**
	 * Synchronizes translation-related stores.
	 */
	syncI18nState(state: AppPayload) {
		translations.set(state.translations);
		locale.set(state.locale);
		availableLocales.set(state.availableLocales);
	}

	/**
	 * Synchronizes theme-related stores and applies UI colors.
	 */
	syncThemeState(state: AppPayload) {
		availableThemes.set(state.availableThemes);
		currentTheme.set(state.theme);
		applyThemeColors(state.themeColors);
	}

	/**
	 * Synchronizes the global settings store.
	 */
	syncSettingsState(state: AppPayload) {
		settings.notesFolder = state.notesFolder || "";
		settings.locale = state.locale;
		settings.theme = state.theme;
		settings.rememberAppLayout = state.rememberAppLayout;
		settings.notesListLayout = state.notesListLayout;
		settings.rememberSettings = state.rememberSettings;
		settings.useDefaultThreadName = state.useDefaultThreadName;
		settings.defaultThreadName = state.defaultThreadName;
		settings.useDefaultThreadName =
			state.useDefaultThreadName !== undefined
				? state.useDefaultThreadName
				: true;
		settings.shortcuts = state.shortcuts;

		// If rememberSettings is false, use default values for component settings
		// Otherwise, use the saved values from the config
		if (state.rememberSettings) {
			settings.searchMode = state.searchMode;
			settings.searchIsFuzzy = state.searchIsFuzzy;
			settings.searchSelectedTag = state.searchSelectedTag;
			settings.sidebarOpen = state.sidebarOpen;
			settings.threadShortcutsMode = state.threadShortcutsMode || "navigation";
			settings.identiconStyle = state.identiconStyle;
			settings.dateFormatStyle = state.dateFormatStyle || "medium";
		} else {
			// Use default values for component settings
			settings.searchMode = "notes";
			settings.searchIsFuzzy = true;
			settings.searchSelectedTag = null;
			settings.sidebarOpen = true;
			settings.threadShortcutsMode = "navigation";
			settings.identiconStyle = "dotmatrix";
			settings.dateFormatStyle = "medium";
		}

		const width = state.controlCenterWidth;
		settings.controlCenterWidth = width > 100 ? width / 16 : width;
	}

	/**
	 * Synchronizes the application's runtime state (active note, etc.).
	 */
	syncSessionState(state: AppPayload, syncSession = false) {
		sessionState.isMac = state.isMac;
		if (state.notesFolder) {
			if (syncSession || !sessionState.todayNotePath) {
				sessionState.todayNotePath = state.todayNotePath;
				sessionState.todayNoteContent = state.todayNoteContent;
			}

			// Initialize sidebar state from settings if in horizontal layout
			if (typeof window !== "undefined" && window.innerWidth > 1024)
				sessionState.sidebarOpen = state.sidebarOpen;
		}

		// Initialize thread shortcuts mode from settings
		sessionState.threadShortcutsMode =
			state.threadShortcutsMode || "navigation";
	}
}

/**
 * Singleton instance for convenience, maintaining backward compatibility
 */
export const appInitializer = new AppInitializer();
