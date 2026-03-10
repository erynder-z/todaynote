import { invoke } from "@tauri-apps/api/core";
import {
	availableLocales,
	locale,
	sessionState,
	settings,
	translations,
} from "$lib";
import type { AppPayload } from "$lib/types/appState";
import { applyThemeColors, availableThemes, currentTheme } from "./theme";

/**
 * Fetches the initial application state from the Tauri backend and synchronizes
 * all frontend stores (settings, theme, i18n, and app state).
 */
export const initializeApp = async () => {
	try {
		const initialState: AppPayload = await invoke("initialize_app");
		syncFullAppState(initialState);

		setTimeout(async () => {
			await invoke("show_window");
		}, 100);
	} catch (error) {
		console.error("Failed to initialize app:", error);
	}
};

/**
 * Synchronizes all frontend stores with the provided state.
 */
export const syncFullAppState = (state: AppPayload) => {
	syncI18nState(state);
	syncThemeState(state);
	syncSettingsState(state);
	syncSessionState(state);
};

/**
 * Synchronizes translation-related stores.
 */
export const syncI18nState = (state: AppPayload) => {
	translations.set(state.translations);
	locale.set(state.locale);
	availableLocales.set(state.available_locales);
};

/**
 * Synchronizes theme-related stores and applies UI colors.
 */
export const syncThemeState = (state: AppPayload) => {
	availableThemes.set(state.available_themes);
	currentTheme.set(state.theme);
	applyThemeColors(state.theme_colors);
};

/**
 * Synchronizes the global settings store.
 */
export const syncSettingsState = (state: AppPayload) => {
	settings.notes_folder = state.notes_folder || "";
	settings.locale = state.locale;
	settings.theme = state.theme;
	settings.remember_window_size = state.remember_window_size;
};

/**
 * Synchronizes the application's runtime state (active note, etc.).
 */
export const syncSessionState = (state: AppPayload) => {
	if (state.notes_folder) {
		sessionState.todayNotePath = state.today_note_path;
		sessionState.todayNoteContent = state.today_note_content;
	}
};
