import { invoke } from "@tauri-apps/api/core";
import { writable } from "svelte/store";
import type { AppSettings, ThemeInfo } from "$lib/types/settings";
import {
	availableLocales,
	type LocaleInfo,
	locale,
	translations,
	updateTranslations,
} from "../utils/i18n";
import {
	applyThemeColors,
	availableThemes,
	currentTheme,
	updateTheme,
} from "../utils/theme";
import { appState } from "./appState";

const createSettingsStore = () => {
	const { subscribe, set, update } = writable<AppSettings>({
		notes_folder: "",
		locale: "en",
		theme: "light",
	});

	return {
		subscribe,
		set,
		update,

		async load() {
			try {
				const settings: AppSettings = await invoke("get_config");
				set(settings);
				await updateTranslations(settings.locale);
				await updateTheme(settings.theme);
				return settings;
			} catch (error) {
				console.error("Error loading settings:", error);
				return { notes_folder: "", locale: "en", theme: "light" };
			}
		},

		async save(newSettings: AppSettings) {
			try {
				if (newSettings.notes_folder) {
					await invoke("set_notes_folder", { path: newSettings.notes_folder });
				}
				if (newSettings.locale) {
					await invoke("set_locale", { locale: newSettings.locale });
					await updateTranslations(newSettings.locale);
				}
				if (newSettings.theme) {
					await invoke("set_theme", { theme: newSettings.theme });
					await updateTheme(newSettings.theme);
				}
				set(newSettings);
				return true;
			} catch (error) {
				console.error("Error saving settings:", error);
				return false;
			}
		},

		async switchNotesFolder(path: string) {
			try {
				const newState: {
					notes_folder: string | null;
					locale: string;
					theme: string;
					available_locales: LocaleInfo[];
					available_themes: ThemeInfo[];
					translations: Record<string, string>;
					theme_colors: Record<string, string>;
					today_note_path: string | null;
					today_note_content: string | null;
				} = await invoke("switch_notes_folder", { path });

				if (newState.notes_folder) {
					translations.set(newState.translations);
					availableLocales.set(newState.available_locales);
					locale.set(newState.locale);

					availableThemes.set(newState.available_themes);
					currentTheme.set(newState.theme);
					applyThemeColors(newState.theme_colors);

					set({
						notes_folder: newState.notes_folder,
						locale: newState.locale,
						theme: newState.theme,
					});
				}

				appState.update((state) => ({
					...state,
					todayNotePath: newState.today_note_path,
					todayNoteContent: newState.today_note_content,
					activePopup: null,
				}));

				return true;
			} catch (error) {
				console.error("Error switching notes folder:", error);
				return false;
			}
		},
	};
};

export const settings = createSettingsStore();
