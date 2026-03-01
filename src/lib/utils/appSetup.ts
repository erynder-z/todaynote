import { invoke } from "@tauri-apps/api/core";
import {
	appState,
	availableLocales,
	locale,
	settings,
	translations,
} from "$lib";
import type { ThemeInfo } from "../types/settings";
import type { LocaleInfo } from "./i18n";
import { applyThemeColors, availableThemes, currentTheme } from "./theme";

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
			today_note_content: string | null;
		} = await invoke("initialize_app");

		translations.set(initialState.translations);
		locale.set(initialState.locale);
		availableLocales.set(initialState.available_locales);

		availableThemes.set(initialState.available_themes);
		currentTheme.set(initialState.theme);
		applyThemeColors(initialState.theme_colors);

		settings.update((s) => ({
			...s,
			notes_folder: initialState.notes_folder || "",
			locale: initialState.locale,
			theme: initialState.theme,
		}));

		if (initialState.notes_folder) {
			appState.update((state) => ({
				...state,
				todayNotePath: initialState.today_note_path,
				todayNoteContent: initialState.today_note_content,
			}));
		}
	} catch (error) {
		console.error("Failed to initialize app:", error);
	}
};
