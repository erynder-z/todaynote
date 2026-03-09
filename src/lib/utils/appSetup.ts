import { invoke } from "@tauri-apps/api/core";
import {
	appState,
	availableLocales,
	locale,
	settings,
	translations,
} from "$lib";
import type { LocaleInfo } from "$lib/types/locale";
import type { NoteContentResponse } from "$lib/types/notes";
import type { ThemeInfo } from "../types/settings";
import { applyThemeColors, availableThemes, currentTheme } from "./theme";

/**
 * Fetches the initial application state from the Tauri backend and synchronizes
 * all frontend stores (settings, theme, i18n, and app state).
 */
export const initializeApp = async () => {
	try {
		const initialState: {
			notes_folder: string | null;
			locale: string;
			theme: string;
			available_locales: LocaleInfo[];
			available_themes: ThemeInfo[];
			translations: Record<string, string>;
			theme_colors: Record<string, string>;
			today_note_path: string | null;
			today_note_content: NoteContentResponse | null;
		} = await invoke("initialize_app");

		translations.set(initialState.translations);
		locale.set(initialState.locale);
		availableLocales.set(initialState.available_locales);

		availableThemes.set(initialState.available_themes);
		currentTheme.set(initialState.theme);
		applyThemeColors(initialState.theme_colors);

		settings.notes_folder = initialState.notes_folder || "";
		settings.locale = initialState.locale;
		settings.theme = initialState.theme;

		if (initialState.notes_folder) {
			appState.todayNotePath = initialState.today_note_path;
			appState.todayNoteContent = initialState.today_note_content;
		}

		setTimeout(async () => {
			await invoke("show_window");
		}, 100);
	} catch (error) {
		console.error("Failed to initialize app:", error);
	}
};
