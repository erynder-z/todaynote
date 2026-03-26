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
	availableLocales.set(state.availableLocales);
};

/**
 * Synchronizes theme-related stores and applies UI colors.
 */
export const syncThemeState = (state: AppPayload) => {
	availableThemes.set(state.availableThemes);
	currentTheme.set(state.theme);
	applyThemeColors(state.themeColors);
};

/**
 * Synchronizes the global settings store.
 */
export const syncSettingsState = (state: AppPayload) => {
	settings.notesFolder = state.notesFolder || "";
	settings.locale = state.locale;
	settings.theme = state.theme;
	settings.rememberWindowSize = state.rememberWindowSize;
};

/**
 * Synchronizes the application's runtime state (active note, etc.).
 */
export const syncSessionState = (state: AppPayload) => {
	sessionState.isMac = state.isMac;
	if (state.notesFolder) {
		sessionState.todayNotePath = state.todayNotePath;
		sessionState.todayNoteContent = state.todayNoteContent;
	}
};
