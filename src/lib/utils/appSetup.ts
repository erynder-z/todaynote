import { invoke } from "@tauri-apps/api/core";
import type { AppPayload } from "$lib/interfaces/appState";
import { sessionState } from "../stores/sessionState.svelte";
import { settings } from "../stores/settings.svelte";
import { availableLocales, locale, translations } from "./i18n";
import { applyThemeColors, availableThemes, currentTheme } from "./theme";

/**
 * Fetches the initial application state from the Tauri backend and synchronizes
 * all frontend stores (settings, theme, i18n, and app state).
 */
export const initializeApp = async () => {
	try {
		const initialState: AppPayload = await invoke("initialize_app");
		syncFullAppState(initialState);

		// Small delay to ensure Svelte has finished rendering before showing the window
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
	settings.rememberAppLayout = state.rememberAppLayout;
	settings.notesListLayout = state.notesListLayout;
	settings.rememberSettings = state.rememberSettings;
	settings.searchMode = state.searchMode;
	settings.searchIsFuzzy = state.searchIsFuzzy;
	settings.searchSelectedTag = state.searchSelectedTag;
	settings.defaultThreadName = state.defaultThreadName;
	settings.shortcuts = state.shortcuts;

	const width = state.controlCenterWidth;
	settings.controlCenterWidth = width > 100 ? width / 16 : width;
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
