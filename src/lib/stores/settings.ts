import { invoke } from "@tauri-apps/api/core";
import { writable } from "svelte/store";
import type { AppSettings } from "$lib/types/settings";
import { translations, updateTranslations } from "../utils/i18n";
import { appState } from "./appState";

const createSettingsStore = () => {
	const { subscribe, set } = writable<AppSettings>({
		notes_folder: "",
		locale: "en",
	});

	return {
		subscribe,
		set,

		async load() {
			try {
				const settings: AppSettings = await invoke("get_config");
				set(settings);
				await updateTranslations(settings.locale);
				return settings;
			} catch (error) {
				console.error("Error loading settings:", error);
				return { notes_folder: "", locale: "en" };
			}
		},

		async save(settings: AppSettings) {
			try {
				if (settings.notes_folder) {
					await invoke("set_notes_folder", { path: settings.notes_folder });
				}
				if (settings.locale) {
					await invoke("set_locale", { locale: settings.locale });
					await updateTranslations(settings.locale);
				}
				set(settings);
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
					translations: Record<string, string>;
					today_note_path: string | null;
					today_note_content: string | null;
				} = await invoke("switch_notes_folder", { path });

				if (newState.notes_folder) {
					translations.set(newState.translations);
					set({ notes_folder: newState.notes_folder, locale: newState.locale });
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
